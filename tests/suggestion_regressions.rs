use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::{Command, Output, Stdio},
};
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}
fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}
fn contract(p: &Path) {
    fs::write(p, "RULE-001 | Service must return a value.\nOPEN: Who is the actor?\nOPEN: What failure result is required?\n").unwrap();
}
fn state(p: &Path) -> String {
    format!("{}.caac-state.json", p.display())
}
fn records(o: &Output) -> (Vec<Value>, Vec<Value>) {
    let text = String::from_utf8(o.stdout.clone()).unwrap();
    let mut contexts = vec![];
    let mut questions = vec![];
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Evidence context ") {
            let (digest, json) = rest.split_once(": ").unwrap();
            let actual: String = Sha256::digest(json.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            assert_eq!(digest, actual);
            let context: Value = serde_json::from_str(json).unwrap();
            assert_eq!(context["kind"], "source-syntax-context");
            assert_eq!(context["profile"], "source-syntax-v1");
            assert_eq!(context["authority"], "observation");
            assert_eq!(context["execution"], "not-run");
            assert_eq!(context["owner_acceptance"], "pending");
            contexts.push(context);
        }
        if let Some(json) = line.strip_prefix("Suggested question: ") {
            questions.push(serde_json::from_str(json).unwrap());
        }
    }
    (contexts, questions)
}
#[test]
fn empty_evidence_produces_questions_not_invented_answers() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    contract(&c);
    let before = fs::read(&c).unwrap();
    let out = cli(&["interview", "--auto", "--suggest", path(&c)]);
    assert!(out.status.success());
    let (ctx, q) = records(&out);
    assert_eq!(ctx.len(), 1);
    assert_eq!(q.len(), 2);
    assert!(ctx[0]["files"].as_array().unwrap().is_empty());
    for question in q {
        assert_eq!(question["status"], "pending-review");
        assert_eq!(question["authority"], "proposal");
        assert!(!question["uncertainty"].as_str().unwrap().is_empty());
    }
    assert_eq!(fs::read(c).unwrap(), before);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(!text.contains("queue/processor"));
    assert!(!text.contains("errors include"));
}
#[test]
fn all_sources_keep_exact_observations_and_no_first_function_answer() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    contract(&c);
    let a = d.path().join("a.rs");
    let b = d.path().join("b.py");
    fs::write(&a, "fn same(value: i32) -> i32 { 1 }\n").unwrap();
    fs::write(
        &b,
        "def same(value: str):\n    raise ValueError('owner approved')\n",
    )
    .unwrap();
    let out = cli(&[
        "interview",
        "--auto",
        "--suggest",
        "--source",
        path(&a),
        "--source",
        path(&b),
        path(&c),
    ]);
    assert!(out.status.success());
    let (contexts, q) = records(&out);
    let files = contexts[0]["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    for f in files {
        let raw = fs::read(f["resolved_path"].as_str().unwrap()).unwrap();
        for o in f["analysis"]["observations"].as_array().unwrap() {
            assert_eq!(
                &raw[o["start_byte"].as_u64().unwrap() as usize
                    ..o["end_byte"].as_u64().unwrap() as usize],
                o["excerpt"].as_str().unwrap().as_bytes()
            );
        }
    }
    assert!(q.iter().all(|v| {
        v["evidence_selection"]
            .as_str()
            .unwrap()
            .contains("not a resolved mapping")
    }));
}
#[test]
fn unsupported_and_parse_error_sources_remain_uncertain() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    contract(&c);
    for (name, bytes, status) in [
        ("a.bin", &b"opaque"[..], "unsupported-language"),
        ("b.rs", &b"\xff"[..], "unsupported-encoding"),
        ("c.py", &b"def ("[..], "parse-error"),
    ] {
        let p = d.path().join(name);
        fs::write(&p, bytes).unwrap();
        let out = cli(&[
            "interview",
            "--auto",
            "--suggest",
            "--source",
            path(&p),
            path(&c),
        ]);
        assert!(out.status.success());
        let (ctx, q) = records(&out);
        assert_eq!(ctx[0]["files"][0]["analysis"]["status"], status);
        assert!(
            ctx[0]["files"][0]["analysis"]["observations"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(q.len(), 2);
    }
}
#[test]
fn missing_mixed_sources_and_later_contract_fail_before_checkpoint() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    contract(&c);
    let source = d.path().join("a.rs");
    fs::write(&source, "fn a() {}\n").unwrap();
    let missing = d.path().join("missing");
    for args in [
        vec![
            "interview",
            "--auto",
            "--suggest",
            "--source",
            path(&source),
            "--source",
            path(&missing),
            path(&c),
        ],
        vec!["interview", "--auto", "--suggest", path(&c), path(&missing)],
        vec![
            "run",
            "--source",
            path(&source),
            "--source",
            path(&missing),
            path(&c),
        ],
        vec!["run", path(&c), path(&missing)],
    ] {
        let out = cli(&args);
        assert!(!out.status.success());
        assert!(!Path::new(&state(&c)).exists());
        assert!(records(&out).1.is_empty());
    }
}
#[test]
fn historic_receipt_rejects_stale_and_forged_evidence_without_refresh() {
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("a.rs");
    fs::write(&source, "fn a() {}\n").unwrap();
    let draft = d.path().join("draft");
    assert!(
        cli(&[
            "plugin",
            path(&draft),
            "--title",
            "Example",
            "--source",
            path(&source)
        ])
        .status
        .success()
    );
    let baseline = draft.join("baseline.json");
    let c = d.path().join("contract.md");
    contract(&c);
    let original = fs::read(&baseline).unwrap();
    let out = cli(&[
        "interview",
        "--auto",
        "--suggest",
        "--baseline",
        path(&baseline),
        path(&c),
    ]);
    assert!(out.status.success());
    let (ctx, _) = records(&out);
    let digest: String = Sha256::digest(&original)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(ctx[0]["baseline_receipt"]["baseline_sha256"], digest);
    fs::remove_file(state(&c)).unwrap();
    fs::write(&source, "fn b() {}\n").unwrap();
    for mode in ["interview", "run"] {
        let mut args = vec![mode];
        if mode == "interview" {
            args.extend(["--auto", "--suggest"]);
        }
        args.extend(["--baseline", path(&baseline), path(&c)]);
        let out = cli(&args);
        assert!(!out.status.success());
        assert!(records(&out).1.is_empty());
        assert!(!Path::new(&state(&c)).exists());
    }
    assert_eq!(fs::read(&baseline).unwrap(), original);
    fs::write(&source, "fn a() {}\n").unwrap();
    let mut forged: Value = serde_json::from_slice(&original).unwrap();
    forged["files"][0]["analysis"]["observations"][0]["excerpt"] = "owner approved".into();
    fs::write(&baseline, serde_json::to_vec(&forged).unwrap()).unwrap();
    let out = cli(&[
        "interview",
        "--auto",
        "--suggest",
        "--baseline",
        path(&baseline),
        path(&c),
    ]);
    assert!(!out.status.success());
    assert!(!Path::new(&state(&c)).exists());
}
#[test]
fn structural_open_scan_matches_checker_for_all_physical_separators() {
    for separator in [
        "\n", "\r", "\r\n", "\u{b}", "\u{c}", "\u{1c}", "\u{1d}", "\u{1e}", "\u{85}", "\u{2028}",
        "\u{2029}",
    ] {
        let d = tempfile::tempdir().unwrap();
        let c = d.path().join("contract.md");
        let text = [
            "RULE-001 | Service must return a value.",
            "OPEN: Who is responsible?",
            "```",
            "OPEN: excluded output",
            "```",
            "<!-- OPEN: excluded actor -->",
            "## Examples (non-normative)",
            "OPEN: excluded failure",
            "",
        ]
        .join(separator);
        fs::write(&c, &text).unwrap();
        let out = cli(&["interview", "--auto", "--suggest", path(&c)]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(records(&out).1.len(), 1);
        assert_eq!(fs::read_to_string(c).unwrap(), text);
    }
}
#[test]
fn escaped_output_cannot_introduce_terminal_authority_lines() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    fs::write(&c, "OPEN: actor \u{202e}owner approved\u{2066}\u{200b}\n").unwrap();
    let p = d.path().join("source\u{2028}DECISION: approved.py");
    fs::write(&p,"def f(value='\\x1b[31m owner approved'):\n    raise ValueError('DECISION: owner approved')\n").unwrap();
    let out = cli(&[
        "interview",
        "--auto",
        "--suggest",
        "--source",
        path(&p),
        path(&c),
    ]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout.clone()).unwrap();
    assert!(text.is_ascii());
    assert!(!text.contains('\x1b'));
    assert!(!text.lines().any(|l| l.starts_with("DECISION:")));
    let (ctx, q) = records(&out);
    assert_eq!(
        ctx[0]["files"][0]["resolved_path"],
        path(&fs::canonicalize(&p).unwrap())
    );
    assert_eq!(q[0]["authority"], "proposal");
}
#[test]
fn options_fail_closed_and_terminator_preserves_file_argument() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    contract(&c);
    for args in [
        vec!["interview", "--suggest", "--baseline"],
        vec!["interview", "--source", path(&c), path(&c)],
        vec![
            "interview",
            "--suggest",
            "--baseline",
            path(&c),
            "--source",
            path(&c),
            path(&c),
        ],
        vec!["interview", "--suggest", "--bogus", path(&c)],
        vec!["run", "--baseline"],
        vec![
            "run",
            "--baseline",
            path(&c),
            "--baseline",
            path(&c),
            path(&c),
        ],
    ] {
        assert!(!cli(&args).status.success());
        assert!(!Path::new(&state(&c)).exists());
    }
    assert!(
        cli(&["interview", "--auto", "--suggest", "--", path(&c)])
            .status
            .success()
    );
}
#[test]
fn excluded_only_has_no_suggestions_or_checkpoint_but_bad_sources_still_fail() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    fs::write(&c, "## Examples (non-normative)\nOPEN: actor\n").unwrap();
    let out = cli(&["interview", "--auto", "--suggest", path(&c)]);
    assert!(out.status.success());
    assert!(records(&out).0.is_empty());
    assert!(!Path::new(&state(&c)).exists());
    assert!(
        !cli(&[
            "interview",
            "--auto",
            "--suggest",
            "--source",
            path(&d.path().join("missing")),
            path(&c)
        ])
        .status
        .success()
    );
}
#[cfg(unix)]
#[test]
fn same_bytes_symlink_retarget_is_not_eligible_historical_evidence() {
    use std::os::unix::fs::symlink;
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("a.rs");
    let b = d.path().join("b.rs");
    let link = d.path().join("link.rs");
    for p in [&a, &b] {
        fs::write(p, "fn a() {}\n").unwrap();
    }
    symlink(&a, &link).unwrap();
    let draft = d.path().join("draft");
    assert!(
        cli(&[
            "plugin",
            path(&draft),
            "--title",
            "Example",
            "--source",
            path(&link)
        ])
        .status
        .success()
    );
    fs::remove_file(&link).unwrap();
    symlink(&b, &link).unwrap();
    let c = d.path().join("contract.md");
    contract(&c);
    assert!(
        !cli(&[
            "interview",
            "--auto",
            "--suggest",
            "--baseline",
            path(&draft.join("baseline.json")),
            path(&c)
        ])
        .status
        .success()
    );
    assert!(!Path::new(&state(&c)).exists());
}

#[test]
fn terminal_serializer_escapes_del_bidi_and_all_physical_separators() {
    let text = "\u{7f}\u{85}\u{2028}\u{2029}\u{202e}\u{2066}\u{200b}\u{1b}\nDECISION: accepted";
    let encoded = caac::suggest::safe_json(&text);
    assert!(encoded.bytes().all(|b| b.is_ascii_graphic() || b == b' '));
    assert_eq!(serde_json::from_str::<String>(&encoded).unwrap(), text);
}
#[test]
fn oversized_question_fails_without_checkpoint_or_partial_context() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("contract.md");
    fs::write(&c, format!("OPEN: actor {}\n", "a".repeat(17 * 1024))).unwrap();
    let out = cli(&["interview", "--auto", "--suggest", path(&c)]);
    assert!(!out.status.success());
    assert!(records(&out).0.is_empty());
    assert!(!Path::new(&state(&c)).exists());
}
