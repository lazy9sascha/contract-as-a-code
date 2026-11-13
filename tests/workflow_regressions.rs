use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;

const BASE: &str = "RULE-001 | Service must return a value.\n\n## Contract cases\n\n| Case | Input | Expected | Clauses |\n| --- | --- | --- | --- |\n| one | input | value | RULE-001 |\n";

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_caac"))
}

fn state_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.caac-state.json", path.display()))
}

fn spawn(args: &[&str], path: &Path) -> Child {
    command()
        .args(args)
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn finish(mut child: Child) -> Output {
    drop(child.stdin.take());
    let end = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= end {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!("workflow hung: {}", String::from_utf8_lossy(&output.stdout));
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

fn answer(args: &[&str], path: &Path, text: &str) -> Output {
    let mut child = spawn(args, path);
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    finish(child)
}

fn wait_for(mut predicate: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(5);
    while !predicate() {
        assert!(Instant::now() < end, "timed out waiting for checkpoint");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn harness_rechecks_its_answer_and_reports_only_a_precheck() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, format!("{BASE}OPEN: Which value?\n")).unwrap();
    let output = answer(&["run", "--max-iterations", "2"], &path, "one\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("harness iteration 2"));
    assert!(stdout.contains("PRE-REVIEW CHECK: PASSED"));
    assert!(stdout.contains("owner acceptance remain pending"));
    assert!(!stdout.contains("COMPLETION GATE"));
}

#[test]
fn harness_no_progress_is_bounded_and_errors_propagate() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, format!("{BASE}OPEN: Which value?\n")).unwrap();
    let output = answer(&["run"], &path, "skip\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("NO PROGRESS"));
    fs::write(state_path(&path), "broken checkpoint").unwrap();
    let output = answer(&["run"], &path, "one\n");
    assert_eq!(output.status.code(), Some(2));
    assert!(fs::read_to_string(&path).unwrap().contains("OPEN:"));
}

#[test]
fn auto_saves_state_and_resume_can_answer_skipped_questions() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, "OPEN: Which value?\n").unwrap();
    let output = command()
        .args(["interview", "--auto"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let state: Value =
        serde_json::from_str(&fs::read_to_string(state_path(&path)).unwrap()).unwrap();
    assert_eq!(state["skipped"][0], "Which value?");
    let output = answer(&["resume"], &path, "one\n");
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), "DECISION: one\n");
}

#[test]
fn interrupted_interview_keeps_recorded_answer_and_resume_asks_only_remaining() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, "OPEN: First?\nOPEN: Second?\n").unwrap();
    let mut child = spawn(&["interview"], &path);
    child.stdin.as_mut().unwrap().write_all(b"one\n").unwrap();
    wait_for(|| fs::read_to_string(&path).unwrap().contains("DECISION: one"));
    child.kill().unwrap();
    child.wait().unwrap();
    let output = answer(&["resume"], &path, "two\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("First?"));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "DECISION: one\nDECISION: two\n"
    );
}

#[test]
fn pending_answer_replays_only_against_its_exact_base() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    let original = "OPEN: First?\n";
    let replacement = "DECISION: one\n";
    fs::write(&path, original).unwrap();
    let output = command()
        .args(["interview", "--auto"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let state_path = state_path(&path);
    let mut state: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["pending"] = serde_json::json!({
        "question_id": "fixture",
        "before_revision": hash(original),
        "after_revision": hash(replacement),
        "content": replacement,
    });
    fs::write(&state_path, serde_json::to_vec(&state).unwrap()).unwrap();
    let output = answer(&["resume"], &path, "");
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), replacement);
    fs::write(&state_path, serde_json::to_vec(&state).unwrap()).unwrap();
    fs::write(&path, "user edit\n").unwrap();
    let output = answer(&["resume"], &path, "");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&path).unwrap(), "user edit\n");
}

#[test]
fn concurrent_contract_edits_are_not_overwritten() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, "OPEN: First?\n").unwrap();
    let mut child = spawn(&["interview"], &path);
    // A checkpoint exists before preparation finishes. Wait for the question,
    // otherwise an early conflict can correctly exit before our stdin write.
    let stdout = child.stdout.take().unwrap();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok_and(|count| count > 0) {
            if line.contains("Decision 1 of 1: First?") {
                let _ = ready_tx.send(reader.into_inner());
                return;
            }
            line.clear();
        }
    });
    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(stdout) => child.stdout = Some(stdout),
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("interview did not reach its question: {error}");
        }
    }
    fs::write(&path, "user edit\n").unwrap();
    child.stdin.as_mut().unwrap().write_all(b"one\n").unwrap();
    let output = finish(child);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&path).unwrap(), "user edit\n");
}

#[test]
fn invalid_checkpoint_and_checkpoint_write_failure_do_not_change_contract() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, "OPEN: First?\n").unwrap();
    fs::create_dir(state_path(&path)).unwrap();
    let output = answer(&["interview"], &path, "one\n");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&path).unwrap(), "OPEN: First?\n");
}

#[test]
fn interview_excludes_examples_and_preserves_line_endings() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(
        &path,
        "<!--\r\nOPEN: Hidden?\r\n-->\r\n```\r\nOPEN: Example?\r\n```\r\nOPEN: Real?\r\n",
    )
    .unwrap();
    let output = answer(&["interview"], &path, "one\n");
    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "<!--\r\nOPEN: Hidden?\r\n-->\r\n```\r\nOPEN: Example?\r\n```\r\nDECISION: one\r\n"
    );
}

#[test]
fn eof_does_not_claim_answers_and_stale_resume_refuses_to_overwrite() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, "OPEN: First?\n").unwrap();
    let output = answer(&["interview"], &path, "");
    assert!(output.status.success());
    assert!(fs::read_to_string(&path).unwrap().starts_with("OPEN:"));
    fs::write(&path, "OPEN: New meaning?\n").unwrap();
    let output = answer(&["resume"], &path, "one\n");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&path).unwrap(), "OPEN: New meaning?\n");
}

fn hash(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn precheck_reads_escaped_case_cells_and_ignores_illustrative_cases() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("contract.md");
    fs::write(&path, BASE.replace("| input |", "| a\\|b |")).unwrap();
    assert!(caac::harness::evaluate_completion(std::slice::from_ref(&path)).is_empty());
    fs::write(&path, "RULE-001 | Service must return a value.\n## Contract cases\n```\n| fake | input | value | RULE-001 |\n```\n").unwrap();
    let issues = caac::harness::evaluate_completion(std::slice::from_ref(&path));
    assert!(
        issues
            .iter()
            .any(|issue| issue.contains("RULE-001 has no acceptance case"))
    );
}

#[test]
fn generated_metadata_does_not_hide_tampered_visible_requirements() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("contract.md");
    let schema: caac::codegen::ContractSchema = serde_json::from_value(serde_json::json!({
        "name": "Example",
        "clauses": [{"id": "RULE-001", "actor": "Service", "obligation": "must return a value."}],
        "cases": [{"name": "one", "input": "a|b", "expected": "value", "clauses": ["RULE-001"]}]
    }))
    .unwrap();
    let markdown = caac::codegen::generate_markdown(&schema).unwrap();
    fs::write(&path, &markdown).unwrap();
    assert!(caac::harness::evaluate_completion(std::slice::from_ref(&path)).is_empty());
    for addition in [
        "RULE-999 | Service must erase data.\n",
        "RULE-999 | malformed\n",
    ] {
        fs::write(&path, format!("{markdown}{addition}")).unwrap();
        let issues = caac::harness::evaluate_completion(std::slice::from_ref(&path));
        assert!(
            !issues.is_empty(),
            "visible additions must not inherit metadata coverage"
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("differs from its embedded schema"))
        );
    }
}

#[test]
fn interview_answers_cannot_hide_uncovered_requirements() {
    for answer_text in ["<!--\n", "-->\n"] {
        let dir = tempdir().unwrap();
        let path = dir.path().join("contract.md");
        let original = format!(
            "{BASE}OPEN: Which storage strategy?\nRULE-002 | Service must retain audit logs.\n<!-- explanatory note -->\n"
        );
        fs::write(&path, &original).unwrap();
        let output = answer(&["run", "--max-iterations", "2"], &path, answer_text);
        assert_eq!(output.status.code(), Some(2));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("PRE-REVIEW CHECK: PASSED"));
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        assert!(
            caac::harness::evaluate_completion(&[path])
                .iter()
                .any(|issue| issue.contains("open decision"))
        );
    }
}

#[test]
fn pending_checkpoint_cannot_replay_comment_injection() {
    for separator in [
        "\n", "\r", "\r\n", "\u{000b}", "\u{000c}", "\u{001c}", "\u{001d}", "\u{001e}", "\u{0085}",
        "\u{2028}", "\u{2029}",
    ] {
        let dir = tempdir().unwrap();
        let path = dir.path().join("contract.md");
        let original = [
            "RULE-001 | Service must respond.",
            "OPEN: First?",
            "RULE-002 | Service must retain logs.",
            "<!-- note -->",
            "",
        ]
        .join(separator);
        fs::write(&path, &original).unwrap();
        assert!(
            command()
                .args(["interview", "--auto"])
                .arg(&path)
                .output()
                .unwrap()
                .status
                .success()
        );
        let state_file = state_path(&path);
        let mut state: Value = serde_json::from_slice(&fs::read(&state_file).unwrap()).unwrap();
        let injected = original.replace("OPEN: First?", "DECISION: <!--");
        state["pending"] = serde_json::json!({"question_id":"fixture","before_revision":hash(&original),"after_revision":hash(&injected),"content":injected});
        fs::write(&state_file, serde_json::to_vec(&state).unwrap()).unwrap();
        let output = answer(&["resume"], &path, "");
        assert_eq!(output.status.code(), Some(2), "separator {separator:?}");
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }
}

#[test]
fn pending_recovery_preserves_excluded_decision_examples() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    let original =
        "```text\nDECISION: <!--\n```\nOPEN: Real?\nRULE-001 | Service must retain logs.\n";
    fs::write(&path, original).unwrap();
    assert!(
        command()
            .args(["interview", "--auto"])
            .arg(&path)
            .output()
            .unwrap()
            .status
            .success()
    );
    let state_file = state_path(&path);
    let mut state: Value = serde_json::from_slice(&fs::read(&state_file).unwrap()).unwrap();
    let replacement = original.replace("OPEN: Real?", "DECISION: keep");
    state["pending"] = serde_json::json!({"question_id":"fixture","before_revision":hash(original),"after_revision":hash(&replacement),"content":replacement});
    fs::write(&state_file, serde_json::to_vec(&state).unwrap()).unwrap();
    let output = answer(&["resume"], &path, "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), replacement);
}

#[test]
fn pending_checkpoint_rejects_changes_outside_its_answer() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    let original = "OPEN: First?\nRULE-001 | Service must retain logs.\n";
    fs::write(&path, original).unwrap();
    assert!(
        command()
            .args(["interview", "--auto"])
            .arg(&path)
            .output()
            .unwrap()
            .status
            .success()
    );
    let state_file = state_path(&path);
    let mut state: Value = serde_json::from_slice(&fs::read(&state_file).unwrap()).unwrap();
    let replacement = "DECISION: keep\nRULE-001 | Service must erase logs.\n";
    state["pending"] = serde_json::json!({"question_id":"fixture","before_revision":hash(original),"after_revision":hash(replacement),"content":replacement});
    fs::write(&state_file, serde_json::to_vec(&state).unwrap()).unwrap();
    let output = answer(&["resume"], &path, "");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
}

#[test]
fn generated_markdown_optional_sections_do_not_block_visible_precheck() {
    for exclusions in [false, true] {
        for extends in [false, true] {
            for covered in [false, true] {
                let dir = tempdir().unwrap();
                let source = dir.path().join("schema.json");
                let contract = dir.path().join("contract.md");
                let schema = serde_json::json!({
                    "name": "Service",
                    "operations": [{"name": "ping", "params": "", "returns": "i64", "clauses": ["PING-001"]}],
                    "clauses": [{"id": "PING-001", "actor": "Service", "obligation": "must return one."}],
                    "cases": if covered { serde_json::json!([{"name": "one", "input": "none", "expected": "one", "clauses": ["PING-001"]}]) } else { serde_json::json!([]) },
                    "exclusions": if exclusions { vec!["network access"] } else { vec![] },
                    "extends": if extends { vec!["Base@1.0"] } else { vec![] },
                });
                fs::write(&source, serde_json::to_vec(&schema).unwrap()).unwrap();
                let generated = command()
                    .arg("md")
                    .arg(&source)
                    .arg("--output")
                    .arg(&contract)
                    .output()
                    .unwrap();
                assert!(
                    generated.status.success(),
                    "{}",
                    String::from_utf8_lossy(&generated.stderr)
                );
                let output = command()
                    .args(["run", "--max-iterations", "1"])
                    .arg(&contract)
                    .output()
                    .unwrap();
                assert_eq!(output.status.code(), Some(if covered { 0 } else { 1 }));
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(!stdout.contains("unsupported"), "{stdout}");
                if covered {
                    assert!(stdout.contains("PRE-REVIEW CHECK: PASSED"), "{stdout}");
                    assert!(stdout.contains("owner acceptance remain pending"));
                } else {
                    assert!(
                        stdout.contains("PING-001 has no acceptance case"),
                        "{stdout}"
                    );
                }
            }
        }
    }
}

#[test]
fn harness_uses_checker_decisions_for_every_supported_line_separator() {
    for separator in [
        "\n", "\r", "\r\n", "\u{000b}", "\u{000c}", "\u{001c}", "\u{001d}", "\u{001e}", "\u{0085}",
        "\u{2028}", "\u{2029}",
    ] {
        let dir = tempdir().unwrap();
        let path = dir.path().join("contract.md");
        let original = format!("{BASE}```text\nOPEN: Excluded example?\n```\nOPEN: Which value?\n")
            .replace('\n', separator);
        fs::write(&path, &original).unwrap();
        let output = answer(&["run", "--max-iterations", "2"], &path, "one\n");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "separator {separator:?}: {stdout}");
        assert!(stdout.contains("harness iteration 2"));
        assert!(stdout.contains("PRE-REVIEW CHECK: PASSED"));
        assert!(!stdout.contains("Decision 1 of 1: Excluded"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            original.replace("OPEN: Which value?", "DECISION: one")
        );
    }
}

#[test]
fn harness_does_not_interview_excluded_decisions_in_other_files() {
    let dir = tempdir().unwrap();
    let real = dir.path().join("real.md");
    let example = dir.path().join("example.md");
    fs::write(&real, format!("{BASE}OPEN: Which value?\n")).unwrap();
    let excluded = format!(
        "{}\n## Example (non-normative)\nOPEN: Not a question?\n",
        BASE.replace("RULE-001", "RULE-002")
    );
    fs::write(&example, &excluded).unwrap();
    let output = answer(
        &["run", "--max-iterations", "2", example.to_str().unwrap()],
        &real,
        "one\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(fs::read_to_string(&example).unwrap(), excluded);
    assert!(!state_path(&example).exists());
}
