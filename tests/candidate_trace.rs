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
    d
}
fn trace(d: &Path, id: &str) -> Output {
    run(
        d,
        &[
            "candidate",
            "verify",
            "--candidate",
            "source/candidate.json",
            "--provenance",
            "source/provenance.json",
            "--review",
            "source/review.md",
            "--trace",
            id,
        ],
    )
}
fn report(d: &Path, id: &str) -> Value {
    let o = trace(d, id);
    assert!(o.status.success(), "{:?}", o);
    assert!(o.stderr.is_empty());
    assert_eq!(o.stdout.iter().filter(|b| **b == b'\n').count(), 1);
    serde_json::from_slice(&o.stdout).unwrap()
}
fn apply_snapshot(d: &Path, bytes: &[u8], destination: &str) -> Output {
    fs::write(d.join("custom-base"), bytes).unwrap();
    write(
        d,
        "empty-answers",
        &json!({"expected_base":caac::candidate::digest(bytes),"answers":[]}),
    );
    run(
        d,
        &[
            "candidate",
            "apply",
            "--base",
            "custom-base",
            "--answers",
            "empty-answers",
            "--output",
            destination,
        ],
    )
}
fn regenerate(d: &Path, c: &Value) {
    let output = apply_snapshot(d, &serde_json::to_vec(c).unwrap(), "custom-source");
    assert!(output.status.success(), "{output:?}");
    fs::rename(d.join("source"), d.join("previous-source")).unwrap();
    fs::rename(d.join("custom-source"), d.join("source")).unwrap();
}

const SHARED_RETRY_DRAFT: &[u8] = include_bytes!("../examples/candidate/shared-retry-draft.json");

#[test]
fn shared_retry_rule_has_one_address_and_both_modules_can_trace_to_it() {
    let d = tempfile::tempdir().unwrap();
    let output = apply_snapshot(d.path(), SHARED_RETRY_DRAFT, "source");
    assert_eq!(output.status.code(), Some(0), "{output:?}");

    let rule = report(d.path(), "rule-retry-limit");
    assert_eq!(rule["selected"]["mapping"].as_array().unwrap().len(), 1);
    assert_eq!(
        rule["selected"]["mapping"][0]["location"]["node_id"],
        "article-policy"
    );
    let address = &rule["selected"]["reference"];
    for operation in ["operation-intake", "operation-replay"] {
        let trace = report(d.path(), operation);
        assert_eq!(trace["direct_outgoing"][0]["field"], "clause_refs");
        assert_eq!(&trace["direct_outgoing"][0]["target"], address);
    }

    let definition = report(d.path(), "term-priority");
    let exception = report(d.path(), "except-priority");
    let replay_case = report(d.path(), "test-replay-priority");
    for (trace, article) in [
        (&definition, "article-policy"),
        (&exception, "article-policy"),
        (&replay_case, "article-replay"),
    ] {
        let location = &trace["selected"]["mapping"][0];
        assert_eq!(location["file"], "review.md");
        assert_eq!(location["location"]["node_id"], article);
    }

    assert_eq!(
        rule["direct_outgoing"][0]["target"],
        definition["selected"]["reference"]
    );
    assert_eq!(&exception["direct_outgoing"][0]["target"], address);
    assert!(
        replay_case["direct_outgoing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|edge| {
                edge["field"] == "exception_refs"
                    && edge["target"] == exception["selected"]["reference"]
            })
    );

    let cases = rule["reachable_declared_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    assert_eq!(cases[0]["body"]["input"], json!({"priority": false}));
    assert_eq!(cases[0]["body"]["expected"], 3);
    assert_eq!(cases[1]["body"]["input"], json!({"priority": true}));
    assert_eq!(cases[1]["body"]["expected"], 5);
    assert_eq!(
        cases[1]["body"]["exception_refs"],
        json!(["except-priority"])
    );
    assert_eq!(
        rule["evidence_scope"]["source_behavior_evidence"],
        "not-run"
    );
    assert_eq!(rule["unresolved_decisions"][0]["body"]["required"], true);
}

#[test]
fn shared_retry_draft_rejects_a_broken_replay_reference_before_publication() {
    let d = tempfile::tempdir().unwrap();
    let mut candidate: Value = serde_json::from_slice(SHARED_RETRY_DRAFT).unwrap();
    let replay = candidate["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entity| entity["id"] == "operation-replay")
        .unwrap();
    replay["clause_refs"][0] = json!("rule-missing");
    let output = apply_snapshot(d.path(), &serde_json::to_vec(&candidate).unwrap(), "broken");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Reference rule-missing must resolve to clause"),
        "{output:?}"
    );
    assert!(output.stdout.is_empty());
    assert!(!d.path().join("broken").exists());
}

#[test]
fn trace_exposes_exact_bidirectional_declared_links_without_readiness() {
    let d = setup();
    let before = fs::read(d.path().join("source/candidate.json")).unwrap();
    let r = report(d.path(), "rule-echo");
    let p = read(d.path(), "source/provenance.json");
    assert_eq!(r["format"], "caac-candidate-trace-v1");
    assert_eq!(
        r["selected"]["reference"]["package_revision"],
        p["revision_manifest"]["package_revision"]
    );
    assert_eq!(r["selected"]["body"]["kind"], "clause");
    assert_eq!(r["direct_outgoing"][0]["target"]["id"], "term-zero");
    assert!(
        r["direct_incoming"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["source"]["id"] == "except-zero")
    );
    assert_eq!(r["reachable_declared_cases"].as_array().unwrap().len(), 2);
    assert_eq!(r["evidence_scope"]["source_behavior_evidence"], "not-run");
    assert_eq!(
        r["evidence_scope"]["execution_evidence"],
        "not-recorded-in-supplied-artifacts"
    );
    assert_eq!(r["evidence_scope"]["completion"], "not-assessed");
    assert_eq!(
        fs::read(d.path().join("source/candidate.json")).unwrap(),
        before
    );
    assert!(r.get("ready").is_none());
}
#[test]
fn reverse_reachability_and_selected_case_scope_are_not_undirected() {
    let d = setup();
    let r = report(d.path(), "term-zero");
    assert!(r["forward_reachable"].as_array().unwrap().is_empty());
    assert!(
        r["reverse_reachable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == "operation-echo")
    );
    let r = report(d.path(), "case-zero");
    assert_eq!(r["reachable_declared_cases"].as_array().unwrap().len(), 1);
    assert_eq!(
        r["reachable_declared_cases"][0]["reference"]["id"],
        "case-zero"
    );
    assert!(
        !r["forward_reachable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == "case-zero")
    );
}
#[test]
fn package_decisions_include_unlinked_and_proposed_unresolved_records() {
    let d = setup();
    let r = report(d.path(), "decision-scope");
    assert!(r["direct_outgoing"].as_array().unwrap().is_empty());
    assert_eq!(
        r["selected"]["mapping"][0]["location"]["kind"],
        "unresolved-decision"
    );
    let mut c = read(d.path(), "source/candidate.json");
    c["semantic"]["unresolved"][0]["proposal"] =
        json!({"text":"Still pending","entity_refs":["rule-echo"]});
    c["semantic"]["unresolved"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"unlinked","question":"Still required?","required":true}));
    regenerate(d.path(), &c);
    let r = report(d.path(), "rule-echo");
    assert_eq!(r["unresolved_decisions"].as_array().unwrap().len(), 2);
    assert!(
        r["unresolved_decisions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["status"] == "unresolved")
    );
    assert!(
        r["direct_incoming"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["source"]["id"] == "decision-scope")
    );
}
#[test]
fn empty_cases_and_decisions_do_not_create_completion() {
    let d = setup();
    let mut c = read(d.path(), "source/candidate.json");
    c["semantic"]["unresolved"] = json!([]);
    c["semantic"]["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|e| e["kind"] != "case");
    c["presentation"]["nodes"][0]["entities"]
        .as_array_mut()
        .unwrap()
        .retain(|id| !id.as_str().unwrap().starts_with("case-"));
    regenerate(d.path(), &c);
    let r = report(d.path(), "rule-echo");
    assert_eq!(r["unresolved_decisions"], json!([]));
    assert_eq!(r["reachable_declared_cases"], json!([]));
    assert_eq!(r["evidence_scope"]["completion"], "not-assessed");
    assert_eq!(
        r["evidence_scope"]["implementation_links"],
        "not-represented"
    );
}
#[test]
fn tampering_and_unknown_ids_have_no_partial_json_stdout() {
    for mode in 0..4 {
        let d = setup();
        match mode {
            0 => {
                let mut p = read(d.path(), "source/provenance.json");
                p["owner_acceptance"] = json!("accepted");
                write(d.path(), "source/provenance.json", &p);
            }
            1 => {
                fs::write(d.path().join("source/review.md"), "forged").unwrap();
            }
            _ => {}
        }
        let o = trace(
            d.path(),
            if mode == 2 {
                "missing"
            } else if mode == 3 {
                "bad\u{1b}[2J"
            } else {
                "rule-echo"
            },
        );
        assert!(!o.status.success());
        assert!(o.stdout.is_empty());
        assert!(!o.stderr.contains(&0x1b));
    }
}
#[test]
fn terminal_safe_json_preserves_data_and_determinism() {
    let d = setup();
    let mut c = read(d.path(), "source/candidate.json");
    c["semantic"]["entities"][0]["text"] = json!("한국어 \u{202e}owner accepted");
    regenerate(d.path(), &c);
    let a = trace(d.path(), "term-zero");
    let b = trace(d.path(), "term-zero");
    assert!(a.status.success());
    assert_eq!(a.stdout, b.stdout);
    assert!(a.stdout.is_ascii());
    let r: Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(
        r["selected"]["body"]["text"],
        c["semantic"]["entities"][0]["text"]
    );
}
#[test]
fn v5_mapping_moves_without_claiming_evidence_reuse() {
    let d = setup();
    let c = read(d.path(), "source/candidate.json");
    let p = read(d.path(), "source/provenance.json");
    write(
        d.path(),
        "presentation-request",
        &json!({"format":"caac-presentation-request-v1","expected_candidate_sha256":p["candidate_sha256"],"expected_package_revision":p["revision_manifest"]["package_revision"],"presentation":{"size":"M","nodes":[{"kind":"chapter","id":"chapter-one","title":"Moved","articles":c["presentation"]["nodes"]}]}}),
    );
    assert!(
        run(
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
                "presentation-request",
                "--output",
                "custom-source"
            ]
        )
        .status
        .success()
    );
    fs::rename(d.path().join("source"), d.path().join("previous-source")).unwrap();
    fs::rename(d.path().join("custom-source"), d.path().join("source")).unwrap();
    let r = report(d.path(), "rule-echo");
    assert_eq!(
        r["source"]["provenance_format"],
        "caac-candidate-provenance-v5"
    );
    assert_eq!(
        r["selected"]["mapping"][0]["location"]["chapter_id"],
        "chapter-one"
    );
    assert_eq!(
        r["evidence_scope"]["execution_evidence"],
        "not-recorded-in-supplied-artifacts"
    );
}

#[test]
fn v4_trace_reports_fresh_policy_without_changing_plain_verify() {
    let d = setup();
    let source = fs::read_to_string(d.path().join("source/review.md")).unwrap();
    fs::write(
        d.path().join("edited"),
        source.replace(
            "Echo returns the supplied integer.",
            "A pending changed clause.",
        ),
    )
    .unwrap();
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
    assert!(
        run(
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
                "--proposal",
                "proposal/proposal.json",
                "--output",
                "custom-source"
            ]
        )
        .status
        .success()
    );
    fs::rename(d.path().join("source"), d.path().join("previous-source")).unwrap();
    fs::rename(d.path().join("custom-source"), d.path().join("source")).unwrap();
    let r = report(d.path(), "rule-echo");
    assert_eq!(
        r["source"]["provenance_format"],
        "caac-candidate-provenance-v4"
    );
    assert_eq!(
        r["evidence_scope"]["source_transition_evidence_policy"],
        "fresh-review-required-for-whole-package"
    );
    let plain = run(
        d.path(),
        &[
            "candidate",
            "verify",
            "--candidate",
            "source/candidate.json",
            "--provenance",
            "source/provenance.json",
            "--review",
            "source/review.md",
        ],
    );
    assert!(plain.status.success());
    assert!(
        String::from_utf8(plain.stdout)
            .unwrap()
            .starts_with("Verified unsigned candidate")
    );
}
