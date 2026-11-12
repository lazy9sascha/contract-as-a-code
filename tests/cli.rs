use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::tempdir;

const VALID: &str = "RULE-001 | The processor must return the result.\n";

fn caac() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_caac"));
    command.current_dir(env!("CARGO_MANIFEST_DIR"));
    command
}

fn write_file(directory: &Path, name: &str, contents: &[u8]) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, contents).expect("write test input");
    path
}

fn run_check(paths: &[PathBuf]) -> Output {
    let mut command = caac();
    command.arg("check").arg("--json");
    command.args(paths);
    command.output().expect("run caac check")
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout was not JSON ({error}); stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn diagnostic_codes(value: &Value) -> Vec<String> {
    value["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .map(|diagnostic| {
            diagnostic["code"]
                .as_str()
                .expect("diagnostic code")
                .to_owned()
        })
        .collect()
}

fn assert_code(value: &Value, expected: &str) {
    assert!(
        diagnostic_codes(value).iter().any(|code| code == expected),
        "expected diagnostic {expected:?} in {value}"
    );
}

#[test]
fn all_authoring_modes_create_reviewable_drafts_without_running_sources() {
    let temporary = tempdir().expect("temporary directory");
    let source = write_file(temporary.path(), "source.bin", b"opaque\0source\n");
    let original = fs::read(&source).expect("read source before authoring");
    let expected_hash = Sha256::digest(&original)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    for mode in ["scratch", "plugin", "migration"] {
        let destination = temporary.path().join(mode);
        let mut command = caac();
        command
            .arg(mode)
            .arg(&destination)
            .arg("--title")
            .arg("Delivery");
        if mode != "scratch" {
            command.arg("--source").arg(&source);
        }

        let output = command.output().expect("run authoring mode");
        assert!(
            output.status.success(),
            "{mode} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let contract = destination.join("CONTRACT.md");
        let text = fs::read_to_string(&contract).expect("read generated contract");
        assert!(text.contains("Owner acceptance: pending"));
        let checked = run_check(std::slice::from_ref(&contract));
        assert_eq!(checked.status.code(), Some(1));
        assert_code(&json_output(&checked), "open-decision");

        let baseline_path = destination.join("baseline.json");
        if mode == "scratch" {
            assert!(!baseline_path.exists());
        } else {
            let baseline: Value =
                serde_json::from_slice(&fs::read(&baseline_path).expect("read source baseline"))
                    .expect("parse source baseline");
            assert_eq!(baseline["files"].as_array().unwrap().len(), 1);
            assert_eq!(baseline["files"][0]["sha256"], expected_hash);
            let recorded_path = baseline["files"][0]["path"]
                .as_str()
                .expect("recorded source path");
            assert_eq!(
                destination.join(recorded_path).canonicalize().unwrap(),
                source.canonicalize().unwrap()
            );
            assert_eq!(baseline["execution"], "not-run");
            assert!(text.contains("## Observed behavior"));
        }

        if mode == "migration" {
            assert!(text.contains("## Keep, change, or remove"));
            assert!(text.contains("## Cutover and rollback"));
        }

        assert_eq!(fs::read(&source).unwrap(), original);
    }
}

#[test]
fn authoring_refuses_existing_output_without_changing_it() {
    let temporary = tempdir().unwrap();
    let destination = temporary.path().join("existing");
    fs::create_dir(&destination).unwrap();
    let marker = write_file(&destination, "CONTRACT.md", b"user work");

    let output = caac()
        .arg("scratch")
        .arg(&destination)
        .arg("--title")
        .arg("Test")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read(&marker).unwrap(), b"user work");
}

#[test]
fn invalid_authoring_requests_do_not_create_output() {
    let temporary = tempdir().unwrap();
    let source = write_file(temporary.path(), "source.bin", b"source");
    let missing = temporary.path().join("missing.bin");

    let requests: Vec<Vec<String>> = vec![
        vec![
            "scratch".into(),
            temporary.path().join("empty-title").display().to_string(),
            "--title".into(),
            "".into(),
        ],
        vec![
            "scratch".into(),
            temporary
                .path()
                .join("line-break-title")
                .display()
                .to_string(),
            "--title".into(),
            "a\nb".into(),
        ],
        vec![
            "scratch".into(),
            temporary.path().join("unicode-nel").display().to_string(),
            "--title".into(),
            "a\u{0085}b".into(),
        ],
        vec![
            "scratch".into(),
            temporary.path().join("unicode-ls").display().to_string(),
            "--title".into(),
            "a\u{2028}b".into(),
        ],
        vec![
            "scratch".into(),
            temporary.path().join("unicode-ps").display().to_string(),
            "--title".into(),
            "a\u{2029}b".into(),
        ],
        vec![
            "plugin".into(),
            temporary.path().join("no-source").display().to_string(),
            "--title".into(),
            "Title".into(),
        ],
        vec![
            "plugin".into(),
            temporary
                .path()
                .join("missing-source")
                .display()
                .to_string(),
            "--title".into(),
            "Title".into(),
            "--source".into(),
            missing.display().to_string(),
        ],
        vec![
            "scratch".into(),
            temporary
                .path()
                .join("scratch-source")
                .display()
                .to_string(),
            "--title".into(),
            "Title".into(),
            "--source".into(),
            source.display().to_string(),
        ],
        vec![
            "unknown".into(),
            temporary.path().join("unknown-mode").display().to_string(),
            "--title".into(),
            "Title".into(),
        ],
    ];

    let output_paths = [
        "empty-title",
        "line-break-title",
        "unicode-nel",
        "unicode-ls",
        "unicode-ps",
        "no-source",
        "missing-source",
        "scratch-source",
        "unknown-mode",
    ];

    for (arguments, output_name) in requests.iter().zip(output_paths) {
        let mut command = caac();
        command.args(arguments);
        let output = command.output().expect("run invalid authoring request");
        assert_eq!(output.status.code(), Some(2), "arguments: {arguments:?}");
        assert!(
            !temporary.path().join(output_name).exists(),
            "invalid request created {output_name}"
        );
    }
}

#[test]
fn duplicate_source_arguments_are_recorded_once() {
    let temporary = tempdir().unwrap();
    let source = write_file(temporary.path(), "source.bin", b"source");
    let destination = temporary.path().join("plugin");

    let output = caac()
        .arg("plugin")
        .arg(&destination)
        .arg("--title")
        .arg("Test")
        .arg("--source")
        .arg(&source)
        .arg("--source")
        .arg(&source)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let baseline: Value =
        serde_json::from_slice(&fs::read(destination.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(baseline["files"].as_array().unwrap().len(), 1);
}

#[cfg(unix)]
#[test]
fn baseline_source_paths_resolve_from_the_actual_symlinked_destination() {
    use std::os::unix::fs::symlink;

    let temporary = tempdir().unwrap();
    let root = temporary.path();
    let physical = root.join("physical");
    let nested = physical.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let alias = root.join("alias");
    symlink(&nested, &alias).unwrap();

    let source = write_file(
        root,
        "source.bin",
        b"source through a symlinked destination",
    );
    let destination = alias.join("..").join("output");
    let output = caac()
        .arg("plugin")
        .arg(&destination)
        .arg("--title")
        .arg("Test")
        .arg("--source")
        .arg(&source)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual_destination = destination.canonicalize().unwrap();
    assert_eq!(
        actual_destination,
        physical.join("output").canonicalize().unwrap()
    );

    let baseline: Value =
        serde_json::from_slice(&fs::read(actual_destination.join("baseline.json")).unwrap())
            .unwrap();
    let recorded_path = baseline["files"][0]["path"].as_str().unwrap();
    assert_eq!(
        actual_destination
            .join(recorded_path)
            .canonicalize()
            .unwrap(),
        source.canonicalize().unwrap()
    );
}

#[test]
fn valid_check_reports_pending_semantic_review_and_locations() {
    let temporary = tempdir().unwrap();
    let valid = write_file(temporary.path(), "valid.md", VALID.as_bytes());
    let output = run_check(std::slice::from_ref(&valid));

    assert_eq!(output.status.code(), Some(0));
    let report = json_output(&output);
    assert_eq!(report["syntax"], "passed");
    assert_eq!(report["semantic_review"], "pending");
    assert_eq!(report["diagnostics"], json!([]));

    let reference = write_file(
        temporary.path(),
        "reference.md",
        format!("{VALID}See @RULE-999.\n").as_bytes(),
    );
    let output = run_check(std::slice::from_ref(&reference));
    let report = json_output(&output);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        report["diagnostics"][0]["path"],
        reference.display().to_string()
    );
    assert_eq!(report["diagnostics"][0]["line"], 2);
    assert_eq!(report["diagnostics"][0]["code"], "unknown-id");
}

#[test]
fn conflicting_duplicate_ids_across_files_are_reported() {
    let temporary = tempdir().unwrap();
    let first = write_file(temporary.path(), "first.md", VALID.as_bytes());
    let second = write_file(
        temporary.path(),
        "second.md",
        b"RULE-001 | The processor must not return the result.\n",
    );
    let output = run_check(&[first, second]);

    assert_eq!(output.status.code(), Some(1));
    assert_code(&json_output(&output), "duplicate-id");
}

#[test]
fn code_and_comments_cannot_define_clause_ids() {
    let temporary = tempdir().unwrap();
    let hidden_clauses = [
        format!("<!-- {VALID} -->"),
        format!("```text\n{VALID}```"),
        format!("~~~\n{VALID}~~~"),
    ];

    for (index, hidden) in hidden_clauses.iter().enumerate() {
        let content = format!("{hidden}\nSee @RULE-001.\n");
        let path = write_file(
            temporary.path(),
            &format!("hidden-{index}.md"),
            content.as_bytes(),
        );
        let output = run_check(&[path]);
        let report = json_output(&output);
        assert_code(&report, "no-clauses");
        assert_code(&report, "unknown-id");
    }
}

#[test]
fn nonnormative_section_ends_at_peer_heading() {
    let temporary = tempdir().unwrap();
    let path = write_file(
        temporary.path(),
        "exclusion.md",
        concat!(
            "RULE-001 | The processor must return the result.\n",
            "\n",
            "## Notes (non-normative)\n",
            "You must guess.\n",
            "### Detail\n",
            "You must guess.\n",
            "## Clauses\n",
            "You must not guess.\n",
        )
        .as_bytes(),
    );
    let output = run_check(&[path]);
    let report = json_output(&output);

    assert_eq!(output.status.code(), Some(1));
    let diagnostics = report["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["line"], 8);
    assert_eq!(diagnostics[0]["code"], "unnumbered-obligation");
}

#[test]
fn only_explicit_at_sign_tokens_are_references() {
    let temporary = tempdir().unwrap();
    for (index, text) in [
        format!("{VALID}Digest: SHA-256.\n"),
        format!("{VALID}`must` states an obligation.\n"),
    ]
    .iter()
    .enumerate()
    {
        let path = write_file(
            temporary.path(),
            &format!("prose-{index}.md"),
            text.as_bytes(),
        );
        let output = run_check(&[path]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(json_output(&output)["diagnostics"], json!([]));
    }
}

#[test]
fn malformed_clauses_and_open_decisions_are_reported() {
    let temporary = tempdir().unwrap();
    let cases = [
        ("RULE-001 |\n", "clause-shape"),
        (
            "RULE-001 | The processor returns a result.\n",
            "clause-shape",
        ),
        ("RULE-001 | must return a result.\n", "clause-shape"),
        ("RULE-001 | The processor must.\n", "clause-shape"),
        ("RULE-001 | The processor must   not.\n", "clause-shape"),
        ("RULE-001 | The processor must\t not.\n", "clause-shape"),
        ("RULE-001 | The processor must\t.\n", "clause-shape"),
        (
            "RULE-01 | The processor must return a result.\n",
            "clause-shape",
        ),
        (
            "RULE-001 The processor must return a result.\n",
            "clause-shape",
        ),
        (
            "RULE-001 | The processor must return a result; it must continue.\n",
            "semicolon",
        ),
        (
            "RULE-001 | The processor must return the result.\nThe processor must invent a result.\n",
            "unnumbered-obligation",
        ),
        (
            "RULE-001 | The processor must return the result.\nOPEN: Choose an outcome.\n",
            "open-decision",
        ),
        (
            "RULE-001 | The processor must return the result.\n```\n",
            "unclosed-fence",
        ),
        (
            "RULE-001 | The processor must return the result.\n<!--\n",
            "unclosed-comment",
        ),
    ];

    for (index, (text, expected_code)) in cases.iter().enumerate() {
        let path = write_file(
            temporary.path(),
            &format!("case-{index}.md"),
            text.as_bytes(),
        );
        let output = run_check(&[path]);
        assert_eq!(output.status.code(), Some(1), "case {index}: {text}");
        assert_code(&json_output(&output), expected_code);
    }

    let twenty_one_words = format!(
        "RULE-001 | The processor must {}.\n",
        "word ".repeat(18).trim_end()
    );
    let path = write_file(
        temporary.path(),
        "twenty-one-words.md",
        twenty_one_words.as_bytes(),
    );
    let output = run_check(&[path]);
    assert_eq!(output.status.code(), Some(1));
    assert_code(&json_output(&output), "clause-length");
}

#[test]
fn exactly_twenty_clause_words_are_valid() {
    let temporary = tempdir().unwrap();
    let twenty_words = format!(
        "RULE-001 | The processor must {}.\n",
        "word ".repeat(17).trim_end()
    );
    let path = write_file(temporary.path(), "twenty-words.md", twenty_words.as_bytes());
    let output = run_check(&[path]);
    let report = json_output(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(
        !diagnostic_codes(&report)
            .iter()
            .any(|code| code == "clause-length")
    );
}

#[test]
fn repository_contracts_pass_the_declared_syntax_check() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = run_check(&[
        root.join("CONTRACT.md"),
        root.join("contracts/tooling.md"),
        root.join("examples/batch-division.md"),
    ]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(json_output(&output)["diagnostics"], json!([]));
}

#[test]
fn checker_cli_uses_zero_one_and_two_exit_codes_and_json_reports() {
    let temporary = tempdir().unwrap();
    let path = write_file(temporary.path(), "contract.md", VALID.as_bytes());
    let good = run_check(std::slice::from_ref(&path));
    assert_eq!(good.status.code(), Some(0));
    assert_eq!(json_output(&good)["syntax"], "passed");

    fs::write(&path, format!("{VALID}OPEN: unresolved\n")).unwrap();
    let bad = run_check(std::slice::from_ref(&path));
    assert_eq!(bad.status.code(), Some(1));
    assert_eq!(json_output(&bad)["syntax"], "failed");

    let missing = temporary.path().join("missing.md");
    let unreadable = run_check(&[missing]);
    assert_eq!(unreadable.status.code(), Some(2));
    assert!(json_output(&unreadable)["error"].is_string());

    let no_files = caac().arg("check").arg("--json").output().unwrap();
    assert_eq!(no_files.status.code(), Some(2));
}

#[test]
fn check_honors_the_option_terminator_for_a_file_named_help() {
    let temporary = tempdir().unwrap();
    let path = write_file(temporary.path(), "--help", VALID.as_bytes());
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(temporary.path())
        .args(["check", "--json", "--", "--help"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let report = json_output(&output);
    assert_eq!(report["syntax"], "passed");
    assert_eq!(report["diagnostics"], json!([]));
    assert!(path.exists());
}

#[test]
fn invalid_utf8_is_a_cli_input_error() {
    let temporary = tempdir().unwrap();
    let path = write_file(temporary.path(), "invalid-utf8.md", &[0xff]);
    let output = run_check(&[path]);

    assert_eq!(output.status.code(), Some(2));
    assert!(json_output(&output)["error"].is_string());
}
