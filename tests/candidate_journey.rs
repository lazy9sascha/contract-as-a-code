//! One synthetic, explicitly unapproved user journey across the bounded APIs.
//! Successful artifact verification is not owner acceptance or executed behavior evidence.
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
fn ok(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn read(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}
fn stage(root: &Path, command: &str, source: &str, extra: &[&str]) -> Output {
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
    run(root, &args)
}
fn trace(root: &Path, source: &str) -> Value {
    serde_json::from_slice(&ok(stage(root, "verify", source, &["--trace", "rule-echo"])).stdout)
        .unwrap()
}
fn rejected(root: &Path, output: Output, path: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.join(path).exists());
}

#[test]
fn guided_interview_review_revision_and_reframe_keep_exact_pending_lineage() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    // Read the real exported shape as authoring guidance, not as a semantic validator.
    let guide: Value =
        serde_json::from_slice(&ok(run(root, &["candidate", "schema"])).stdout).unwrap();
    assert_eq!(guide["title"], "Entity");
    assert!(
        guide["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["properties"]["kind"]["const"] == "clause")
    );
    let base = include_bytes!("../examples/candidate/draft.json");
    fs::write(root.join("draft.json"), base).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(root)
        .args([
            "candidate",
            "interview",
            "--base",
            "draft.json",
            "--answers",
            "journal.json",
            "--output",
            "interview",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(include_bytes!(
            "../examples/candidate/interview-input.jsonl"
        ))
        .unwrap();
    ok(child.wait_with_output().unwrap());
    let journal = fs::read(root.join("journal.json")).unwrap();
    let original: Vec<_> = ["candidate.json", "provenance.json", "review.md"]
        .iter()
        .map(|f| {
            (
                format!("interview/{f}"),
                fs::read(root.join(format!("interview/{f}"))).unwrap(),
            )
        })
        .collect();
    let first = read(root, "interview/candidate.json");
    let first_trace = trace(root, "interview");
    let old_text = "Echo returns the supplied integer.";
    let new_text = "Echo returns the supplied integer under this proposed scope.";
    let review = String::from_utf8(original[2].1.clone()).unwrap();
    assert_eq!(review.matches(old_text).count(), 2); // Labeled field and exact projection must agree.
    fs::write(root.join("edited.md"), review.replace(old_text, new_text)).unwrap();
    ok(stage(
        root,
        "compare-review",
        "interview",
        &["--edited", "edited.md", "--output", "proposal"],
    ));
    let proposal_bytes = fs::read(root.join("proposal/proposal.json")).unwrap();
    let proposal = read(root, "proposal/proposal.json");
    assert_eq!(proposal["status"], "proposed");
    ok(stage(
        root,
        "revise",
        "interview",
        &[
            "--proposal",
            "proposal/proposal.json",
            "--output",
            "revised",
        ],
    ));
    ok(stage(root, "verify", "revised", &[]));
    let revised = read(root, "revised/candidate.json");
    let receipt = read(root, "revised/provenance.json");
    let revised_trace = trace(root, "revised");
    let revised_inputs: Vec<_> = ["candidate.json", "provenance.json", "review.md"]
        .iter()
        .map(|f| {
            (
                format!("revised/{f}"),
                fs::read(root.join(format!("revised/{f}"))).unwrap(),
            )
        })
        .collect();
    assert_eq!(receipt["format"], "caac-candidate-provenance-v4");
    assert_eq!(revised_trace["selected"]["body"]["text"], new_text);
    assert_ne!(
        first_trace["selected"]["reference"]["entity_revision"],
        revised_trace["selected"]["reference"]["entity_revision"]
    );
    assert_eq!(
        first["semantic"]["unresolved"],
        revised["semantic"]["unresolved"]
    );
    // Replaying the original proposal on the new exact source cannot publish.
    rejected(
        root,
        stage(
            root,
            "revise",
            "revised",
            &[
                "--proposal",
                "proposal/proposal.json",
                "--output",
                "stale-proposal",
            ],
        ),
        "stale-proposal",
    );
    let request = json!({"format":"caac-presentation-request-v1", "expected_candidate_sha256":receipt["candidate_sha256"], "expected_package_revision":receipt["revision_manifest"]["package_revision"], "presentation":{"size":"M","nodes":[{"kind":"chapter","id":"chapter-proposed","title":"Synthetic scope, unapproved","articles":revised["presentation"]["nodes"]}]}});
    fs::write(
        root.join("presentation.json"),
        serde_json::to_vec_pretty(&request).unwrap(),
    )
    .unwrap();
    let request_bytes = fs::read(root.join("presentation.json")).unwrap();
    ok(stage(
        root,
        "revise",
        "revised",
        &[
            "--presentation",
            "presentation.json",
            "--output",
            "reframed",
        ],
    ));
    ok(stage(root, "verify", "reframed", &[]));
    let final_candidate = read(root, "reframed/candidate.json");
    let final_receipt = read(root, "reframed/provenance.json");
    let final_trace = trace(root, "reframed");
    rejected(
        root,
        stage(
            root,
            "revise",
            "reframed",
            &[
                "--presentation",
                "presentation.json",
                "--output",
                "stale-presentation",
            ],
        ),
        "stale-presentation",
    );
    assert_eq!(final_receipt["format"], "caac-candidate-provenance-v5");
    assert_eq!(final_candidate["semantic"], revised["semantic"]);
    assert_eq!(
        final_receipt["revision_manifest"]["objects"],
        receipt["revision_manifest"]["objects"]
    );
    assert_eq!(
        final_receipt["revision_manifest"]["semantic_revision"],
        receipt["revision_manifest"]["semantic_revision"]
    );
    assert_ne!(
        final_trace["source"]["package_revision"],
        revised_trace["source"]["package_revision"]
    );
    assert_ne!(
        final_trace["selected"]["mapping"],
        revised_trace["selected"]["mapping"]
    );
    for report in [&first_trace, &revised_trace, &final_trace] {
        assert_eq!(
            report["evidence_scope"]["source_behavior_evidence"],
            "not-run"
        );
        assert_eq!(
            report["evidence_scope"]["execution_evidence"],
            "not-recorded-in-supplied-artifacts"
        );
        assert_eq!(report["evidence_scope"]["owner_acceptance"], "pending");
        assert_eq!(report["evidence_scope"]["completion"], "not-assessed");
        assert_eq!(report["unresolved_decisions"][0]["status"], "unresolved");
        assert_eq!(
            report["reachable_declared_cases"].as_array().unwrap().len(),
            2
        );
    }
    // Relationship addresses must bind the revised clause in the final package,
    // including an operation and exception whose direct content did not change.
    for source_id in ["operation-echo", "except-zero"] {
        let edge = final_trace["direct_incoming"]
            .as_array()
            .unwrap()
            .iter()
            .find(|edge| edge["source"]["id"] == source_id)
            .unwrap();
        assert_eq!(edge["target"], final_trace["selected"]["reference"]);
        assert_eq!(
            edge["source"]["package_revision"],
            final_trace["source"]["package_revision"]
        );
        assert_eq!(
            edge["target"]["entity_revision"],
            revised_trace["selected"]["reference"]["entity_revision"]
        );
    }
    for case in final_trace["reachable_declared_cases"].as_array().unwrap() {
        assert_eq!(
            case["reference"]["package_revision"],
            final_trace["source"]["package_revision"]
        );
        assert_eq!(case["body"]["clause_refs"], json!(["rule-echo"]));
        let edge = final_trace["direct_incoming"]
            .as_array()
            .unwrap()
            .iter()
            .find(|edge| edge["source"]["id"] == case["reference"]["id"])
            .unwrap();
        assert_eq!(edge["source"], case["reference"]);
        assert_eq!(edge["target"], final_trace["selected"]["reference"]);
    }
    assert_eq!(final_candidate["authority"], first["authority"]);
    // Original document/provenance cannot be mixed into a newer package.
    rejected(
        root,
        run(
            root,
            &[
                "candidate",
                "compare-review",
                "--candidate",
                "reframed/candidate.json",
                "--provenance",
                "interview/provenance.json",
                "--review",
                "interview/review.md",
                "--edited",
                "edited.md",
                "--output",
                "stale-artifacts",
            ],
        ),
        "stale-artifacts",
    );
    assert_eq!(fs::read(root.join("draft.json")).unwrap(), base);
    assert_eq!(fs::read(root.join("journal.json")).unwrap(), journal);
    assert_eq!(
        fs::read(root.join("proposal/proposal.json")).unwrap(),
        proposal_bytes
    );
    assert_eq!(
        fs::read(root.join("presentation.json")).unwrap(),
        request_bytes
    );
    for (path, bytes) in revised_inputs {
        assert_eq!(fs::read(root.join(path)).unwrap(), bytes);
    }
    for (path, bytes) in original {
        assert_eq!(fs::read(root.join(path)).unwrap(), bytes);
    }
}

#[test]
fn additional_case_observable_stays_a_pending_linked_proposal_not_a_clause_rewrite() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    // Synthetic prose-only buffer contract: no fake i32 Buffer operation or typed case.
    // Bounds plus padding do not entail final-value preservation outside the block.
    // Strict write isolation is different: same-value writes can preserve final values.
    let base = json!({
        "format":"caac-candidate-v1",
        "semantic":{"namespace":"unapproved-buffer-review","entities":[
            {"kind":"clause","id":"BUFFER-001","actor":"Formatter","modality":"obligation",
             "text":"Keep all writes within the supplied buffer bounds.","definition_refs":[]},
            {"kind":"clause","id":"BUFFER-002","actor":"Formatter","modality":"obligation",
             "text":"Produce sixteen zero-padded decimal digits for the selected block.","definition_refs":[]}
        ],"unresolved":[{"id":"decision-block-footprint","question":"Should block formatting preserve adjacent bytes within the buffer?","required":true}]},
        "presentation":{"size":"S","nodes":[{"kind":"article","id":"article-buffer","title":"Unapproved buffer obligations","entities":["BUFFER-001","BUFFER-002"]}]},
        "authority":{"owner_acceptance":"pending","implementation_admission":"pending"}
    });
    let base_bytes = serde_json::to_vec_pretty(&base).unwrap();
    fs::write(root.join("base.json"), &base_bytes).unwrap();
    let mut answers = json!({"expected_base":caac::candidate::digest(&base_bytes),"answers":[{
        "action":"propose-decision","answer_id":"answer-footprint","decision_id":"decision-block-footprint",
        "value":{"text":"Case FOOTPRINT-001: input is a valid block offset with sentinel bytes before and after it. Expected observable: adjacent bytes remain unchanged. Disposition: additional-proposal-not-adopted, not entailed by global bounds or zero padding. Execution: NOT RUN. Semantic review and adoption remain unresolved.","entity_refs":["BUFFER-001","BUFFER-002"]},
        "source":"Synthetic reviewer-identified additional expectation; not an entailment check or owner approval."
    }]});
    let write_answers = |v: &Value| {
        fs::write(
            root.join("answers.json"),
            serde_json::to_vec_pretty(v).unwrap(),
        )
        .unwrap();
    };
    write_answers(&answers);
    let answers_bytes = fs::read(root.join("answers.json")).unwrap();
    let apply = |destination: &str| {
        run(
            root,
            &[
                "candidate",
                "apply",
                "--base",
                "base.json",
                "--answers",
                "answers.json",
                "--output",
                destination,
            ],
        )
    };
    ok(apply("proposed"));
    ok(stage(root, "verify", "proposed", &[]));
    let candidate = read(root, "proposed/candidate.json");
    assert_eq!(
        candidate["semantic"]["entities"],
        base["semantic"]["entities"]
    );
    assert_eq!(candidate["authority"], base["authority"]);
    let decision = &candidate["semantic"]["unresolved"][0];
    assert_eq!(decision["required"], true);
    assert_eq!(
        decision["question"],
        base["semantic"]["unresolved"][0]["question"]
    );
    assert_eq!(decision["proposal"], answers["answers"][0]["value"]);
    let report: Value = serde_json::from_slice(
        &ok(stage(
            root,
            "verify",
            "proposed",
            &["--trace", "decision-block-footprint"],
        ))
        .stdout,
    )
    .unwrap();
    assert_eq!(report["selected"]["body"], *decision);
    assert_eq!(report["unresolved_decisions"][0]["status"], "unresolved");
    assert_eq!(report["evidence_scope"]["semantic_review"], "not-supplied");
    assert_eq!(
        report["evidence_scope"]["source_behavior_evidence"],
        "not-run"
    );
    assert_eq!(report["evidence_scope"]["owner_acceptance"], "pending");
    let outgoing = report["direct_outgoing"].as_array().unwrap();
    assert_eq!(outgoing.len(), 2);
    for (edge, id) in outgoing.iter().zip(["BUFFER-001", "BUFFER-002"]) {
        assert_eq!(edge["target"]["id"], id);
        assert_eq!(
            edge["target"]["package_revision"],
            report["source"]["package_revision"]
        );
    }
    assert!(
        fs::read_to_string(root.join("proposed/review.md"))
            .unwrap()
            .contains("additional-proposal-not-adopted")
    );
    // Prose disposition is an authored review record, NOT a parsed assertion of truth.
    // Even an unjustified stronger assertion is format-valid: a meaning reviewer is necessary.
    answers["answers"][0]["value"]["text"] =
        json!("Adjacent bytes must remain unchanged; asserted to follow from buffer bounds.");
    write_answers(&answers);
    ok(apply("format-valid-not-entailed"));
    assert_eq!(
        read(root, "format-valid-not-entailed/candidate.json")["semantic"]["entities"],
        base["semantic"]["entities"]
    );
    answers["answers"][0]["value"]["entity_refs"] = json!(["missing-clause"]);
    write_answers(&answers);
    rejected(root, apply("invalid-reference"), "invalid-reference");
    fs::write(root.join("answers.json"), &answers_bytes).unwrap();
    let mut forged = base.clone();
    forged["authority"]["owner_acceptance"] = json!("accepted");
    let forged_bytes = serde_json::to_vec_pretty(&forged).unwrap();
    fs::write(root.join("base.json"), &forged_bytes).unwrap();
    let mut forged_answers: Value = serde_json::from_slice(&answers_bytes).unwrap();
    forged_answers["expected_base"] = json!(caac::candidate::digest(&forged_bytes));
    write_answers(&forged_answers);
    rejected(root, apply("forged-acceptance"), "forged-acceptance");
    fs::write(root.join("base.json"), &base_bytes).unwrap();
    rejected(root, apply("stale-base"), "stale-base");
    fs::write(root.join("answers.json"), &answers_bytes).unwrap();
    assert_eq!(fs::read(root.join("base.json")).unwrap(), base_bytes);
    assert_eq!(fs::read(root.join("answers.json")).unwrap(), answers_bytes);
}
