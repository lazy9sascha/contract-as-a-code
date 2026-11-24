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
    let c = read(d.path(), "source/candidate.json");
    let p = read(d.path(), "source/provenance.json");
    let request = json!({"format":"caac-presentation-request-v1","expected_candidate_sha256":p["candidate_sha256"],"expected_package_revision":p["revision_manifest"]["package_revision"],"presentation":{"size":"M","nodes":[{"kind":"chapter","id":"chapter-one","title":"Explicit chapter","articles":c["presentation"]["nodes"]}]}});
    write(d.path(), "request", &request);
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
            "--presentation",
            "request",
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
fn s_to_m_preserves_every_semantic_field_and_exact_objects() {
    let d = setup();
    let c = read(d.path(), "source/candidate.json");
    let p = read(d.path(), "source/provenance.json");
    let o = revise(d.path());
    assert!(o.status.success(), "{:?}", o);
    assert!(verify(d.path()).status.success());
    let new = read(d.path(), "revised/candidate.json");
    let receipt = read(d.path(), "revised/provenance.json");
    assert_eq!(c["semantic"], new["semantic"]);
    assert_eq!(c["authority"], new["authority"]);
    assert_eq!(receipt["format"], "caac-candidate-provenance-v5");
    assert_eq!(
        receipt["revision_manifest"]["objects"],
        p["revision_manifest"]["objects"]
    );
    assert_ne!(
        receipt["revision_manifest"]["package_revision"],
        p["revision_manifest"]["package_revision"]
    );
    assert_eq!(
        receipt["revision_transition"]["before_mapping"],
        p["revision_manifest"]["presentation"]
    );
    assert_eq!(
        receipt["revision_transition"]["after_mapping"],
        receipt["revision_manifest"]["presentation"]
    );
}
#[test]
fn revert_can_restore_content_identity_without_restoring_authority() {
    let d = setup();
    let original = read(d.path(), "source/candidate.json");
    let old = read(d.path(), "source/provenance.json");
    assert!(revise(d.path()).status.success());
    let middle = read(d.path(), "revised/provenance.json");
    write(
        d.path(),
        "request",
        &json!({"format":"caac-presentation-request-v1","expected_candidate_sha256":middle["candidate_sha256"],"expected_package_revision":middle["revision_manifest"]["package_revision"],"presentation":original["presentation"]}),
    );
    assert!(
        run(
            d.path(),
            &[
                "candidate",
                "revise",
                "--candidate",
                "revised/candidate.json",
                "--provenance",
                "revised/provenance.json",
                "--review",
                "revised/review.md",
                "--presentation",
                "request",
                "--output",
                "reverted"
            ]
        )
        .status
        .success()
    );
    let p = read(d.path(), "reverted/provenance.json");
    assert_eq!(p["revision_manifest"], old["revision_manifest"]);
    assert_eq!(read(d.path(), "reverted/candidate.json"), original);
    assert_eq!(p["owner_acceptance"], "pending");
    assert_eq!(p["behavior_evidence"], "not-run");
    assert_ne!(
        p["revision_transition"]["parent_package_revision"],
        old["revision_manifest"]["package_revision"]
    );
}
#[test]
fn invalid_tree_and_request_preconditions_refuse_output() {
    for mode in 0..9 {
        let d = setup();
        let mut r = read(d.path(), "request");
        match mode {
            0 => r["expected_candidate_sha256"] = json!("0".repeat(64)),
            1 => r["expected_package_revision"] = json!("sha256:stale"),
            2 => r["presentation"]["size"] = json!("XL"),
            3 => r["presentation"]["nodes"][0]["articles"][0]["entities"] = json!([]),
            4 => r["presentation"]["nodes"][0]["articles"][0]["entities"][0] = json!("missing"),
            5 => r["presentation"]["nodes"][0]["articles"][0]["entities"][1] = json!("term-zero"),
            6 => r["semantic"] = json!({}),
            7 => r["format"] = json!("future"),
            _ => {
                r["presentation"] = read(d.path(), "source/candidate.json")["presentation"].clone()
            }
        };
        write(d.path(), "request", &r);
        assert!(!revise(d.path()).status.success(), "mode{mode}");
        assert!(!d.path().join("revised").exists());
    }
}
#[test]
fn v5_receipt_derived_fields_are_checked() {
    for mode in 0..10 {
        let d = setup();
        assert!(revise(d.path()).status.success());
        let mut p = read(d.path(), "revised/provenance.json");
        match mode {
            0 => p["revision_transition"]["preserved_objects"] = json!([]),
            1 => p["revision_transition"]["before_mapping"] = json!([]),
            2 => p["revision_transition"]["after_mapping"] = json!([]),
            3 => p["revision_transition"]["preserved_semantic_revision"] = json!("sha256:stale"),
            4 => p["revision_transition"]["before_presentation_revision"] = json!("sha256:stale"),
            5 => p["revision_transition"]["after_presentation_revision"] = json!("sha256:stale"),
            6 => p["revision_transition"]["operation"] = json!("replace-entity"),
            7 => p["revision_transition"]["evidence_policy"] = json!("reuse"),
            8 => p["owner_acceptance"] = json!("accepted"),
            _ => p["revision_transition"]["parent_candidate_sha256"] = json!("0".repeat(64)),
        };
        write(d.path(), "revised/provenance.json", &p);
        assert!(!verify(d.path()).status.success(), "mode{mode}");
    }
}
#[test]
fn duplicate_keys_and_exclusive_modes_are_rejected() {
    let d = setup();
    let text = fs::read_to_string(d.path().join("request")).unwrap();
    fs::write(
        d.path().join("request"),
        text.replacen("{", "{\"format\":\"caac-presentation-request-v1\",", 1),
    )
    .unwrap();
    assert!(!revise(d.path()).status.success());
    let o = run(
        d.path(),
        &[
            "candidate",
            "revise",
            "--candidate",
            "source/candidate.json",
            "--provenance",
            "source/provenance.json",
            "--review",
            "source/review.md",
            "--presentation",
            "request",
            "--proposal",
            "anything",
            "--output",
            "revised",
        ],
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(!d.path().join("revised").exists());
}
#[test]
fn move_and_renumber_shared_clause_preserves_exception_and_case_relations() {
    let d = setup();
    let c = read(d.path(), "source/candidate.json");
    let refs = c["presentation"]["nodes"][0]["entities"]
        .as_array()
        .unwrap();
    let mut r = read(d.path(), "request");
    r["presentation"] = json!({"size":"S","nodes":[{"kind":"article","id":"first","title":"Moved clause","entities":[refs[1]]},{"kind":"article","id":"second","title":"Remaining content","entities":refs.iter().enumerate().filter(|(i,_)|*i!=1).map(|(_,v)|v).collect::<Vec<_>>()}]});
    write(d.path(), "request", &r);
    assert!(revise(d.path()).status.success());
    let new = read(d.path(), "revised/candidate.json");
    assert_eq!(new["semantic"], c["semantic"]);
    let text = fs::read_to_string(d.path().join("revised/review.md")).unwrap();
    assert!(text.contains("Article 2"));
    assert_eq!(text.matches("### Clause ·").count(), 1);
    assert!(verify(d.path()).status.success());
}
#[test]
fn unchanged_package_with_different_raw_bytes_is_stale() {
    let d = setup();
    let c = read(d.path(), "source/candidate.json");
    let bytes = serde_json::to_vec(&c).unwrap();
    fs::write(d.path().join("source/candidate.json"), &bytes).unwrap();
    let mut p = read(d.path(), "source/provenance.json");
    p["candidate_sha256"] = json!(caac::candidate::digest(&bytes));
    write(d.path(), "source/provenance.json", &p);
    assert!(!revise(d.path()).status.success());
    assert!(!d.path().join("revised").exists());
}
#[test]
fn unsigned_historical_hash_claims_and_collision_boundaries() {
    let d = setup();
    assert!(revise(d.path()).status.success());
    let mut p = read(d.path(), "revised/provenance.json");
    p["revision_transition"]["parent_provenance_sha256"] = json!("0".repeat(64));
    p["revision_transition"]["input_presentation_sha256"] = json!("1".repeat(64));
    write(d.path(), "revised/provenance.json", &p);
    assert!(verify(d.path()).status.success());
    let d = setup();
    fs::create_dir(d.path().join("revised")).unwrap();
    assert!(!revise(d.path()).status.success());
}

fn paragraph_request(d: &Path) -> Value {
    let mut r = read(d, "request");
    r["presentation"]["profile"] = json!("caac-paragraph-presentation-v1");
    let article = &mut r["presentation"]["nodes"][0]["articles"][0];
    let ids = article.as_object_mut().unwrap().remove("entities").unwrap();
    let ids = ids.as_array().unwrap();
    article["paragraphs"] = json!([
        {"id":"paragraph-first","entities":&ids[..2]},
        {"id":"paragraph-second","entities":&ids[2..]}
    ]);
    r
}
fn stage_from(d: &Path, source: &str, command: &str, extra: &[&str]) -> Output {
    let c = format!("{source}/candidate.json");
    let p = format!("{source}/provenance.json");
    let r = format!("{source}/review.md");
    let mut args = vec![
        "candidate",
        command,
        "--candidate",
        &c,
        "--provenance",
        &p,
        "--review",
        &r,
    ];
    args.extend_from_slice(extra);
    run(d, &args)
}
#[test]
fn paragraphs_preserve_meaning_support_body_edits_and_restore_legacy_bytes() {
    let d = setup();
    let d = d.path();
    let source = read(d, "source/candidate.json");
    let old = read(d, "source/provenance.json");
    write(d, "request", &paragraph_request(d));
    let result = revise(d);
    assert!(result.status.success(), "{:?}", result);
    assert!(verify(d).status.success());
    let new = read(d, "revised/candidate.json");
    let p = read(d, "revised/provenance.json");
    assert_eq!(new["semantic"], source["semantic"]);
    assert_eq!(new["authority"], source["authority"]);
    assert_eq!(p["renderer_profile"], "caac-candidate-legal-review-v2");
    assert_eq!(
        p["revision_manifest"]["format"],
        "caac-candidate-revisions-v2"
    );
    assert_eq!(
        p["revision_manifest"]["objects"],
        old["revision_manifest"]["objects"]
    );
    assert_eq!(
        p["revision_manifest"]["semantic_revision"],
        old["revision_manifest"]["semantic_revision"]
    );
    let traced = stage_from(d, "revised", "verify", &["--trace", "rule-echo"]);
    assert!(traced.status.success());
    let trace: Value = serde_json::from_slice(&traced.stdout).unwrap();
    let loc = &trace["selected"]["mapping"][0]["location"];
    assert_eq!(loc["kind"], "paragraph");
    assert_eq!(loc["node_id"], "paragraph-first");
    assert_eq!(loc["chapter_id"], "chapter-one");
    assert_eq!(loc["article_number"], 1);
    assert_eq!(loc["number"], 1);
    let original_review = fs::read_to_string(d.join("revised/review.md")).unwrap();
    assert!(original_review.contains("Paragraph 1"));
    assert!(original_review.contains("Paragraph 2"));
    fs::write(
        d.join("edited"),
        original_review.replace(
            "Echo returns the supplied integer.",
            "Echo returns the supplied integer under the proposed boundary.",
        ),
    )
    .unwrap();
    let compared = stage_from(
        d,
        "revised",
        "compare-review",
        &["--edited", "edited", "--output", "proposal"],
    );
    assert!(compared.status.success(), "{:?}", compared);
    assert_eq!(
        read(d, "proposal/proposal.json")["renderer_profile"],
        "caac-candidate-legal-review-v2"
    );
    let changed = stage_from(
        d,
        "revised",
        "revise",
        &[
            "--proposal",
            "proposal/proposal.json",
            "--output",
            "body-revised",
        ],
    );
    assert!(changed.status.success(), "{:?}", changed);
    assert!(
        stage_from(d, "body-revised", "verify", &[])
            .status
            .success()
    );
    // Separate path back to the original flat placement, with profile explicitly omitted.
    let back = json!({"format":"caac-presentation-request-v1","expected_candidate_sha256":p["candidate_sha256"],"expected_package_revision":p["revision_manifest"]["package_revision"],"presentation":source["presentation"]});
    write(d, "back", &back);
    let restored = stage_from(
        d,
        "revised",
        "revise",
        &["--presentation", "back", "--output", "restored"],
    );
    assert!(restored.status.success(), "{:?}", restored);
    assert!(stage_from(d, "restored", "verify", &[]).status.success());
    for file in ["candidate.json", "review.md"] {
        assert_eq!(
            fs::read(d.join(format!("source/{file}"))).unwrap(),
            fs::read(d.join(format!("restored/{file}"))).unwrap()
        );
    }
    assert_eq!(
        read(d, "restored/provenance.json")["revision_manifest"],
        old["revision_manifest"]
    );
    // The same typed paragraph tree is admitted by ordinary apply, not just revise.
    let mut s_candidate = new.clone();
    s_candidate["presentation"]["size"] = json!("S");
    s_candidate["presentation"]["nodes"] = new["presentation"]["nodes"][0]["articles"].clone();
    write(d, "paragraph-base", &s_candidate);
    write(
        d,
        "paragraph-answers",
        &json!({"expected_base":caac::candidate::digest(&fs::read(d.join("paragraph-base")).unwrap()),"answers":[]}),
    );
    assert!(
        run(
            d,
            &[
                "candidate",
                "apply",
                "--base",
                "paragraph-base",
                "--answers",
                "paragraph-answers",
                "--output",
                "applied"
            ]
        )
        .status
        .success()
    );
    assert!(stage_from(d, "applied", "verify", &[]).status.success());
}

#[test]
fn paragraph_shapes_profiles_and_duplicate_keys_fail_closed() {
    let d = setup();
    let d = d.path();
    let valid = paragraph_request(d);
    let mut mutations = Vec::new();
    let mut v = valid.clone();
    v["presentation"].as_object_mut().unwrap().remove("profile");
    mutations.push(v);
    for value in [Value::Null, json!("unknown")] {
        let mut v = valid.clone();
        v["presentation"]["profile"] = value;
        mutations.push(v);
    }
    for field in ["entities", "paragraphs"] {
        let mut v = valid.clone();
        v["presentation"]["nodes"][0]["articles"][0][field] = Value::Null;
        mutations.push(v);
    }
    let mut v = valid.clone();
    v["presentation"]["nodes"][0]["articles"][0]["entities"] = json!(["rule-echo"]);
    mutations.push(v);
    let mut v = valid.clone();
    v["presentation"]["nodes"][0]["articles"][0]
        .as_object_mut()
        .unwrap()
        .remove("paragraphs");
    mutations.push(v);
    for value in [
        json!([]),
        json!([{"id":"p","entities":[]}]),
        json!([{"id":"p","entities":["rule-echo"],"paragraphs":[]}]),
        json!([{"id":"p","entities":["rule-echo"],"title":"not admitted"}]),
    ] {
        let mut v = valid.clone();
        v["presentation"]["nodes"][0]["articles"][0]["paragraphs"] = value;
        mutations.push(v);
    }
    for duplicate in ["paragraph-first", "chapter-one", "article-main"] {
        let mut v = valid.clone();
        v["presentation"]["nodes"][0]["articles"][0]["paragraphs"][1]["id"] = json!(duplicate);
        mutations.push(v);
    }
    let mut v = valid.clone();
    v["presentation"]["nodes"][0]["articles"][0]["paragraphs"][1]["entities"] =
        json!(["rule-echo"]);
    mutations.push(v);
    for v in mutations {
        write(d, "request", &v);
        let result = revise(d);
        assert_eq!(result.status.code(), Some(2), "{:?}", result);
        assert!(!d.join("revised").exists());
    }
    let raw = serde_json::to_string(&valid).unwrap();
    fs::write(
        d.join("request"),
        raw.replace(
            "\"paragraph-first\"",
            "\"paragraph-first\",\"id\":\"paragraph-first\"",
        ),
    )
    .unwrap();
    assert_eq!(revise(d).status.code(), Some(2));
    assert!(!d.join("revised").exists());
    // Profile is forbidden even if the flat tree is otherwise valid.
    let mut flat = read(d, "source/candidate.json")["presentation"].clone();
    flat["profile"] = json!("caac-paragraph-presentation-v1");
    let mut v = valid;
    v["presentation"] = flat;
    write(d, "request", &v);
    assert_eq!(revise(d).status.code(), Some(2));
}

#[test]
fn mixed_articles_reset_paragraph_numbers_and_reorder_without_semantic_change() {
    let d = setup();
    let d = d.path();
    let mut request = paragraph_request(d);
    let original = read(d, "source/candidate.json");
    let ids = original["presentation"]["nodes"][0]["entities"]
        .as_array()
        .unwrap();
    request["presentation"]["nodes"][0]["articles"] = json!([
        {"kind":"article","id":"article-a","title":"한글 ` <b> heading","paragraphs":[{"id":"p-a","entities":&ids[..2]}]},
        {"kind":"article","id":"article-b","title":"Second","paragraphs":[{"id":"p-b","entities":&ids[2..4]}]},
        {"kind":"article","id":"article-flat","title":"Flat","entities":&ids[4..]}
    ]);
    write(d, "request", &request);
    let out = revise(d);
    assert!(out.status.success(), "{:?}", out);
    assert!(verify(d).status.success());
    let receipt = read(d, "revised/provenance.json");
    let mappings = receipt["revision_manifest"]["presentation"]
        .as_array()
        .unwrap();
    for id in [&ids[0], &ids[2]] {
        let entry = mappings.iter().find(|x| x["target"]["id"] == *id).unwrap();
        assert_eq!(entry["location"]["number"], 1);
    }
    let flat = mappings
        .iter()
        .find(|x| x["target"]["id"] == ids[4])
        .unwrap();
    assert_eq!(flat["location"]["kind"], "article");
    let mut second = request.clone();
    second["expected_candidate_sha256"] = receipt["candidate_sha256"].clone();
    second["expected_package_revision"] = receipt["revision_manifest"]["package_revision"].clone();
    second["presentation"]["nodes"][0]["articles"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    write(d, "second", &second);
    assert!(
        stage_from(
            d,
            "revised",
            "revise",
            &["--presentation", "second", "--output", "moved"]
        )
        .status
        .success()
    );
    assert!(stage_from(d, "moved", "verify", &[]).status.success());
    let moved = read(d, "moved/provenance.json");
    assert_eq!(
        moved["revision_manifest"]["objects"],
        receipt["revision_manifest"]["objects"]
    );
    assert_eq!(
        read(d, "moved/candidate.json")["semantic"],
        original["semantic"]
    );
    let entry = moved["revision_manifest"]["presentation"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["target"]["id"] == ids[0])
        .unwrap();
    assert_eq!(entry["location"]["node_id"], "p-a");
    assert_eq!(entry["location"]["article_number"], 2);
    for field in ["renderer_profile", "revision_manifest"] {
        let mut forged = receipt.clone();
        if field == "renderer_profile" {
            forged[field] = json!("caac-candidate-legal-review-v1");
        } else {
            forged[field]["presentation"][0]["location"]["number"] = json!(9);
        }
        write(d, "revised/provenance.json", &forged);
        assert_eq!(verify(d).status.code(), Some(2));
    }
    write(d, "revised/provenance.json", &receipt);
    let raw = fs::read_to_string(d.join("revised/review.md")).unwrap();
    fs::write(
        d.join("revised/review.md"),
        raw.replace("Paragraph 1", "Paragraph 9"),
    )
    .unwrap();
    let mut forged = receipt;
    forged["review_sha256"] = json!(caac::candidate::digest(
        &fs::read(d.join("revised/review.md")).unwrap()
    ));
    write(d, "revised/provenance.json", &forged);
    assert_eq!(verify(d).status.code(), Some(2));
}

#[test]
fn profile_applicability_prose_is_not_implicit_clause_inheritance() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    // Synthetic, unapproved prose. Mentioning COMMON IDs does not create typed
    // clause-to-clause applicability or prove semantic entailment.
    let mut base: Value =
        serde_json::from_slice(include_bytes!("../examples/candidate/draft.json")).unwrap();
    base["semantic"]["namespace"] = json!("synthetic-profile-applicability");
    base["semantic"]["entities"] = json!([
        {"kind":"definition","id":"term-common","text":"Common completion vocabulary; not inherited obligations."},
        {"kind":"definition","id":"term-profile","text":"Proposed profile applicability names COMMON-001 and COMMON-002; owner selection remains unresolved."},
        {"kind":"clause","id":"COMMON-001","actor":"Core","modality":"obligation","text":"Preserve the proposed completion identity.","definition_refs":["term-common"]},
        {"kind":"clause","id":"COMMON-002","actor":"Core","modality":"obligation","text":"Preserve the proposed original acknowledgement boundary.","definition_refs":["term-common"]},
        {"kind":"clause","id":"HOST-001","actor":"Host","modality":"obligation","text":"Expose the proposed profile result.","definition_refs":["term-common","term-profile"]}
    ]);
    base["semantic"]["unresolved"] = json!([{"id":"decision-applicability","question":"Which common obligations apply to this profile?","required":true}]);
    let ids = json!([
        "term-common",
        "term-profile",
        "COMMON-001",
        "COMMON-002",
        "HOST-001"
    ]);
    base["presentation"]["nodes"][0]["entities"] = ids.clone();
    write(d, "base", &base);
    write(
        d,
        "answers",
        &json!({"expected_base":caac::candidate::digest(&fs::read(d.join("base")).unwrap()),"answers":[]}),
    );
    assert!(
        run(
            d,
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
    let review = fs::read_to_string(d.join("source/review.md")).unwrap();
    fs::write(
        d.join("edited"),
        review.replace(
            "Expose the proposed profile result.",
            "Expose the explicitly selected proposed profile result.",
        ),
    )
    .unwrap();
    assert!(
        stage_from(
            d,
            "source",
            "compare-review",
            &["--edited", "edited", "--output", "proposal"]
        )
        .status
        .success()
    );
    assert!(
        stage_from(
            d,
            "source",
            "revise",
            &["--proposal", "proposal/proposal.json", "--output", "body"]
        )
        .status
        .success()
    );
    let body = read(d, "body/candidate.json");
    assert_eq!(
        body["semantic"]["entities"][4]["text"],
        "Expose the explicitly selected proposed profile result."
    );
    for i in 0..4 {
        assert_eq!(
            body["semantic"]["entities"][i],
            base["semantic"]["entities"][i]
        );
    }
    assert_eq!(
        body["semantic"]["entities"][4]["definition_refs"],
        json!(["term-common", "term-profile"])
    );
    assert_eq!(
        body["semantic"]["unresolved"],
        base["semantic"]["unresolved"]
    );
    let grouped = json!({"size":"S","profile":"caac-paragraph-presentation-v1","nodes":[{"kind":"article","id":"article-main","title":"Proposed profile","paragraphs":[{"id":"paragraph-common","entities":["term-common","COMMON-001","COMMON-002"]},{"id":"paragraph-host","entities":["term-profile","HOST-001"]}]}]});
    for (from, to, presentation) in [
        ("body", "grouped", grouped),
        ("grouped", "restored", body["presentation"].clone()),
    ] {
        let receipt = read(d, &format!("{from}/provenance.json"));
        write(
            d,
            "request",
            &json!({"format":"caac-presentation-request-v1","expected_candidate_sha256":receipt["candidate_sha256"],"expected_package_revision":receipt["revision_manifest"]["package_revision"],"presentation":presentation}),
        );
        assert!(
            stage_from(
                d,
                from,
                "revise",
                &["--presentation", "request", "--output", to]
            )
            .status
            .success()
        );
        let candidate = read(d, &format!("{to}/candidate.json"));
        assert_eq!(candidate["semantic"], body["semantic"]);
        assert_eq!(
            candidate["authority"],
            json!({"owner_acceptance":"pending","implementation_admission":"pending"})
        );
        let p = read(d, &format!("{to}/provenance.json"));
        assert_eq!(
            p["revision_manifest"]["objects"],
            receipt["revision_manifest"]["objects"]
        );
        let output = stage_from(d, to, "verify", &["--trace", "HOST-001"]);
        assert!(output.status.success());
        let trace: Value = serde_json::from_slice(&output.stdout).unwrap();
        let targets: Vec<_> = trace["direct_outgoing"]
            .as_array()
            .unwrap()
            .iter()
            .map(|edge| edge["target"]["id"].as_str().unwrap())
            .collect();
        assert_eq!(targets, vec!["term-common", "term-profile"]);
        let reachable = trace["forward_reachable"].as_array().unwrap();
        assert!(
            !reachable
                .iter()
                .any(|r| r["id"] == "COMMON-001" || r["id"] == "COMMON-002")
        );
        assert_eq!(trace["unresolved_decisions"].as_array().unwrap().len(), 1);
    }
    for file in ["candidate.json", "review.md"] {
        assert_eq!(
            fs::read(d.join(format!("body/{file}"))).unwrap(),
            fs::read(d.join(format!("restored/{file}"))).unwrap()
        );
    }
}

fn item_request(d: &Path) -> Value {
    let mut request = paragraph_request(d);
    request["presentation"]["profile"] = json!("caac-item-presentation-v1");
    for (i, paragraph) in request["presentation"]["nodes"][0]["articles"][0]["paragraphs"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        let ids = paragraph
            .as_object_mut()
            .unwrap()
            .remove("entities")
            .unwrap();
        let ids = ids.as_array().unwrap();
        paragraph["items"] = json!([
            {"id":format!("item-{i}-a"),"entities":&ids[..1]},
            {"id":format!("item-{i}-b"),"entities":&ids[1..]}
        ]);
    }
    request
}

#[test]
fn items_preserve_semantics_support_body_revision_and_restore_old_profiles() {
    let temp = setup();
    let d = temp.path();
    let flat = read(d, "source/candidate.json");
    let mut request = item_request(d);
    write(d, "request", &paragraph_request(d));
    assert!(revise(d).status.success());
    let paragraph = read(d, "revised/candidate.json");
    let p = read(d, "revised/provenance.json");
    request["expected_candidate_sha256"] = p["candidate_sha256"].clone();
    request["expected_package_revision"] = p["revision_manifest"]["package_revision"].clone();
    write(d, "items-request", &request);
    assert!(
        stage_from(
            d,
            "revised",
            "revise",
            &["--presentation", "items-request", "--output", "items"]
        )
        .status
        .success()
    );
    let c = read(d, "items/candidate.json");
    let p = read(d, "items/provenance.json");
    assert_eq!(c["semantic"], flat["semantic"]);
    assert_eq!(c["authority"], flat["authority"]);
    assert_eq!(p["renderer_profile"], "caac-candidate-legal-review-v3");
    assert_eq!(
        p["revision_manifest"]["format"],
        "caac-candidate-revisions-v3"
    );
    let traced = stage_from(d, "items", "verify", &["--trace", "rule-echo"]);
    assert!(traced.status.success());
    let trace: Value = serde_json::from_slice(&traced.stdout).unwrap();
    assert_eq!(
        trace["selected"]["mapping"][0]["location"],
        json!({"kind":"item","node_id":"item-0-b","paragraph_id":"paragraph-first","article_id":"article-main","chapter_id":"chapter-one","article_number":1,"paragraph_number":1,"number":2})
    );
    let raw = fs::read_to_string(d.join("items/review.md")).unwrap();
    assert_eq!(raw.matches("##### Item 1\n").count(), 2); // Reset in each paragraph.
    fs::write(
        d.join("edited-items"),
        raw.replace(
            "Echo returns the supplied integer.",
            "Echo returns the supplied integer within this proposed boundary.",
        ),
    )
    .unwrap();
    let out = stage_from(
        d,
        "items",
        "compare-review",
        &["--edited", "edited-items", "--output", "item-proposal"],
    );
    assert!(out.status.success(), "{:?}", out);
    assert!(
        stage_from(
            d,
            "items",
            "revise",
            &[
                "--proposal",
                "item-proposal/proposal.json",
                "--output",
                "item-body"
            ]
        )
        .status
        .success()
    );
    assert!(stage_from(d, "item-body", "verify", &[]).status.success());
    let revised_body = read(d, "item-body/candidate.json");
    let revised_clause = revised_body["semantic"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entity| entity["id"] == "rule-echo")
        .unwrap();
    assert_eq!(
        revised_clause["text"],
        "Echo returns the supplied integer within this proposed boundary."
    );
    assert_eq!(
        read(d, "item-body/candidate.json")["presentation"],
        c["presentation"]
    );
    assert_eq!(
        read(d, "item-body/candidate.json")["semantic"]["unresolved"],
        c["semantic"]["unresolved"]
    );
    // Restore from unchanged item semantics, not from the independent body-edit branch.
    for (name, original, source) in [
        ("back-paragraph", &paragraph, "revised"),
        ("back-flat", &flat, "source"),
    ] {
        write(
            d,
            "restore-request",
            &json!({"format":"caac-presentation-request-v1","expected_candidate_sha256":p["candidate_sha256"],"expected_package_revision":p["revision_manifest"]["package_revision"],"presentation":original["presentation"]}),
        );
        assert!(
            stage_from(
                d,
                "items",
                "revise",
                &["--presentation", "restore-request", "--output", name]
            )
            .status
            .success()
        );
        assert!(stage_from(d, name, "verify", &[]).status.success());
        for file in ["candidate.json", "review.md"] {
            assert_eq!(
                fs::read(d.join(format!("{name}/{file}"))).unwrap(),
                fs::read(d.join(format!("{source}/{file}"))).unwrap()
            );
        }
    }
    // Both supported sizes admit the same item structure through ordinary apply.
    for size in ["S", "M"] {
        let mut base = c.clone();
        if size == "S" {
            base["presentation"]["size"] = json!("S");
            base["presentation"]["nodes"] = c["presentation"]["nodes"][0]["articles"].clone();
        }
        write(d, "item-base", &base);
        write(
            d,
            "item-answers",
            &json!({"expected_base":caac::candidate::digest(&fs::read(d.join("item-base")).unwrap()),"answers":[]}),
        );
        let output = format!("applied-{size}");
        assert!(
            run(
                d,
                &[
                    "candidate",
                    "apply",
                    "--base",
                    "item-base",
                    "--answers",
                    "item-answers",
                    "--output",
                    &output
                ]
            )
            .status
            .success()
        );
        assert!(stage_from(d, &output, "verify", &[]).status.success());
    }
    let mut forged = p.clone();
    forged["revision_manifest"]["presentation"][0]["location"]["paragraph_number"] = json!(8);
    write(d, "items/provenance.json", &forged);
    assert_eq!(stage_from(d, "items", "verify", &[]).status.code(), Some(2));
}

#[test]
fn invalid_item_shapes_profiles_and_placements_fail_closed() {
    let temp = setup();
    let d = temp.path();
    let valid = item_request(d);
    for mode in 0..16 {
        let mut r = valid.clone();
        let paragraph = &mut r["presentation"]["nodes"][0]["articles"][0]["paragraphs"][0];
        match mode {
            0 => paragraph["items"] = Value::Null,
            1 => paragraph["items"] = json!([]),
            2 => paragraph["entities"] = json!(["term-zero"]),
            3 => paragraph["entities"] = Value::Null,
            4 => {
                paragraph.as_object_mut().unwrap().remove("items");
            }
            5 => paragraph["items"][0]["entities"] = Value::Null,
            6 => paragraph["items"][0]["entities"] = json!([]),
            7 => paragraph["items"][0]["title"] = json!("Not supported"),
            8 => paragraph["items"][0]["items"] = json!([]),
            9 => paragraph["items"][0]["id"] = json!("paragraph-first"),
            10 => paragraph["items"][1]["entities"] = json!(["term-zero"]),
            11 => paragraph["items"][0]["entities"] = json!(["unknown"]),
            12 => r["presentation"]["profile"] = json!("caac-paragraph-presentation-v1"),
            13 => {
                r["presentation"].as_object_mut().unwrap().remove("profile");
            }
            14 => paragraph["items"][1]["id"] = json!("item-0-a"),
            15 => {
                paragraph["items"].as_array_mut().unwrap().remove(0);
            }
            _ => unreachable!(),
        }
        write(d, "request", &r);
        assert_eq!(revise(d).status.code(), Some(2), "mode {mode}");
        assert!(!d.join("revised").exists());
    }
    let mut wrong_profile = valid.clone();
    for paragraph in wrong_profile["presentation"]["nodes"][0]["articles"][0]["paragraphs"]
        .as_array_mut()
        .unwrap()
    {
        let items = paragraph.as_object_mut().unwrap().remove("items").unwrap();
        paragraph["entities"] = Value::Array(
            items
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|item| item["entities"].as_array().unwrap().iter().cloned())
                .collect(),
        );
    }
    wrong_profile["presentation"]["profile"] = json!("caac-item-presentation-v1");
    write(d, "request", &wrong_profile);
    assert_eq!(revise(d).status.code(), Some(2));
    let raw = serde_json::to_string(&valid).unwrap().replace(
        "\"id\":\"item-0-a\"",
        "\"id\":\"item-0-a\",\"id\":\"item-0-a\"",
    );
    fs::write(d.join("request"), raw).unwrap();
    assert_eq!(revise(d).status.code(), Some(2));
    assert!(!d.join("revised").exists());
}

fn apply_presentation_fixture(d: &Path, candidate: &Value, output: &str) -> Output {
    let bytes = serde_json::to_vec_pretty(candidate).unwrap();
    fs::write(d.join("presentation-base"), &bytes).unwrap();
    write(
        d,
        "presentation-answers",
        &json!({"expected_base":caac::candidate::digest(&bytes),"answers":[]}),
    );
    run(
        d,
        &[
            "candidate",
            "apply",
            "--base",
            "presentation-base",
            "--answers",
            "presentation-answers",
            "--output",
            output,
        ],
    )
}

#[test]
fn xs_single_operation_preserves_semantics_trace_and_original_bytes() {
    let temp = setup();
    let d = temp.path();
    let source = read(d, "source/candidate.json");
    let source_files = ["candidate.json", "review.md", "provenance.json"];
    let originals = source_files.map(|file| fs::read(d.join("source").join(file)).unwrap());
    let old = read(d, "source/provenance.json");
    let mut xs = source.clone();
    xs["presentation"]["size"] = json!("XS");
    let mut request = read(d, "request");
    request["presentation"] = xs["presentation"].clone();
    write(d, "request", &request);
    let result = revise(d);
    assert!(result.status.success(), "{result:?}");
    assert!(verify(d).status.success());

    let candidate = read(d, "revised/candidate.json");
    assert_eq!(candidate, xs);
    let provenance = read(d, "revised/provenance.json");
    assert_eq!(
        provenance["renderer_profile"],
        "caac-candidate-legal-review-v4"
    );
    let manifest = &provenance["revision_manifest"];
    let previous_manifest = &old["revision_manifest"];
    assert_eq!(manifest["format"], "caac-candidate-revisions-v4");
    for field in ["objects", "semantic_revision"] {
        assert_eq!(manifest[field], previous_manifest[field], "{field}");
    }
    for field in ["presentation_revision", "package_revision"] {
        assert_ne!(manifest[field], previous_manifest[field], "{field}");
    }
    let review = fs::read_to_string(d.join("revised/review.md")).unwrap();
    assert!(review.contains("## ` \"Echo behavior\" `\n"));
    assert!(review.contains("Presentation ID: ` \"article-main\" `\n"));
    assert!(!review.contains("Article 1"));
    let output = stage_from(d, "revised", "verify", &["--trace", "except-zero"]);
    assert!(output.status.success(), "{output:?}");
    let trace: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        trace["selected"]["mapping"][0]["location"],
        json!({"kind":"unnumbered-article","node_id":"article-main"})
    );
    assert_eq!(trace["direct_outgoing"][0]["target"]["id"], "rule-echo");
    assert_eq!(
        trace["direct_outgoing"][0]["target"]["package_revision"],
        manifest["package_revision"]
    );
    assert_eq!(trace["direct_incoming"][0]["source"]["id"], "case-zero");
    assert_eq!(trace["evidence_scope"]["owner_acceptance"], "pending");
    assert_eq!(
        trace["evidence_scope"]["source_behavior_evidence"],
        "not-run"
    );

    let result = apply_presentation_fixture(d, &xs, "applied-xs");
    assert!(result.status.success(), "{result:?}");
    assert!(stage_from(d, "applied-xs", "verify", &[]).status.success());
    request["expected_candidate_sha256"] = provenance["candidate_sha256"].clone();
    request["expected_package_revision"] = manifest["package_revision"].clone();
    request["presentation"] = source["presentation"].clone();
    write(d, "back", &request);
    let result = stage_from(
        d,
        "revised",
        "revise",
        &["--presentation", "back", "--output", "restored"],
    );
    assert!(result.status.success(), "{result:?}");
    assert!(stage_from(d, "restored", "verify", &[]).status.success());
    for (file, original) in source_files.into_iter().zip(originals) {
        assert_eq!(
            fs::read(d.join("source").join(file)).unwrap(),
            original,
            "{file}"
        );
        if file != "provenance.json" {
            assert_eq!(
                fs::read(d.join("restored").join(file)).unwrap(),
                original,
                "{file}"
            );
        }
    }
    assert_eq!(
        read(d, "restored/provenance.json")["revision_manifest"],
        *previous_manifest
    );
}

#[test]
fn xs_rejects_multiple_articles_grouping_profiles_and_missing_placement() {
    let temp = setup();
    let d = temp.path();
    let mut candidate = read(d, "source/candidate.json");
    let mut request = read(d, "request");
    let article = candidate["presentation"]["nodes"][0].clone();
    let ids = article["entities"].as_array().unwrap();
    let mut first = article.clone();
    first["entities"] = json!(&ids[..1]);
    let mut second = article.clone();
    second["id"] = json!("article-second");
    second["entities"] = json!(&ids[1..]);
    let mut grouped = article.clone();
    grouped.as_object_mut().unwrap().remove("entities");
    grouped["paragraphs"] = json!([{"id":"group","entities":ids}]);
    let mut omitted = article.clone();
    omitted["entities"] = json!(&ids[..ids.len() - 1]);
    let mut omitted_clause = article.clone();
    omitted_clause["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|id| id != "rule-echo");

    for (name, presentation) in [
        ("two articles", json!({"size":"XS","nodes":[first,second]})),
        (
            "chapter",
            json!({"size":"XS","nodes":[{"kind":"chapter","id":"chapter","title":"Chapter","articles":[article]}]}),
        ),
        ("paragraph", json!({"size":"XS","nodes":[grouped]})),
        (
            "profile",
            json!({"size":"XS","profile":"caac-paragraph-presentation-v1","nodes":[article]}),
        ),
        ("omitted placement", json!({"size":"XS","nodes":[omitted]})),
        (
            "omitted clause placement with unchanged semantic clause count",
            json!({"size":"XS","nodes":[omitted_clause]}),
        ),
    ] {
        candidate["presentation"] = presentation.clone();
        let output = apply_presentation_fixture(d, &candidate, "applied");
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(!d.join("applied").exists());
        request["presentation"] = presentation;
        write(d, "request", &request);
        let output = revise(d);
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(!d.join("revised").exists());
    }
}

#[test]
fn xs_requires_exactly_one_typed_operation_in_apply_and_revise() {
    let temp = setup();
    let d = temp.path();
    let source = read(d, "source/candidate.json");
    let mut empty = source.clone();
    empty["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["kind"] == "definition" || e["kind"] == "clause");
    empty["presentation"]["nodes"][0]["entities"] = json!(["term-zero", "rule-echo"]);
    let mut multiple = source.clone();
    let entities = multiple["semantic"]["entities"].as_array_mut().unwrap();
    let mut second = entities
        .iter()
        .find(|e| e["kind"] == "operation")
        .unwrap()
        .clone();
    second["id"] = json!("operation-other");
    second["name"] = json!("other");
    entities.push(second);
    multiple["presentation"]["nodes"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .push(json!("operation-other"));

    for (name, mut candidate) in [("zero-operations", empty), ("two-operations", multiple)] {
        let result = apply_presentation_fixture(d, &candidate, name);
        assert!(result.status.success(), "S fixture {name}: {result:?}");
        candidate["presentation"]["size"] = json!("XS");
        let result = apply_presentation_fixture(d, &candidate, "applied");
        assert_eq!(result.status.code(), Some(2), "{name}: {result:?}");
        assert!(!d.join("applied").exists());

        let p = read(d, &format!("{name}/provenance.json"));
        let mut request = read(d, "request");
        request["expected_candidate_sha256"] = p["candidate_sha256"].clone();
        request["expected_package_revision"] = p["revision_manifest"]["package_revision"].clone();
        request["presentation"] = candidate["presentation"].clone();
        write(d, "reject-xs", &request);
        let result = stage_from(
            d,
            name,
            "revise",
            &["--presentation", "reject-xs", "--output", "rejected"],
        );
        assert_eq!(result.status.code(), Some(2), "{name}: {result:?}");
        assert!(!d.join("rejected").exists());
    }
}
