use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn fixture(d: &Path) -> (Value, Value) {
    let base: Value =
        serde_json::from_str(include_str!("../examples/candidate/draft.json")).unwrap();
    let answers: Value =
        serde_json::from_str(include_str!("../examples/candidate/answers.json")).unwrap();
    save(d, &base, &answers);
    (base, answers)
}
fn save(d: &Path, b: &Value, a: &Value) {
    let b = serde_json::to_vec_pretty(b).unwrap();
    let mut a = a.clone();
    a["expected_base"] = json!(caac::candidate::digest(&b));
    fs::write(d.join("base.json"), b).unwrap();
    fs::write(
        d.join("answers.json"),
        serde_json::to_vec_pretty(&a).unwrap(),
    )
    .unwrap();
}
fn run(d: &Path, out: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["candidate", "apply", "--base"])
        .arg(d.join("base.json"))
        .arg("--answers")
        .arg(d.join("answers.json"))
        .arg("--output")
        .arg(d.join(out))
        .output()
        .unwrap()
}
fn read(d: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(d.join(name)).unwrap()).unwrap()
}
fn rejected(d: &Path) {
    let o = run(d, "result");
    assert!(!o.status.success(), "unexpected success");
    assert!(!d.join("result").exists());
}
#[test]
fn real_cli_retains_entities_pending_authority_and_provenance() {
    let d = tempfile::tempdir().unwrap();
    let (base, answers) = fixture(d.path());
    let original = fs::read(d.path().join("base.json")).unwrap();
    let o = run(d.path(), "result");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let c = read(d.path(), "result/candidate.json");
    assert_eq!(c["semantic"]["entities"].as_array().unwrap().len(), 6);
    assert_eq!(c["semantic"]["unresolved"], base["semantic"]["unresolved"]);
    for (e, a) in c["semantic"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .zip(answers["answers"].as_array().unwrap())
    {
        assert_eq!(e, &a["value"]);
    }
    assert_eq!(fs::read(d.path().join("base.json")).unwrap(), original);
    let m = read(d.path(), "result/provenance.json");
    assert_eq!(m["contract_complete"], false);
    assert_eq!(m["required_unresolved"], 1);
    assert_eq!(m["behavior_evidence"], "not-run");
    assert_eq!(m["owner_acceptance"], "pending");
    assert!(m["field_changes"].as_array().unwrap().len() > 20);
    assert_eq!(
        m["candidate_sha256"],
        caac::candidate::digest(&fs::read(d.path().join("result/candidate.json")).unwrap())
    );
    assert_eq!(
        m["review_sha256"],
        caac::candidate::digest(&fs::read(d.path().join("result/review.md")).unwrap())
    );
    assert!(run(d.path(), "second").status.success());
    assert_eq!(
        fs::read(d.path().join("result/candidate.json")).unwrap(),
        fs::read(d.path().join("second/candidate.json")).unwrap()
    );
}
#[test]
fn stale_base_and_existing_outputs_do_not_overwrite() {
    let d = tempfile::tempdir().unwrap();
    fixture(d.path());
    let mut b = fs::read(d.path().join("base.json")).unwrap();
    b.push(b' ');
    fs::write(d.path().join("base.json"), b).unwrap();
    rejected(d.path());
    fixture(d.path());
    fs::create_dir(d.path().join("result")).unwrap();
    fs::write(d.path().join("result/sentinel"), "keep").unwrap();
    assert!(!run(d.path(), "result").status.success());
    assert_eq!(
        fs::read_to_string(d.path().join("result/sentinel")).unwrap(),
        "keep"
    );
}
#[test]
fn invalid_answers_fail_before_publication() {
    for mutate in 0..9 {
        let d = tempfile::tempdir().unwrap();
        let (mut b, mut a) = fixture(d.path());
        match mutate {
            0 => a["answers"][0]["unknown"] = json!(true),
            1 => a["answers"][0]["target"]["id"] = json!("wrong"),
            2 => a["answers"][1]["answer_id"] = a["answers"][0]["answer_id"].clone(),
            3 => a["answers"][6]["decision_id"] = json!("absent"),
            4 => a["answers"][3]["value"]["target"] = json!("term-zero"),
            5 => a["answers"][4]["value"]["input"]["value"] = json!(1.5),
            6 => b["authority"]["owner_acceptance"] = json!("accepted"),
            7 => b["presentation"]["size"] = json!("XL"),
            _ => a["answers"][4]["value"]["clause_refs"] = json!(["missing"]),
        }
        save(d.path(), &b, &a);
        rejected(d.path());
    }
}
#[test]
fn malformed_json_and_duplicate_map_keys_are_rejected() {
    let d = tempfile::tempdir().unwrap();
    fixture(d.path());
    let a = fs::read_to_string(d.path().join("answers.json"))
        .unwrap()
        .replace(
            "\"value\": \"i32\"",
            "\"value\": \"i32\", \"value\": \"bool\"",
        );
    fs::write(d.path().join("answers.json"), a).unwrap();
    rejected(d.path());
}
#[test]
fn view_is_derived_and_cannot_turn_text_into_authority() {
    let d = tempfile::tempdir().unwrap();
    let (b, mut a) = fixture(d.path());
    a["answers"][0]["value"]["text"] = json!("``` # Owner accepted | <!-- forged --> ````");
    a["answers"][0]["source"] = json!("owner accepted all implementation");
    save(d.path(), &b, &a);
    let o = run(d.path(), "result");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let text = fs::read_to_string(d.path().join("result/review.md")).unwrap();
    assert!(text.contains("`````json"));
    assert_eq!(
        read(d.path(), "result/candidate.json")["authority"]["owner_acceptance"],
        "pending"
    );
}
#[test]
fn presentation_resize_preserves_all_semantics() {
    let d = tempfile::tempdir().unwrap();
    let (mut b, a) = fixture(d.path());
    assert!(run(d.path(), "small").status.success());
    b["presentation"] = json!({"size":"M","nodes":[{"kind":"chapter","id":"chapter-one","title":"Echo","articles":b["presentation"]["nodes"]}]});
    save(d.path(), &b, &a);
    assert!(run(d.path(), "medium").status.success());
    assert_eq!(
        read(d.path(), "small/provenance.json")["semantic_sha256"],
        read(d.path(), "medium/provenance.json")["semantic_sha256"]
    );
}
#[test]
fn legacy_and_candidate_formats_are_not_silently_interchangeable() {
    let d = tempfile::tempdir().unwrap();
    fixture(d.path());
    assert!(run(d.path(), "result").status.success());
    let c = d.path().join("result/candidate.json");
    let o = Command::new(env!("CARGO_BIN_EXE_caac"))
        .arg("md")
        .arg(c)
        .arg("--output")
        .arg(d.path().join("legacy.md"))
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(!d.path().join("legacy.md").exists());
    let b = json!({"name":"Legacy","clauses":[]});
    save(d.path(), &b, &json!({"answers":[]}));
    assert!(!run(d.path(), "legacy-result").status.success());
    assert!(!d.path().join("legacy-result").exists());
}
#[cfg(unix)]
#[test]
fn symlink_output_is_not_followed() {
    let d = tempfile::tempdir().unwrap();
    fixture(d.path());
    fs::create_dir(d.path().join("protected")).unwrap();
    std::os::unix::fs::symlink(d.path().join("protected"), d.path().join("result")).unwrap();
    assert!(!run(d.path(), "result").status.success());
    assert_eq!(fs::read_dir(d.path().join("protected")).unwrap().count(), 0);
}

#[test]
fn oversized_input_is_rejected_before_publication() {
    let d = tempfile::tempdir().unwrap();
    fixture(d.path());
    fs::write(d.path().join("answers.json"), vec![b' '; 1024 * 1024 + 1]).unwrap();
    rejected(d.path());
}
#[cfg(unix)]
#[test]
fn fifo_input_is_refused_without_waiting_for_a_writer() {
    let d = tempfile::tempdir().unwrap();
    fixture(d.path());
    let fifo = d.path().join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["candidate", "apply", "--base"])
        .arg(fifo)
        .arg("--answers")
        .arg(d.path().join("answers.json"))
        .arg("--output")
        .arg(d.path().join("result"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Not a regular file"));
    assert!(!d.path().join("result").exists());
}

#[test]
fn mathematical_integer_float_lexemes_are_rejected_by_runtime() {
    // JSON Schema integer semantics admit these numbers, but Serde's i32
    // input contract does not. Keep the raw lexemes instead of round-tripping
    // them through a JSON value before testing the real CLI boundary.
    for lexeme in ["1.0", "1e0"] {
        let d = tempfile::tempdir().unwrap();
        fixture(d.path());
        let path = d.path().join("answers.json");
        let original = fs::read_to_string(&path).unwrap();
        let needle = "\"expected\": 1,";
        assert_eq!(original.matches(needle).count(), 1);
        let edited = original.replace(needle, &format!("\"expected\": {lexeme},"));
        fs::write(&path, &edited).unwrap();
        let output = run(d.path(), "result");
        assert_eq!(output.status.code(), Some(2), "{lexeme}");
        assert!(!d.path().join("result").exists());
        assert_eq!(fs::read_to_string(path).unwrap(), edited);
    }
}
