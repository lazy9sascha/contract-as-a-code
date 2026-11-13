use std::fs;
use std::process::Command;

#[test]
fn quoted_markers_use_matching_backtick_runs() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("contract.md");
    fs::write(&input, "X-001 | Service must return a result.\n``may`` means permission.\n``text `must` text`` is illustrative.\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .arg("check")
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    fs::write(
        &input,
        "X-001 | Service must return a result.\n`Service must return another result.\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .arg("check")
        .arg(&input)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("unnumbered-obligation"));
}

#[test]
fn frontmatter_is_excluded_and_unclosed_frontmatter_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("contract.md");
    fs::write(
        &input,
        "---\nRULE-001 | Server must retain logs.\n---\nRULE-002 | Server must apply @RULE-001.\n",
    )
    .unwrap();
    let diagnostics = caac::checker::check(std::slice::from_ref(&input)).unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "unknown-id" && d.line == 4)
    );
    fs::write(&input, "---\nRULE-001 | Server must retain logs.\n").unwrap();
    let diagnostics = caac::checker::check(&[input]).unwrap();
    assert!(diagnostics.iter().any(|d| d.code == "unclosed-frontmatter"));
    assert!(diagnostics.iter().any(|d| d.code == "no-clauses"));
}
