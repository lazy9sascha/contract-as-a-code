use caac::candidate::{Candidate, RevisionManifest, revisions, verify_revisions};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn candidate() -> Value {
    let mut b: Value =
        serde_json::from_str(include_str!("../examples/candidate/draft.json")).unwrap();
    let a: Value =
        serde_json::from_str(include_str!("../examples/candidate/answers.json")).unwrap();
    b["semantic"]["entities"] = json!(
        a["answers"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|a| a.get("value").cloned())
            .collect::<Vec<_>>()
    );
    b
}
fn manifest(value: &Value) -> Value {
    serde_json::to_value(
        revisions(&serde_json::from_value::<Candidate>(value.clone()).unwrap()).unwrap(),
    )
    .unwrap()
}
fn object<'a>(m: &'a Value, id: &str) -> &'a Value {
    m["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == id)
        .unwrap()
}
fn setup(d: &Path) {
    let b = candidate();
    let bytes = serde_json::to_vec_pretty(&b).unwrap();
    fs::write(d.join("base.json"), &bytes).unwrap();
    fs::write(
        d.join("answers.json"),
        serde_json::to_vec(&json!({"expected_base":caac::candidate::digest(&bytes),"answers":[]}))
            .unwrap(),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["candidate", "apply", "--base"])
        .arg(d.join("base.json"))
        .arg("--answers")
        .arg(d.join("answers.json"))
        .arg("--output")
        .arg(d.join("out"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn verify(d: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["candidate", "verify", "--candidate"])
        .arg(d.join("out/candidate.json"))
        .arg("--provenance")
        .arg(d.join("out/provenance.json"))
        .arg("--review")
        .arg(d.join("out/review.md"))
        .output()
        .unwrap()
}
fn read(d: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(d.join(name)).unwrap()).unwrap()
}
fn write(d: &Path, name: &str, value: &Value) {
    fs::write(d.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
#[test]
fn real_cli_verifies_exact_revision_bindings_and_review_without_mutation() {
    let d = tempfile::tempdir().unwrap();
    setup(d.path());
    let out = verify(d.path());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("Original input provenance"));
    let p = read(d.path(), "out/provenance.json");
    assert_eq!(p["format"], "caac-candidate-provenance-v3");
    let m = &p["revision_manifest"];
    assert_eq!(m["objects"].as_array().unwrap().len(), 7);
    assert_eq!(object(m, "decision-scope")["kind"], "decision");
    let review = fs::read_to_string(d.path().join("out/review.md")).unwrap();
    for name in [
        "package_revision",
        "semantic_revision",
        "presentation_revision",
    ] {
        assert!(review.contains(m[name].as_str().unwrap()));
    }
    for binding in m["references"].as_array().unwrap() {
        assert_eq!(binding["source"]["package_revision"], m["package_revision"]);
        assert_eq!(binding["target"]["package_revision"], m["package_revision"]);
        assert_eq!(
            binding["target"]["entity_revision"],
            object(m, binding["target"]["id"].as_str().unwrap())["revision"]
        );
    }
    assert_eq!(fs::read_dir(d.path().join("out")).unwrap().count(), 3);
}
#[test]
fn presentation_move_changes_package_not_semantic_or_object_revisions() {
    let mut c = candidate();
    let old = manifest(&c);
    let articles = c["presentation"]["nodes"].clone();
    c["presentation"] = json!({"size":"M","nodes":[{"kind":"chapter","id":"chapter-one","title":"Moved","articles":articles}]});
    let new = manifest(&c);
    assert_ne!(old["package_revision"], new["package_revision"]);
    assert_ne!(old["presentation_revision"], new["presentation_revision"]);
    assert_eq!(old["semantic_revision"], new["semantic_revision"]);
    assert_eq!(old["objects"], new["objects"]);
    assert_ne!(old["presentation"], new["presentation"]);
}
#[test]
fn direct_content_revision_is_not_effective_meaning_or_transitive_evidence() {
    let mut c = candidate();
    let old = manifest(&c);
    c["semantic"]["entities"][0]["text"] = json!("Zero has a changed proposed meaning.");
    let new = manifest(&c);
    assert_ne!(
        object(&old, "term-zero")["revision"],
        object(&new, "term-zero")["revision"]
    );
    assert_eq!(
        object(&old, "rule-echo")["revision"],
        object(&new, "rule-echo")["revision"]
    );
    assert_ne!(old["semantic_revision"], new["semantic_revision"]);
    assert_ne!(
        old["references"][0]["target"]["entity_revision"],
        new["references"][0]["target"]["entity_revision"]
    );
    let typed: Candidate = serde_json::from_value(c).unwrap();
    let stale: RevisionManifest = serde_json::from_value(old).unwrap();
    assert!(verify_revisions(&typed, &stale).is_err());
}
#[test]
fn namespace_and_decision_proposals_are_revision_inputs() {
    let mut c = candidate();
    let old = manifest(&c);
    c["semantic"]["namespace"] = json!("different-contract");
    let new = manifest(&c);
    assert_ne!(
        object(&old, "term-zero")["revision"],
        object(&new, "term-zero")["revision"]
    );
    assert_ne!(old["package_revision"], new["package_revision"]);
    c["semantic"]["unresolved"][0]["proposal"] =
        json!({"text":"Review scope","entity_refs":["operation-echo"]});
    let with_proposal = manifest(&c);
    assert_ne!(
        object(&new, "decision-scope")["revision"],
        object(&with_proposal, "decision-scope")["revision"]
    );
    assert!(
        with_proposal["references"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["source"]["kind"] == "decision" && r["field"] == "proposal_entity_refs")
    );
}
#[test]
fn typed_revisions_ignore_json_formatting_but_raw_checksums_do_not() {
    let d = tempfile::tempdir().unwrap();
    setup(d.path());
    let c = read(d.path(), "out/candidate.json");
    let compact = serde_json::to_vec(&c).unwrap();
    fs::write(d.path().join("out/candidate.json"), &compact).unwrap();
    assert!(!verify(d.path()).status.success());
    let mut p = read(d.path(), "out/provenance.json");
    let typed: Candidate = serde_json::from_value(c).unwrap();
    let m: RevisionManifest = serde_json::from_value(p["revision_manifest"].clone()).unwrap();
    assert!(verify_revisions(&typed, &m).is_ok());
    p["candidate_sha256"] = json!(caac::candidate::digest(&compact));
    write(d.path(), "out/provenance.json", &p);
    assert!(verify(d.path()).status.success());
}
#[test]
fn stale_missing_extra_wrong_kind_or_wrong_version_bindings_are_refused() {
    for attack in 0..10 {
        let d = tempfile::tempdir().unwrap();
        setup(d.path());
        let mut p = read(d.path(), "out/provenance.json");
        let m = &mut p["revision_manifest"];
        match attack {
            0 => {
                m["objects"].as_array_mut().unwrap().pop();
            }
            1 => {
                let extra = m["objects"][0].clone();
                m["objects"].as_array_mut().unwrap().push(extra);
            }
            2 => m["references"][0]["target"]["kind"] = json!("operation"),
            3 => m["references"][0]["target"]["namespace"] = json!("other"),
            4 => m["references"][0]["target"]["entity_revision"] = json!("sha256:stale"),
            5 => m["references"][0]["index"] = json!(9),
            6 => m["format"] = json!("caac-candidate-revisions-v99"),
            7 => m["encoding"] = json!("unknown"),
            8 => {
                m["references"].as_array_mut().unwrap().clear();
            }
            _ => p["format"] = json!("caac-candidate-provenance-v1"),
        }
        write(d.path(), "out/provenance.json", &p);
        let before = fs::read(d.path().join("out/provenance.json")).unwrap();
        assert!(!verify(d.path()).status.success(), "attack {attack}");
        assert_eq!(
            fs::read(d.path().join("out/provenance.json")).unwrap(),
            before
        );
    }
}
#[test]
fn forged_authority_or_modified_review_fails_even_with_updated_view_checksum() {
    for attack in 0..3 {
        let d = tempfile::tempdir().unwrap();
        setup(d.path());
        let mut p = read(d.path(), "out/provenance.json");
        match attack {
            0 => p["owner_acceptance"] = json!("accepted"),
            1 => p["contract_complete"] = json!(true),
            _ => {
                let view = b"# Forged accepted contract\n";
                fs::write(d.path().join("out/review.md"), view).unwrap();
                p["review_sha256"] = json!(caac::candidate::digest(view));
            }
        }
        write(d.path(), "out/provenance.json", &p);
        assert!(!verify(d.path()).status.success());
    }
}
#[test]
fn duplicate_fields_and_unknown_manifest_fields_are_refused() {
    let d = tempfile::tempdir().unwrap();
    setup(d.path());
    let p = fs::read_to_string(d.path().join("out/provenance.json")).unwrap();
    let bad = p.replacen(
        "\"namespace\": \"synthetic-echo\"",
        "\"namespace\": \"synthetic-echo\", \"namespace\": \"synthetic-echo\"",
        1,
    );
    assert_ne!(p, bad);
    fs::write(d.path().join("out/provenance.json"), bad).unwrap();
    assert!(!verify(d.path()).status.success());
    let mut p: Value = serde_json::from_str(&p).unwrap();
    p["revision_manifest"]["unknown"] = json!(true);
    write(d.path(), "out/provenance.json", &p);
    assert!(!verify(d.path()).status.success());
}
#[test]
fn nullable_location_keys_may_be_omitted_without_changing_typed_identity() {
    let d = tempfile::tempdir().unwrap();
    setup(d.path());
    let mut p = read(d.path(), "out/provenance.json");
    for r in p["revision_manifest"]["references"].as_array_mut().unwrap() {
        if r["index"].is_null() {
            r.as_object_mut().unwrap().remove("index");
        }
    }
    for r in p["revision_manifest"]["presentation"]
        .as_array_mut()
        .unwrap()
    {
        if r["location"]["kind"] == "article" {
            r["location"].as_object_mut().unwrap().remove("chapter_id");
        }
    }
    write(d.path(), "out/provenance.json", &p);
    assert!(verify(d.path()).status.success());
}
#[test]
fn combined_candidate_size_is_bounded_before_publication() {
    let d = tempfile::tempdir().unwrap();
    let body = "x".repeat(600_000);
    let b = json!({"format":"caac-candidate-v1","semantic":{"namespace":"large","entities":[{"kind":"definition","id":"first","text":body}],"unresolved":[]},"presentation":{"size":"S","nodes":[{"kind":"article","id":"article","title":"Large","entities":["first","second"]}]},"authority":{"owner_acceptance":"pending","implementation_admission":"pending"}});
    let bytes = serde_json::to_vec(&b).unwrap();
    assert!(bytes.len() < 1024 * 1024);
    fs::write(d.path().join("base.json"), &bytes).unwrap();
    let a = json!({"expected_base":caac::candidate::digest(&bytes),"answers":[{"action":"propose-entity","answer_id":"second-answer","target":{"kind":"definition","id":"second"},"value":{"kind":"definition","id":"second","text":body},"source":"synthetic"}]});
    let bytes = serde_json::to_vec(&a).unwrap();
    assert!(bytes.len() < 1024 * 1024);
    fs::write(d.path().join("answers.json"), bytes).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["candidate", "apply", "--base"])
        .arg(d.path().join("base.json"))
        .arg("--answers")
        .arg(d.path().join("answers.json"))
        .arg("--output")
        .arg(d.path().join("out"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!d.path().join("out").exists());
}

#[test]
fn semantic_array_order_is_revision_content_but_map_key_order_is_not() {
    let mut c = candidate();
    let old = manifest(&c);
    c["semantic"]["entities"].as_array_mut().unwrap().reverse();
    let reordered = manifest(&c);
    assert_ne!(old["semantic_revision"], reordered["semantic_revision"]);
    for e in old["objects"].as_array().unwrap() {
        assert_eq!(
            e["revision"],
            object(&reordered, e["id"].as_str().unwrap())["revision"]
        );
    }
    let compact = serde_json::to_string(&c).unwrap();
    let pretty = serde_json::to_string_pretty(&c).unwrap();
    let left: Candidate = serde_json::from_str(&compact).unwrap();
    let right: Candidate = serde_json::from_str(&pretty).unwrap();
    assert_eq!(revisions(&left).unwrap(), revisions(&right).unwrap());
}
#[test]
fn exception_change_is_not_erased_by_unchanged_clause_body() {
    let mut c = candidate();
    let old = manifest(&c);
    c["semantic"]["entities"][3]["applicability"]["value"] = json!(1);
    let new = manifest(&c);
    assert_eq!(
        object(&old, "rule-echo")["revision"],
        object(&new, "rule-echo")["revision"]
    );
    assert_ne!(
        object(&old, "except-zero")["revision"],
        object(&new, "except-zero")["revision"]
    );
    assert_ne!(old["package_revision"], new["package_revision"]);
}
