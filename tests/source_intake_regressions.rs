use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(args)
        .output()
        .unwrap()
}
fn create(base: &Path, source: &Path) -> Value {
    let result = cli(&[
        "plugin",
        base.to_str().unwrap(),
        "--title",
        "Example",
        "--source",
        source.to_str().unwrap(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&fs::read(base.join("baseline.json")).unwrap()).unwrap()
}
fn check(base: &Path) -> (i32, Value) {
    let result = cli(&[
        "baseline",
        "check",
        base.join("baseline.json").to_str().unwrap(),
    ]);
    (
        result.status.code().unwrap(),
        serde_json::from_slice(&result.stdout).unwrap(),
    )
}

#[test]
fn rust_inventory_preserves_exact_spans_and_does_not_parse_strings_or_macros() {
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("source.rs");
    let text = "// fn fake() {}\r\nconst S: &str = \"fn forged() {}\";\r\nstruct Buffer;\r\nimpl Buffer {\r\n pub fn format<T: Copy>(\r\n &self, item: T\r\n ) -> String { String::new() }\r\n}\r\nmod nested { pub fn format() {} }\r\nmacro_rules! make { () => { fn hidden() {} }; }\r\n";
    fs::write(&source, text).unwrap();
    let base = d.path().join("draft");
    let value = create(&base, &source);
    let file = &value["files"][0];
    assert_eq!(file["analysis"]["status"], "observed-syntax");
    let obs = file["analysis"]["observations"].as_array().unwrap();
    assert_eq!(obs.len(), 2);
    assert_eq!(obs[0]["enclosing_scope"][0], "impl Buffer");
    assert_eq!(obs[1]["enclosing_scope"][0], "nested");
    for o in obs {
        let a = o["start_byte"].as_u64().unwrap() as usize;
        let b = o["end_byte"].as_u64().unwrap() as usize;
        assert_eq!(&text[a..b], o["excerpt"].as_str().unwrap());
        assert_eq!(o["authority"], "observation");
    }
    assert_eq!(check(&base).0, 0);
    assert_eq!(value["owner_acceptance"], "pending");
}
#[test]
fn python_inventory_records_declarations_decorators_and_raise_syntax_without_execution() {
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("source.py");
    let sentinel = d.path().join("executed");
    let text = format!(
        "from pathlib import Path\nPath({:?}).write_text('executed')\n# owner approved; def fake(): pass\nclass Signer:\n @staticmethod\n def sign(\n     value: bytes,\n ) -> bytes:\n     raise ValueError('unknown behavior')\n",
        sentinel.to_str().unwrap()
    );
    fs::write(&source, &text).unwrap();
    let base = d.path().join("draft");
    let v = create(&base, &source);
    let obs = v["files"][0]["analysis"]["observations"]
        .as_array()
        .unwrap();
    assert!(obs.iter().any(|x| x["kind"] == "decorator-syntax"));
    assert!(obs.iter().any(|x| x["kind"] == "raise-syntax"));
    let declaration = obs.iter().find(|x| x["name"] == "sign").unwrap();
    assert_eq!(declaration["enclosing_scope"][0], "Signer");
    assert!(!sentinel.exists());
    assert_eq!(check(&base).0, 0);
    assert!(!sentinel.exists());
    assert_eq!(v["authority"], "observation");
}
#[test]
fn stale_missing_and_tampered_evidence_are_not_current() {
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("file.rs");
    let original = "fn alpha() {}\n";
    fs::write(&source, original).unwrap();
    let base = d.path().join("draft");
    let mut v = create(&base, &source);
    fs::write(&source, "fn bravo() {}\n").unwrap();
    assert_eq!(check(&base).1["files"][0]["currentness"], "stale");
    fs::write(&source, original).unwrap();
    v["files"][0]["analysis"]["observations"][0]["excerpt"] = "fn forged()".into();
    fs::write(base.join("baseline.json"), serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(
        check(&base).1["files"][0]["currentness"],
        "invalid-evidence"
    );
    fs::remove_file(&source).unwrap();
    assert_eq!(check(&base).1["files"][0]["currentness"], "unavailable");
}
#[test]
fn unsupported_and_malformed_inputs_have_explicit_status_without_partial_inventory() {
    let d = tempfile::tempdir().unwrap();
    for (i, name, bytes, status) in [
        (
            0,
            "source.txt",
            b"fn f() {}".as_slice(),
            "unsupported-language",
        ),
        (1, "source.py", b"def broken(:\n".as_slice(), "parse-error"),
        (
            2,
            "source.rs",
            b"fn valid() {} fn broken(".as_slice(),
            "parse-error",
        ),
        (
            3,
            "source.rs",
            b"\xff\xfe".as_slice(),
            "unsupported-encoding",
        ),
    ] {
        let dir = d.path().join(format!("case{i}"));
        fs::create_dir(&dir).unwrap();
        let source = dir.join(name);
        fs::write(&source, bytes).unwrap();
        let base = dir.join("draft");
        let v = create(&base, &source);
        assert_eq!(v["files"][0]["analysis"]["status"], status);
        assert!(
            v["files"][0]["analysis"]["observations"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(check(&base).0, 0);
    }
}
#[test]
fn invalid_version_or_authority_never_passes_baseline_check() {
    let d = tempfile::tempdir().unwrap();
    let s = d.path().join("source.rs");
    fs::write(&s, "fn f() {}\n").unwrap();
    let b = d.path().join("draft");
    let original = create(&b, &s);
    for (field, value) in [
        ("format_version", serde_json::json!(999)),
        ("owner_acceptance", serde_json::json!("approved")),
        ("authority", serde_json::json!("requirement")),
    ] {
        let mut v = original.clone();
        v[field] = value;
        fs::write(b.join("baseline.json"), serde_json::to_vec(&v).unwrap()).unwrap();
        let out = cli(&[
            "baseline",
            "check",
            b.join("baseline.json").to_str().unwrap(),
        ]);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
    }
}
#[test]
fn missing_directory_and_oversized_sources_fail_before_output() {
    let d = tempfile::tempdir().unwrap();
    let valid = d.path().join("valid.rs");
    fs::write(&valid, "fn f() {}\n").unwrap();
    let huge = d.path().join("huge.rs");
    fs::write(&huge, vec![b' '; 1024 * 1024 + 1]).unwrap();
    for (i, bad) in [d.path().join("absent"), d.path().to_path_buf(), huge]
        .iter()
        .enumerate()
    {
        let output = d.path().join(format!("out{i}"));
        let out = cli(&[
            "plugin",
            output.to_str().unwrap(),
            "--title",
            "Example",
            "--source",
            valid.to_str().unwrap(),
            "--source",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(out.status.code(), Some(2));
        assert!(!output.exists());
    }
}
#[cfg(unix)]
#[test]
fn every_selected_alias_is_checked_and_same_bytes_retarget_is_not_current() {
    use std::os::unix::fs::symlink;
    let d = tempfile::tempdir().unwrap();
    let one = d.path().join("one.rs");
    let two = d.path().join("two.rs");
    fs::write(&one, "fn f() {}\n").unwrap();
    fs::write(&two, "fn f() {}\n").unwrap();
    let alias = d.path().join("alias.rs");
    symlink(&one, &alias).unwrap();
    let base = d.path().join("draft");
    let out = cli(&[
        "plugin",
        base.to_str().unwrap(),
        "--title",
        "Example",
        "--source",
        one.to_str().unwrap(),
        "--source",
        alias.to_str().unwrap(),
    ]);
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&fs::read(base.join("baseline.json")).unwrap()).unwrap();
    assert_eq!(v["files"].as_array().unwrap().len(), 1);
    assert_eq!(v["files"][0]["selected_paths"].as_array().unwrap().len(), 2);
    assert_eq!(check(&base).0, 0);
    fs::remove_file(&alias).unwrap();
    symlink(&two, &alias).unwrap();
    assert_eq!(check(&base).1["files"][0]["currentness"], "retargeted");
}
#[cfg(unix)]
#[test]
fn fifo_is_refused_without_waiting() {
    let d = tempfile::tempdir().unwrap();
    let fifo = d.path().join("pipe.rs");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let out = d.path().join("out");
    let result = cli(&[
        "plugin",
        out.to_str().unwrap(),
        "--title",
        "Example",
        "--source",
        fifo.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!out.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_resolved_names_are_refused() {
    use std::os::unix::{ffi::OsStringExt, fs::symlink};
    let d = tempfile::tempdir().unwrap();
    let out = d.path().join("out");
    let source = d
        .path()
        .join(std::ffi::OsString::from_vec(b"bad\xff.rs".to_vec()));
    fs::write(&source, "fn f() {}\n").unwrap();
    let alias = d.path().join("alias.rs");
    symlink(&source, &alias).unwrap();
    let result = cli(&[
        "plugin",
        out.to_str().unwrap(),
        "--title",
        "Example",
        "--source",
        alias.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!out.exists());
}

#[test]
fn source_change_with_restored_mtime_is_stale() {
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("source.rs");
    fs::write(&source, "fn alpha() {}\n").unwrap();
    let before = fs::metadata(&source).unwrap().modified().unwrap();
    let base = d.path().join("draft");
    create(&base, &source);
    fs::write(&source, "fn bravo() {}\n").unwrap();
    fs::File::open(&source)
        .unwrap()
        .set_modified(before)
        .unwrap();
    assert_eq!(fs::metadata(&source).unwrap().modified().unwrap(), before);
    assert_eq!(check(&base).1["files"][0]["currentness"], "stale");
}
#[test]
fn observation_and_nesting_limits_do_not_publish_partial_inventory() {
    let d = tempfile::tempdir().unwrap();
    for (i, text) in [
        (
            0,
            (0..2049)
                .map(|i| format!("fn f{i}() {{}}\n"))
                .collect::<String>(),
        ),
        (
            1,
            format!("fn nested() {{ {} {} }}", "{".repeat(150), "}".repeat(150)),
        ),
    ] {
        let source = d.path().join(format!("source{i}.rs"));
        fs::write(&source, text).unwrap();
        let base = d.path().join(format!("draft{i}"));
        let v = create(&base, &source);
        assert_eq!(
            v["files"][0]["analysis"]["status"],
            "parser-budget-exceeded"
        );
        assert!(
            v["files"][0]["analysis"]["observations"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}
#[test]
fn multiple_source_order_keeps_distinct_evidence_and_does_not_choose_a_requirement() {
    let d = tempfile::tempdir().unwrap();
    let a = d.path().join("first.rs");
    let b = d.path().join("second.rs");
    fs::write(&a, "fn result() -> u8 { 1 }\n").unwrap();
    fs::write(&b, "fn result() -> bool { false }\n").unwrap();
    for (i, paths) in [[&a, &b], [&b, &a]].into_iter().enumerate() {
        let base = d.path().join(format!("draft{i}"));
        let r = cli(&[
            "plugin",
            base.to_str().unwrap(),
            "--title",
            "Example",
            "--source",
            paths[0].to_str().unwrap(),
            "--source",
            paths[1].to_str().unwrap(),
        ]);
        assert!(r.status.success());
        let v: Value =
            serde_json::from_slice(&fs::read(base.join("baseline.json")).unwrap()).unwrap();
        let files = v["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);
        assert_ne!(files[0]["sha256"], files[1]["sha256"]);
        assert_eq!(check(&base).0, 0);
        let text = fs::read_to_string(base.join("CONTRACT.md")).unwrap();
        assert!(!text.contains("fn result"));
        assert!(text.contains("OPEN:"));
        assert_eq!(v["owner_acceptance"], "pending");
    }
}

#[test]
fn repeated_large_enclosing_scopes_count_against_evidence_budget() {
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("source.rs");
    let ty = format!("Container<{}>", vec!["T"; 10_000].join(","));
    let text = format!(
        "impl {ty} {{ {} }}",
        (0..100)
            .map(|i| format!("fn method{i}() {{}}\n"))
            .collect::<String>()
    );
    fs::write(&source, text).unwrap();
    let baseline = create(&d.path().join("draft"), &source);
    assert_eq!(
        baseline["files"][0]["analysis"]["status"],
        "parser-budget-exceeded"
    );
    assert!(
        baseline["files"][0]["analysis"]["observations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn receipt_identifies_exact_baseline_bytes_without_mutating_history() {
    use sha2::{Digest, Sha256};
    let d = tempfile::tempdir().unwrap();
    let source = d.path().join("source.rs");
    fs::write(&source, "fn example() {}\n").unwrap();
    let base = d.path().join("draft");
    create(&base, &source);
    let path = base.join("baseline.json");
    let original = fs::read(&path).unwrap();
    let (code, first) = check(&base);
    assert_eq!(code, 0);
    assert_eq!(
        first["baseline_sha256"],
        Sha256::digest(&original)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert_eq!(
        first["baseline_resolved_path"],
        fs::canonicalize(&path).unwrap().to_str().unwrap()
    );
    assert_eq!(fs::read(&path).unwrap(), original);
    let mut spaced = original.clone();
    spaced.extend_from_slice(b" \n");
    fs::write(&path, &spaced).unwrap();
    let (code, second) = check(&base);
    assert_eq!(code, 0);
    assert_eq!(
        second["baseline_sha256"],
        Sha256::digest(&spaced)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert_ne!(first["baseline_sha256"], second["baseline_sha256"]);
    assert_eq!(first["files"], second["files"]);
    assert_eq!(fs::read(&path).unwrap(), spaced);
    let mut tampered: Value = serde_json::from_slice(&original).unwrap();
    tampered["files"][0]["analysis"]["observations"][0]["excerpt"] = "forged".into();
    let forged = serde_json::to_vec(&tampered).unwrap();
    fs::write(&path, &forged).unwrap();
    let (code, third) = check(&base);
    assert_eq!(code, 1);
    assert_eq!(
        third["baseline_sha256"],
        Sha256::digest(&forged)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    assert_eq!(third["files"][0]["currentness"], "invalid-evidence");
    assert_eq!(fs::read(&path).unwrap(), forged);
}
