use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, Output, Stdio},
};
fn setup(d: &Path) -> (Value, Vec<Value>) {
    let b: Value = serde_json::from_str(include_str!("../examples/candidate/draft.json")).unwrap();
    fs::write(d.join("base.json"), serde_json::to_vec_pretty(&b).unwrap()).unwrap();
    let a: Value =
        serde_json::from_str(include_str!("../examples/candidate/answers.json")).unwrap();
    (
        b,
        a["answers"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|x| x.get("value").cloned())
            .collect(),
    )
}
fn command(d: &Path, resume: bool) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_caac"));
    c.args(["candidate", "interview", "--base"])
        .arg(d.join("base.json"))
        .arg("--answers")
        .arg(d.join("journal.json"))
        .arg("--output")
        .arg(d.join("output"));
    if resume {
        c.arg("--resume");
    }
    c
}
fn run(d: &Path, resume: bool, input: &str) -> Output {
    let mut child = command(d, resume)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    child.wait_with_output().unwrap()
}
fn lines(entities: &[Value]) -> String {
    entities
        .iter()
        .map(|e| format!("{}\n", serde_json::to_string(e).unwrap()))
        .collect()
}
fn read(d: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(d.join(name)).unwrap()).unwrap()
}
#[test]
fn typed_non_auto_interview_generates_candidate_with_pending_proposal() {
    let d = tempfile::tempdir().unwrap();
    let (b, es) = setup(d.path());
    let input = lines(&es)
        + "{\"text\":\"Owner should review the echo scope\",\"entity_refs\":[\"operation-echo\"]}\n";
    let out = run(d.path(), false, &input);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let candidate = read(d.path(), "output/candidate.json");
    assert_eq!(candidate["semantic"]["entities"], json!(es));
    let decision = &candidate["semantic"]["unresolved"][0];
    assert_eq!(
        decision["question"],
        b["semantic"]["unresolved"][0]["question"]
    );
    assert_eq!(decision["required"], true);
    assert!(decision["proposal"]["text"].is_string());
    let manifest = read(d.path(), "output/provenance.json");
    assert_eq!(manifest["required_unresolved"], 1);
    assert_eq!(manifest["owner_acceptance"], "pending");
    assert_eq!(manifest["contract_complete"], false);
}
#[test]
fn eof_and_resume_preserve_prefix_and_require_explicit_reuse() {
    let d = tempfile::tempdir().unwrap();
    let (_, es) = setup(d.path());
    assert!(!run(d.path(), false, &lines(&es[..2])).status.success());
    assert!(!d.path().join("output").exists());
    let saved = fs::read(d.path().join("journal.json")).unwrap();
    assert_eq!(
        read(d.path(), "journal.json")["answers"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(!run(d.path(), true, "cancel\n").status.success());
    assert_eq!(fs::read(d.path().join("journal.json")).unwrap(), saved);
    let out = run(
        d.path(),
        true,
        &("reuse\n".to_owned() + &lines(&es[2..]) + "skip\n"),
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        read(d.path(), "output/candidate.json")["semantic"]["unresolved"][0]
            .get("proposal")
            .is_none()
    );
}
#[test]
fn invalid_entity_or_skip_does_not_advance_the_journal() {
    for bad in [
        "skip\n",
        "{}\n",
        "{\"kind\":\"definition\",\"id\":\"wrong\",\"text\":\"x\"}\n",
        "{\"kind\":\"definition\",\"id\":\"term-zero\",\"text\":5}\n",
    ] {
        let d = tempfile::tempdir().unwrap();
        setup(d.path());
        assert!(!run(d.path(), false, bad).status.success());
        assert_eq!(read(d.path(), "journal.json")["answers"], json!([]));
        assert!(!d.path().join("output").exists());
    }
}
#[test]
fn stale_base_and_forged_journal_fields_cannot_resume() {
    for attack in 0..4 {
        let d = tempfile::tempdir().unwrap();
        let (_, es) = setup(d.path());
        assert!(!run(d.path(), false, &lines(&es[..1])).status.success());
        let mut j = read(d.path(), "journal.json");
        match attack {
            0 => {
                let mut b = fs::read(d.path().join("base.json")).unwrap();
                b.push(b' ');
                fs::write(d.path().join("base.json"), b).unwrap();
            }
            1 => j["format"] = json!("foreign"),
            2 => j["answers"][0]["target"]["id"] = json!("wrong"),
            _ => j["owner_acceptance"] = json!("accepted"),
        }
        fs::write(
            d.path().join("journal.json"),
            serde_json::to_vec_pretty(&j).unwrap(),
        )
        .unwrap();
        let saved = fs::read(d.path().join("journal.json")).unwrap();
        let out = run(d.path(), true, "reuse\n");
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
        assert_eq!(fs::read(d.path().join("journal.json")).unwrap(), saved);
        assert!(!d.path().join("output").exists());
    }
}
#[test]
fn final_failure_retains_all_collected_answers() {
    let d = tempfile::tempdir().unwrap();
    let (_, es) = setup(d.path());
    let input = lines(&es) + "{\"text\":\"Proposed scope\",\"entity_refs\":[\"absent\"]}\n";
    assert!(!run(d.path(), false, &input).status.success());
    assert_eq!(
        read(d.path(), "journal.json")["answers"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert!(!d.path().join("output").exists());
}
#[test]
fn output_created_during_interview_is_preserved_with_collected_journal() {
    let d = tempfile::tempdir().unwrap();
    let (_, es) = setup(d.path());
    let mut child = command(d.path(), false)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.starts_with("Question 1:"));
    fs::create_dir(d.path().join("output")).unwrap();
    fs::write(d.path().join("output/keep"), "keep").unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all((lines(&es) + "skip\n").as_bytes())
        .unwrap();
    assert!(!child.wait().unwrap().success());
    assert_eq!(
        read(d.path(), "journal.json")["answers"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert_eq!(
        fs::read_to_string(d.path().join("output/keep")).unwrap(),
        "keep"
    );
}
#[test]
fn existing_output_is_rejected_before_questions_or_journal() {
    let d = tempfile::tempdir().unwrap();
    setup(d.path());
    fs::create_dir(d.path().join("output")).unwrap();
    let out = run(d.path(), false, "");
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(!d.path().join("journal.json").exists());
}
#[test]
fn path_overlap_and_bad_profile_fail_before_journal_or_prompt() {
    let d = tempfile::tempdir().unwrap();
    let (mut b, _) = setup(d.path());
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["candidate", "interview", "--base"])
        .arg(d.path().join("base.json"))
        .arg("--answers")
        .arg(d.path().join("new/journal.json"))
        .arg("--output")
        .arg(d.path().join("new"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!d.path().join("new").exists());
    b["presentation"]["size"] = json!("XL");
    fs::write(d.path().join("base.json"), serde_json::to_vec(&b).unwrap()).unwrap();
    let out = run(d.path(), false, "");
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(!d.path().join("journal.json").exists());
}
#[test]
fn concurrent_base_change_does_not_save_answer_or_publish() {
    let d = tempfile::tempdir().unwrap();
    let (_, es) = setup(d.path());
    let mut child = command(d.path(), false)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.starts_with("Question 1:"));
    let mut base = fs::read(d.path().join("base.json")).unwrap();
    base.push(b' ');
    fs::write(d.path().join("base.json"), &base).unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(lines(&es[..1]).as_bytes())
        .unwrap();
    assert!(!child.wait().unwrap().success());
    assert_eq!(read(d.path(), "journal.json")["answers"], json!([]));
    assert_eq!(fs::read(d.path().join("base.json")).unwrap(), base);
    assert!(!d.path().join("output").exists());
}
#[test]
fn overlong_input_and_terminal_error_injection_are_bounded() {
    let d = tempfile::tempdir().unwrap();
    setup(d.path());
    assert!(
        !run(d.path(), false, &("x".repeat(65537) + "\n"))
            .status
            .success()
    );
    assert_eq!(read(d.path(), "journal.json")["answers"], json!([]));
    let d = tempfile::tempdir().unwrap();
    let (_, mut es) = setup(d.path());
    es[3]["target"] = json!("\u{1b}[2Jforged");
    for e in &mut es {
        if e["kind"] == "case" {
            e["exception_refs"] = json!([]);
        }
    }
    let out = run(d.path(), false, &(lines(&es) + "skip\n"));
    assert!(!out.status.success());
    assert!(!out.stderr.contains(&27));
    assert!(String::from_utf8_lossy(&out.stderr).contains("\\u001b"));
    assert!(!d.path().join("output").exists());
}

#[test]
fn changed_journal_after_reuse_is_not_replayed_or_overwritten() {
    let d = tempfile::tempdir().unwrap();
    let (_, es) = setup(d.path());
    assert!(!run(d.path(), false, &lines(&es[..1])).status.success());
    let mut child = command(d.path(), true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    reader.read_line(&mut line).unwrap();
    let mut input = child.stdin.take().unwrap();
    input.write_all(b"reuse\n").unwrap();
    input.flush().unwrap();
    line.clear();
    reader.read_line(&mut line).unwrap();
    assert!(line.starts_with("Question 2:"));
    let foreign = b"foreign journal edit";
    fs::write(d.path().join("journal.json"), foreign).unwrap();
    input.write_all(lines(&es[1..2]).as_bytes()).unwrap();
    drop(input);
    assert!(!child.wait().unwrap().success());
    assert_eq!(fs::read(d.path().join("journal.json")).unwrap(), foreign);
    assert!(!d.path().join("output").exists());
}

#[test]
fn journal_change_before_reuse_is_rejected_before_next_question() {
    use std::io::Read;
    let d = tempfile::tempdir().unwrap();
    let (_, es) = setup(d.path());
    assert!(!run(d.path(), false, &lines(&es[..1])).status.success());
    let mut child = command(d.path(), true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    reader.read_line(&mut line).unwrap();
    fs::write(d.path().join("journal.json"), b"foreign edit").unwrap();
    child.stdin.take().unwrap().write_all(b"reuse\n").unwrap();
    let mut remaining = String::new();
    reader.read_to_string(&mut remaining).unwrap();
    assert!(!remaining.contains("Question"));
    assert!(!child.wait().unwrap().success());
    assert_eq!(
        fs::read(d.path().join("journal.json")).unwrap(),
        b"foreign edit"
    );
    assert!(!d.path().join("output").exists());
}
