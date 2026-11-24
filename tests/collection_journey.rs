//! Explicit synthetic authoring handoff, not automated reasoning or conformance.
use caac::candidate::digest;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn run(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
fn read(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}
#[test]
fn collected_evidence_supports_explicit_pending_draft_without_inferred_authority() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let source = b"pub fn echo(value: i32) -> i32 { value }\n";
    let contract =
        b"# Echo module draft\r\n\r\nECHO-001 | Echo must return zero for negative inputs.\r\n";
    fs::write(root.join("lib.rs"), source).unwrap();
    fs::write(root.join("module.md"), contract).unwrap();
    run(
        root,
        &[
            "plugin",
            "intake",
            "--title",
            "Synthetic echo",
            "--source",
            "lib.rs",
        ],
    );
    let baseline_bytes = fs::read(root.join("intake/baseline.json")).unwrap();
    let scaffold_bytes = fs::read(root.join("intake/CONTRACT.md")).unwrap();
    run(
        root,
        &[
            "collect",
            "--contract",
            "module.md",
            "--baseline",
            "intake/baseline.json",
            "--output",
            "collection.json",
        ],
    );
    let bundle = read(root, "collection.json");
    assert_eq!(
        bundle["documents"][0]["text"],
        std::str::from_utf8(contract).unwrap()
    );
    assert_eq!(bundle["documents"][0]["sha256"], digest(contract));
    assert_eq!(bundle["root"]["finding"], "missing-root-contract");
    assert_eq!(bundle["comparison"], "not-performed");
    assert_eq!(bundle["coverage"], "unverified");
    assert_eq!(bundle["mismatch"], "unverified");
    assert_eq!(bundle["baseline"]["sha256"], digest(&baseline_bytes));
    assert_eq!(
        bundle["baseline"]["evidence"]["files"][0]["sha256"],
        digest(source)
    );
    assert_eq!(bundle["baseline"]["evidence"]["execution"], "not-run");

    // These questions and proposals are explicitly authored fixture inputs.
    // No collection operation or LLM inferred them, and the source string is not a verified binding.
    let questions = json!([
        {"id":"missing-root","question":"What project purpose and root obligations apply? No root was selected.","required":true},
        {"id":"suspected-mismatch","question":"The draft asks for zero on negatives; the source appears to return its input. Which behavior is intended? This suspicion is unverified.","required":true},
        {"id":"unverified-execution","question":"What execution and semantic review evidence is required? None was supplied.","required":true}
    ]);
    let base = json!({"format":"caac-candidate-v1","semantic":{"namespace":"synthetic-collection","entities":[],"unresolved":questions},"presentation":{"size":"S","nodes":[{"kind":"article","id":"article-echo","title":"Proposed echo requirement","entities":["rule-echo"]}]},"authority":{"owner_acceptance":"pending","implementation_admission":"pending"}});
    let base_bytes = serde_json::to_vec_pretty(&base).unwrap();
    fs::write(root.join("base.json"), &base_bytes).unwrap();
    let bundle_bytes = fs::read(root.join("collection.json")).unwrap();
    let explanation = format!(
        "Explicit synthetic proposal from module.md ECHO-001, raw sha256 {}; collection sha256 {}. Role and inferred mismatch remain unverified; this string is not an authenticated provenance link.",
        digest(contract),
        digest(&bundle_bytes)
    );
    let answers = json!({"expected_base":digest(&base_bytes),"answers":[{"action":"propose-entity","answer_id":"answer-rule","target":{"kind":"clause","id":"rule-echo"},"value":{"kind":"clause","id":"rule-echo","actor":"Echo","modality":"obligation","text":"Echo returns zero for negative inputs.","definition_refs":[]},"source":explanation}]});
    fs::write(
        root.join("answers.json"),
        serde_json::to_vec_pretty(&answers).unwrap(),
    )
    .unwrap();
    let inputs: Vec<_> = [
        "lib.rs",
        "module.md",
        "intake/baseline.json",
        "intake/CONTRACT.md",
        "collection.json",
        "base.json",
        "answers.json",
    ]
    .iter()
    .map(|p| (*p, fs::read(root.join(p)).unwrap()))
    .collect();
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
            "candidate",
        ],
    );
    let trace: Value = serde_json::from_slice(&run(
        root,
        &[
            "candidate",
            "verify",
            "--candidate",
            "candidate/candidate.json",
            "--provenance",
            "candidate/provenance.json",
            "--review",
            "candidate/review.md",
            "--trace",
            "rule-echo",
        ],
    ))
    .unwrap();
    let candidate = read(root, "candidate/candidate.json");
    assert_eq!(candidate["semantic"]["unresolved"], questions);
    assert_eq!(candidate["authority"]["owner_acceptance"], "pending");
    assert_eq!(
        candidate["authority"]["implementation_admission"],
        "pending"
    );
    assert_eq!(trace["evidence_scope"]["completion"], "not-assessed");
    assert_eq!(
        trace["evidence_scope"]["execution_evidence"],
        "not-recorded-in-supplied-artifacts"
    );
    assert_eq!(
        trace["evidence_scope"]["implementation_links"],
        "not-represented"
    );
    assert_eq!(fs::read(root.join("lib.rs")).unwrap(), source);
    assert_eq!(fs::read(root.join("module.md")).unwrap(), contract);
    assert_eq!(
        fs::read(root.join("intake/baseline.json")).unwrap(),
        baseline_bytes
    );
    assert_eq!(
        fs::read(root.join("intake/CONTRACT.md")).unwrap(),
        scaffold_bytes
    );
    for (path, bytes) in inputs {
        assert_eq!(fs::read(root.join(path)).unwrap(), bytes, "{path}");
    }
}
