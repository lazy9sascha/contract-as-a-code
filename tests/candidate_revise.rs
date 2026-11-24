use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(d: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(d)
        .args(args)
        .output()
        .unwrap()
}
fn read(d: &Path, p: &str) -> Value {
    serde_json::from_slice(&fs::read(d.join(p)).unwrap()).unwrap()
}
fn write(d: &Path, p: &str, v: &Value) {
    fs::write(d.join(p), serde_json::to_vec_pretty(v).unwrap()).unwrap();
}
fn setup() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::write(
        d.path().join("base"),
        include_bytes!("../examples/candidate/draft.json"),
    )
    .unwrap();
    fs::write(
        d.path().join("answers"),
        include_bytes!("../examples/candidate/answers.json"),
    )
    .unwrap();
    assert!(
        run(
            d.path(),
            &[
                "candidate",
                "apply",
                "--base",
                "base",
                "--answers",
                "answers",
                "--output",
                "source"
            ]
        )
        .status
        .success()
    );
    let text = fs::read_to_string(d.path().join("source/review.md"))
        .unwrap()
        .replace(
            "Echo returns the supplied integer.",
            "Echo returns the explicitly proposed integer.",
        );
    fs::write(d.path().join("edited"), text).unwrap();
    assert!(
        run(
            d.path(),
            &[
                "candidate",
                "compare-review",
                "--candidate",
                "source/candidate.json",
                "--provenance",
                "source/provenance.json",
                "--review",
                "source/review.md",
                "--edited",
                "edited",
                "--output",
                "proposal"
            ]
        )
        .status
        .success()
    );
    d
}
fn revise(d: &Path) -> Output {
    run(
        d,
        &[
            "candidate",
            "revise",
            "--candidate",
            "source/candidate.json",
            "--provenance",
            "source/provenance.json",
            "--review",
            "source/review.md",
            "--proposal",
            "proposal/proposal.json",
            "--output",
            "revised",
        ],
    )
}
fn verify(d: &Path) -> Output {
    run(
        d,
        &[
            "candidate",
            "verify",
            "--candidate",
            "revised/candidate.json",
            "--provenance",
            "revised/provenance.json",
            "--review",
            "revised/review.md",
        ],
    )
}
#[test]
fn explicit_revision_creates_verified_pending_v4_without_inherited_history() {
    let d = setup();
    let old = fs::read(d.path().join("source/candidate.json")).unwrap();
    let o = revise(d.path());
    assert!(o.status.success(), "{:?}", o);
    assert!(verify(d.path()).status.success());
    let p = read(d.path(), "revised/provenance.json");
    assert_eq!(p["format"], "caac-candidate-provenance-v4");
    assert!(p.get("answers_sha256").is_none());
    assert!(p.get("field_changes").is_none());
    assert_eq!(
        p["revision_transition"]["changed_objects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        p["revision_transition"]["evidence_policy"],
        "fresh-review-required-for-whole-package"
    );
    assert_eq!(p["owner_acceptance"], "pending");
    assert_eq!(p["behavior_evidence"], "not-run");
    assert_eq!(
        fs::read(d.path().join("source/candidate.json")).unwrap(),
        old
    );
    assert_ne!(
        p["revision_manifest"]["package_revision"],
        p["revision_transition"]["parent_package_revision"]
    );
    for r in p["revision_transition"]["reference_preconditions"]
        .as_array()
        .unwrap()
    {
        assert_eq!(
            r["target"]["package_revision"],
            p["revision_transition"]["parent_package_revision"]
        );
    }
}
#[test]
fn altered_proposal_fields_or_raw_edits_are_not_trusted() {
    for mode in 0..8 {
        let d = setup();
        let mut p = read(d.path(), "proposal/proposal.json");
        match mode {
            0 => p["replacements"][0]["expected_revision"] = json!("sha256:stale"),
            1 => p["source_revisions"]["package_revision"] = json!("sha256:stale"),
            2 => p["replacements"][0]["after"]["text"] = json!("Forged"),
            3 => p["status"] = json!("accepted"),
            4 => p["owner_acceptance"] = json!("accepted"),
            5 => p["raw_review_change"]["after"] = json!("Ignored extra document"),
            6 => p["extra"] = json!(true),
            _ => p["source_candidate_sha256"] = json!("0".repeat(64)),
        }
        write(d.path(), "proposal/proposal.json", &p);
        assert!(!revise(d.path()).status.success(), "mode{mode}");
        assert!(!d.path().join("revised").exists());
    }
}
#[test]
fn duplicate_proposal_keys_refused_before_value_conversion() {
    let d = setup();
    let p = fs::read_to_string(d.path().join("proposal/proposal.json")).unwrap();
    fs::write(
        d.path().join("proposal/proposal.json"),
        p.replacen("{", "{\"status\":\"proposed\",", 1),
    )
    .unwrap();
    assert!(!revise(d.path()).status.success());
    assert!(!d.path().join("revised").exists());
}
#[test]
fn every_derived_transition_component_is_verified() {
    for mode in 0..10 {
        let d = setup();
        assert!(revise(d.path()).status.success());
        let mut p = read(d.path(), "revised/provenance.json");
        match mode {
            0 => p["revision_transition"]["changed_objects"] = json!([]),
            1 => {
                p["revision_transition"]["changed_objects"][0]["after_revision"] =
                    json!("sha256:stale")
            }
            2 => p["revision_transition"]["parent_package_revision"] = json!("sha256:stale"),
            3 => {
                p["revision_transition"]["reference_preconditions"][0]["target"]["entity_revision"] =
                    json!("sha256:stale")
            }
            4 => p["revision_transition"]["reference_preconditions"] = json!([]),
            5 => p["revision_transition"]["evidence_policy"] = json!("reuse"),
            6 => p["revision_transition"]["parent_candidate_sha256"] = json!("0".repeat(64)),
            7 => p["owner_acceptance"] = json!("accepted"),
            8 => p["revision_transition"]["parent_review_sha256"] = json!("0".repeat(64)),
            _ => p["revision_transition"]["input_proposal_sha256"] = json!("bad"),
        };
        write(d.path(), "revised/provenance.json", &p);
        assert!(!verify(d.path()).status.success(), "mode{mode}");
    }
}
#[test]
fn historical_hash_claims_are_not_authentication() {
    let d = setup();
    assert!(revise(d.path()).status.success());
    let mut p = read(d.path(), "revised/provenance.json");
    p["revision_transition"]["parent_provenance_sha256"] = json!("0".repeat(64));
    p["revision_transition"]["input_proposal_sha256"] = json!("1".repeat(64));
    write(d.path(), "revised/provenance.json", &p);
    assert!(verify(d.path()).status.success());
}
#[test]
fn collision_and_stale_parent_refuse_publication() {
    let d = setup();
    fs::create_dir(d.path().join("revised")).unwrap();
    assert!(!revise(d.path()).status.success());
    let d = setup();
    let mut c = read(d.path(), "source/candidate.json");
    c["presentation"]["nodes"][0]["title"] = json!("New title");
    write(d.path(), "source/candidate.json", &c);
    assert!(!revise(d.path()).status.success());
    assert!(!d.path().join("revised").exists());
}
#[test]
fn same_revision_transition_is_deterministic() {
    let a = setup();
    let b = setup();
    assert!(revise(a.path()).status.success());
    assert!(revise(b.path()).status.success());
    for p in ["candidate.json", "review.md", "provenance.json"] {
        assert_eq!(
            fs::read(a.path().join("revised").join(p)).unwrap(),
            fs::read(b.path().join("revised").join(p)).unwrap()
        );
    }
}

fn proposal_for(d: &Path, target: Value) {
    let text = caac::candidate::render(&serde_json::from_value(target).unwrap()).unwrap();
    let marker = "## Exact source revisions and review mapping";
    let source = fs::read_to_string(d.join("source/review.md")).unwrap();
    let edited = format!(
        "{}{}{}",
        text.split_once(marker).unwrap().0,
        marker,
        source.split_once(marker).unwrap().1
    );
    fs::write(d.join("edited"), edited).unwrap();
    let o = run(
        d,
        &[
            "candidate",
            "compare-review",
            "--candidate",
            "source/candidate.json",
            "--provenance",
            "source/provenance.json",
            "--review",
            "source/review.md",
            "--edited",
            "edited",
            "--output",
            "proposal2",
        ],
    );
    assert!(o.status.success(), "{:?}", o);
    fs::copy(
        d.join("proposal2/proposal.json"),
        d.join("proposal/proposal.json"),
    )
    .unwrap();
}
#[test]
fn changed_dependency_retains_parent_precondition_and_no_evidence_reuse() {
    let d = setup();
    let mut c = read(d.path(), "source/candidate.json");
    c["semantic"]["entities"][0]["text"] = json!("A proposed revised definition.");
    proposal_for(d.path(), c);
    assert!(revise(d.path()).status.success());
    assert!(verify(d.path()).status.success());
    let old = read(d.path(), "source/provenance.json");
    let p = read(d.path(), "revised/provenance.json");
    assert_eq!(
        p["revision_transition"]["changed_objects"][0]["id"],
        "term-zero"
    );
    let pre = p["revision_transition"]["reference_preconditions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"]["id"] == "term-zero")
        .unwrap();
    let post = p["revision_manifest"]["references"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["target"]["id"] == "term-zero")
        .unwrap();
    assert_eq!(
        pre["target"]["package_revision"],
        old["revision_manifest"]["package_revision"]
    );
    assert_ne!(
        pre["target"]["entity_revision"],
        post["target"]["entity_revision"]
    );
    assert_eq!(
        pre["source"]["entity_revision"],
        post["source"]["entity_revision"]
    );
}
#[test]
fn internally_valid_new_parent_still_rejects_old_proposal() {
    let d = setup();
    let mut c = read(d.path(), "source/candidate.json");
    c["presentation"]["nodes"][0]["title"] = json!("Moved presentation context");
    let bytes = serde_json::to_vec(&c).unwrap();
    fs::write(d.path().join("newbase"), &bytes).unwrap();
    write(
        d.path(),
        "emptyanswers",
        &json!({"expected_base":caac::candidate::digest(&bytes),"answers":[]}),
    );
    assert!(
        run(
            d.path(),
            &[
                "candidate",
                "apply",
                "--base",
                "newbase",
                "--answers",
                "emptyanswers",
                "--output",
                "newsource"
            ]
        )
        .status
        .success()
    );
    fs::rename(d.path().join("source"), d.path().join("oldsource")).unwrap();
    fs::rename(d.path().join("newsource"), d.path().join("source")).unwrap();
    assert!(!revise(d.path()).status.success());
    assert!(!d.path().join("revised").exists());
}
#[test]
fn no_op_and_conflict_proposals_never_create_revisions() {
    for suffix in ["", "\n", "\nUnsupported obligation\n"] {
        let d = setup();
        let s = fs::read_to_string(d.path().join("source/review.md")).unwrap() + suffix;
        fs::write(d.path().join("edited"), s).unwrap();
        run(
            d.path(),
            &[
                "candidate",
                "compare-review",
                "--candidate",
                "source/candidate.json",
                "--provenance",
                "source/provenance.json",
                "--review",
                "source/review.md",
                "--edited",
                "edited",
                "--output",
                "proposal2",
            ],
        );
        fs::copy(
            d.path().join("proposal2/proposal.json"),
            d.path().join("proposal/proposal.json"),
        )
        .unwrap();
        assert!(!revise(d.path()).status.success());
        assert!(!d.path().join("revised").exists());
    }
}
