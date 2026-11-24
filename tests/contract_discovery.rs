use caac::contract_discovery::{discover, terminal_json};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn temp() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("caac-discovery-")
        .tempdir()
        .unwrap()
}
fn run(p: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(p)
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn explicit_scopes_suggest_then_explicit_collect_preserves_unverified_role() {
    let d = temp();
    let p = d.path();
    fs::create_dir(p.join("contracts")).unwrap();
    fs::write(p.join("CONTRACT.md"), "Root name is not authority").unwrap();
    fs::write(p.join("README.md"), "Index only").unwrap();
    fs::write(
        p.join("contracts/tooling.md"),
        "# 계약\r\nSynthetic, unapproved\r\n",
    )
    .unwrap();
    fs::write(p.join("contracts/private.txt"), "DO NOT READ").unwrap();
    let out = run(
        p,
        &["discover", "--directory", ".", "--directory", "contracts"],
    );
    assert!(out.status.success(), "{:?}", out);
    assert!(out.stderr.is_empty());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["suggestions"].as_array().unwrap().len(), 3);
    assert_eq!(
        v["root_assessment"],
        "not-established-by-metadata-inventory"
    );
    assert_eq!(v["outcome"], "selection-required");
    assert!(
        !String::from_utf8(out.stdout)
            .unwrap()
            .contains("DO NOT READ")
    );
    for s in v["suggestions"].as_array().unwrap() {
        assert_eq!(s["authority"], "unverified-selection-suggestion");
        assert!(s.get("text").is_none());
    }
    let selected = v["suggestions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["filename"] == "tooling.md")
        .unwrap()["path"]
        .as_str()
        .unwrap();
    assert!(
        run(
            p,
            &[
                "collect",
                "--contract",
                selected,
                "--output",
                "selected.json"
            ]
        )
        .status
        .success()
    );
    let collected: Value =
        serde_json::from_slice(&fs::read(p.join("selected.json")).unwrap()).unwrap();
    assert_eq!(
        collected["documents"][0]["text"],
        "# 계약\r\nSynthetic, unapproved\r\n"
    );
    assert_eq!(collected["root"]["finding"], "missing-root-contract");
}
#[test]
fn empty_deduplicated_scopes_do_not_claim_project_absence() {
    let d = temp();
    let p = d.path();
    fs::create_dir(p.join("nested")).unwrap();
    fs::write(p.join("nested/CONTRACT.md"), "not traversed").unwrap();
    fs::write(p.join(".private.md"), "not opened").unwrap();
    let result = discover(&[p.to_path_buf(), p.join(".")]).unwrap();
    let v = serde_json::to_value(result).unwrap();
    assert_eq!(v["directories"].as_array().unwrap().len(), 1);
    assert_eq!(v["suggestions"], serde_json::json!([]));
    assert_eq!(v["outcome"], "no-candidates-in-declared-scope");
    assert_eq!(v["omissions"]["directories"], 1);
    assert_eq!(v["omissions"]["hidden"], 1);
}
#[cfg(unix)]
#[test]
fn no_child_symlink_special_or_body_reads_and_safe_names() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let d = temp();
    let p = d.path();
    fs::create_dir(p.join("scope")).unwrap();
    let scope = p.join("scope");
    fs::write(scope.join("unreadable.md"), "secret body").unwrap();
    fs::set_permissions(scope.join("unreadable.md"), fs::Permissions::from_mode(0o0)).unwrap();
    fs::write(scope.join("한글\u{1b}[2J.md"), "not read").unwrap();
    symlink(p.join("scope/unreadable.md"), scope.join("linked.md")).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(scope.join("pipe.md"))
            .status()
            .unwrap()
            .success()
    );
    let result = discover(std::slice::from_ref(&scope)).unwrap();
    let text = terminal_json(&result).unwrap();
    assert!(!text.contains('\u{1b}'));
    assert!(!text.contains("secret body"));
    assert_eq!(result.suggestions.len(), 2);
    assert_eq!(result.omissions.symlinks, 1);
    assert_eq!(result.omissions.special_files, 1);
    symlink(&scope, p.join("selected-link")).unwrap();
    assert!(discover(&[p.join("selected-link")]).is_err());
    fs::create_dir(p.join(".hidden")).unwrap();
    fs::create_dir(p.join(".hidden/visible")).unwrap();
    symlink(p.join(".hidden"), p.join("alias")).unwrap();
    assert!(discover(&[p.join("alias/visible")]).is_err());
    fs::set_permissions(
        scope.join("unreadable.md"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
}
#[test]
fn bounds_and_cli_errors_produce_no_partial_inventory() {
    let d = temp();
    let p = d.path();
    for i in 0..129 {
        fs::write(p.join(format!("{i}.md")), "").unwrap();
    }
    let o = run(p, &["discover", "--directory", "."]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty());
    assert!(discover(&vec![p.to_path_buf(); 17]).is_err());
    for args in [
        vec!["discover"],
        vec!["discover", "--recursive"],
        vec!["discover", "--directory", ".", "--output", "out"],
    ] {
        let o = run(p, &args);
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
    }
    assert!(!p.join("out").exists());
}
#[test]
fn total_entries_bound_includes_omitted_entries() {
    let d = temp();
    let p = d.path();
    for i in 0..4097 {
        fs::write(p.join(format!("{i}.txt")), "").unwrap();
    }
    let error = discover(&[p.to_path_buf()]).unwrap_err();
    assert!(error.contains("4096"));
}
#[test]
fn deterministic_sorted_api_matches_cli_and_help() {
    let d = temp();
    let p = d.path();
    for name in ["z.markdown", "INDEX.md", "contract-guide.md", "a.MD"] {
        fs::write(p.join(name), "ignored").unwrap();
    }
    let first = terminal_json(&discover(&[p.to_path_buf()]).unwrap()).unwrap();
    let second = terminal_json(&discover(&[p.to_path_buf()]).unwrap()).unwrap();
    assert_eq!(first, second);
    let cli = run(p, &["discover", "--directory", "."]);
    assert!(cli.status.success());
    assert_eq!(String::from_utf8(cli.stdout).unwrap().trim(), first);
    let v: Value = serde_json::from_str(&first).unwrap();
    assert_eq!(v["suggestions"][0]["filename"], "INDEX.md");
    assert_eq!(v["suggestions"][0]["reason"], "navigation-name-hint-only");
    assert!(run(p, &["discover", "--help"]).status.success());
}

#[test]
fn bounded_depth_and_explicit_drilldown_discover_scattered_contracts() {
    use caac::contract_discovery::{Options, discover_with_options};
    let d = temp();
    let p = d.path();
    for dir in [
        "common",
        "cloud/codex",
        "local/codex",
        "local/codex/deeper/final",
    ] {
        fs::create_dir_all(p.join(dir)).unwrap();
    }
    for file in [
        "CONTRACT.md",
        "common/shared.md",
        "cloud/codex/agent.md",
        "local/codex/agent.md",
        "local/codex/deeper/final/limit.md",
    ] {
        fs::write(p.join(file), "private body not inspected").unwrap();
    }
    let zero = discover(&[p.to_path_buf()]).unwrap();
    assert_eq!(zero.suggestions.len(), 1);
    assert_eq!(zero.frontier.len(), 3);
    let common = discover(&[p.join("common")]).unwrap();
    assert_eq!(common.suggestions.len(), 1);
    let one = discover_with_options(&[p.to_path_buf()], &Options { depth: 1 }).unwrap();
    assert_eq!(one.suggestions.len(), 2);
    assert_eq!(one.frontier.len(), 2);
    let two = discover_with_options(&[p.to_path_buf()], &Options { depth: 2 }).unwrap();
    assert_eq!(two.suggestions.len(), 4);
    assert_eq!(two.frontier.len(), 1);
    assert!(two.frontier[0].path.ends_with("local/codex/deeper"));
    let three = discover_with_options(&[p.to_path_buf()], &Options { depth: 3 }).unwrap();
    assert_eq!(three.suggestions.len(), 4);
    assert_eq!(three.frontier.len(), 1);
    assert!(three.frontier[0].path.ends_with("local/codex/deeper/final"));
    let four = discover_with_options(&[p.to_path_buf()], &Options { depth: 4 }).unwrap();
    assert_eq!(four.suggestions.len(), 5);
    assert!(four.frontier.is_empty());
    let overlap = discover_with_options(
        &[p.to_path_buf(), p.join("local/codex"), p.to_path_buf()],
        &Options { depth: 3 },
    )
    .unwrap();
    let reversed = discover_with_options(
        &[p.join("local/codex"), p.to_path_buf(), p.to_path_buf()],
        &Options { depth: 3 },
    )
    .unwrap();
    assert_eq!(
        terminal_json(&overlap).unwrap(),
        terminal_json(&reversed).unwrap()
    );
    let explicit_frontier =
        discover_with_options(&[p.to_path_buf(), p.join("common")], &Options { depth: 0 }).unwrap();
    assert!(
        !explicit_frontier
            .frontier
            .iter()
            .any(|f| f.path.ends_with("/common"))
    );
    assert_eq!(overlap.suggestions.len(), 5);
    assert!(overlap.frontier.is_empty());
    let cli = run(p, &["discover", "--directory", ".", "--depth", "3"]);
    assert!(cli.status.success());
    let value: Value = serde_json::from_slice(&cli.stdout).unwrap();
    assert_eq!(value["depth"], 3);
    assert_eq!(value["suggestions"].as_array().unwrap().len(), 4);
    assert!(
        !String::from_utf8(cli.stdout)
            .unwrap()
            .contains("private body")
    );
}
#[test]
fn depth_validation_and_recursive_global_caps_fail_closed() {
    use caac::contract_discovery::{Options, discover_with_options};
    let d = temp();
    let p = d.path();
    assert!(discover_with_options(&[p.to_path_buf()], &Options { depth: 5 }).is_err());
    for depth in ["-1", "5", "1.0", "01"] {
        let o = run(p, &["discover", "--directory", ".", "--depth", depth]);
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
    }
    assert_eq!(
        run(
            p,
            &[
                "discover",
                "--directory",
                ".",
                "--depth",
                "1",
                "--depth",
                "2"
            ]
        )
        .status
        .code(),
        Some(2)
    );
    for dir in ["a", "b"] {
        fs::create_dir(p.join(dir)).unwrap();
        for i in 0..65 {
            fs::write(p.join(format!("{dir}/{i}.md")), "").unwrap();
        }
    }
    assert!(discover(&[p.to_path_buf()]).unwrap().suggestions.is_empty());
    let o = run(p, &["discover", "--directory", ".", "--depth", "1"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stdout.is_empty());
}
#[cfg(unix)]
#[test]
fn recursive_scope_never_follows_child_symlink_or_hidden_subtree() {
    use caac::contract_discovery::{Options, discover_with_options};
    use std::os::unix::fs::symlink;
    let d = temp();
    let p = d.path();
    fs::create_dir_all(p.join("scope/visible/.hidden")).unwrap();
    fs::create_dir(p.join("outside")).unwrap();
    fs::write(p.join("scope/visible/ok.md"), "").unwrap();
    fs::write(p.join("scope/visible/.hidden/secret.md"), "").unwrap();
    fs::write(p.join("outside/private.md"), "").unwrap();
    symlink(p.join("outside"), p.join("scope/visible/link")).unwrap();
    let r = discover_with_options(&[p.join("scope")], &Options { depth: 4 }).unwrap();
    assert_eq!(r.suggestions.len(), 1);
    assert_eq!(r.omissions.hidden, 1);
    assert_eq!(r.omissions.symlinks, 1);
    assert_eq!(r.visited_directories.len(), 2);
}
