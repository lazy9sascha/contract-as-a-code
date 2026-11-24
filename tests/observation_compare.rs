use serde_json::{Value, json};
use std::{fs, process::Command};

fn compare(before: &str, after: &str) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("before.json"), before).unwrap();
    fs::write(dir.path().join("after.json"), after).unwrap();
    Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(dir.path())
        .args([
            "compare-observations",
            "--before",
            "before.json",
            "--after",
            "after.json",
        ])
        .output()
        .unwrap()
}

fn run(cases: Value) -> Value {
    json!({
        "format": "caac-observation-run-v1",
        "oracle": {"id": "reference", "revision": "sha256:1234"},
        "scope": {"paths": ["module.rs"], "profile": "v1"},
        "cases": cases,
    })
}

fn case(id: &str, actual: Value, expected: Value) -> Value {
    json!({
        "id": id,
        "input": {"args": ["case"], "fixture_tree": {"a.txt": "original"}},
        "expected": expected,
        "actual": actual,
    })
}

fn invoke(before: &Value, after: &Value) -> std::process::Output {
    compare(&before.to_string(), &after.to_string())
}

#[test]
fn classifies_each_transition_and_retains_raw_process_and_tree_differences() {
    let expected = json!({
        "exit_code": 0,
        "stdout": {"results": ["ok", {"a/b~c": 2}]},
        "stderr": "",
        "fixture_tree": {"a.txt": "done", "missing_is_not_null": null},
    });
    let mut old = expected.clone();
    old["stdout"]["results"][1]["a/b~c"] = json!(1);
    old["fixture_tree"]
        .as_object_mut()
        .unwrap()
        .remove("missing_is_not_null");
    let mut new = expected.clone();
    new["exit_code"] = json!(1);
    new["fixture_tree"]["a.txt"] = json!("changed");
    let before = run(json!([
        case("fixed", old.clone(), expected.clone()),
        case("regressed", expected.clone(), expected.clone()),
        case("same", expected.clone(), expected.clone()),
        case("different", old, expected.clone()),
    ]));
    let after = run(json!([
        case("different", new.clone(), expected.clone()),
        case("same", expected.clone(), expected.clone()),
        case("regressed", new, expected.clone()),
        case("fixed", expected.clone(), expected),
    ]));
    let result = invoke(&before, &after);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["format"], "caac-observation-comparison-v1");
    assert_eq!(
        report["counts"],
        json!({"fixed": 1, "regressed": 1, "still-matches": 1, "still-differs": 1})
    );
    let cases = report["cases"].as_array().unwrap();
    let fixed = cases.iter().find(|case| case["id"] == "fixed").unwrap();
    let differences = fixed["before_vs_expected"].as_array().unwrap();
    let stdout = differences
        .iter()
        .find(|d| d["path"] == "/stdout/results/1/a~1b~0c")
        .unwrap();
    assert_eq!(stdout["left"]["raw_json"], "1");
    assert_eq!(stdout["right"]["raw_json"], "2");
    let missing = differences
        .iter()
        .find(|d| d["path"] == "/fixture_tree/missing_is_not_null")
        .unwrap();
    assert_eq!(missing["left"], json!({"present": false}));
    assert_eq!(
        missing["right"],
        json!({"present": true, "raw_json": "null"})
    );
    assert!(fixed["after_vs_expected"].as_array().unwrap().is_empty());
    let different = cases.iter().find(|case| case["id"] == "different").unwrap();
    assert_eq!(different["status"], "still-differs");
    assert!(!different["before_vs_after"].as_array().unwrap().is_empty());
}

#[test]
fn rejects_mismatched_preconditions_before_printing_any_report() {
    let original = run(json!([case("id", json!(1), json!(1))]));
    for mode in 0..9 {
        let mut changed = original.clone();
        match mode {
            0 => changed["oracle"]["id"] = json!("other"),
            1 => changed["oracle"]["revision"] = json!("other"),
            2 => changed["scope"]["profile"] = json!("other"),
            3 => changed["cases"][0]["id"] = json!("other"),
            4 => changed["cases"][0]["input"]["fixture_tree"]["a.txt"] = json!("other"),
            5 => changed["cases"][0]["expected"] = json!(2),
            6 => {
                let duplicate = changed["cases"][0].clone();
                changed["cases"].as_array_mut().unwrap().push(duplicate);
            }
            7 => {
                changed["cases"][0].as_object_mut().unwrap().remove("input");
            }
            _ => changed["extra"] = json!(true),
        }
        let result = invoke(&original, &changed);
        assert_eq!(result.status.code(), Some(2), "mode {mode}: {result:?}");
        assert!(result.stdout.is_empty(), "mode {mode}: {result:?}");
    }
}

#[test]
fn duplicate_json_keys_at_any_depth_are_rejected() {
    let valid = run(json!([case("id", json!({"stdout": {"x": 1}}), json!(1))])).to_string();
    for invalid in [
        valid.replacen(
            "\"format\":",
            "\"format\":\"caac-observation-run-v1\",\"format\":",
            1,
        ),
        valid.replacen("\"x\":1", "\"x\":1,\"x\":1", 1),
    ] {
        assert_ne!(valid, invalid);
        let result = compare(&valid, &invalid);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn preserves_exact_numeric_tokens_without_rounding() {
    let template = r#"{"format":"caac-observation-run-v1","oracle":{"id":"ref","revision":"rev"},"scope":{"s":true},"cases":[{"id":"x","input":{"n":INPUT},"expected":{"n":EXPECTED},"actual":{"n":ACTUAL}}]}"#;
    let make = |input: &str, expected: &str, actual: &str| {
        template
            .replace("INPUT", input)
            .replace("EXPECTED", expected)
            .replace("ACTUAL", actual)
    };
    let before = make(
        "18446744073709551616",
        "1.0000000000000001",
        "1.0000000000000002",
    );
    let after = make(
        "18446744073709551616",
        "1.0000000000000001",
        "1.0000000000000001",
    );
    let result = compare(&before, &after);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["cases"][0]["status"], "fixed");
    assert_eq!(
        report["cases"][0]["before_vs_expected"][0]["left"]["raw_json"],
        "1.0000000000000002"
    );
    assert_eq!(
        report["cases"][0]["before_vs_expected"][0]["right"]["raw_json"],
        "1.0000000000000001"
    );
    let changed_input = make(
        "18446744073709551617",
        "1.0000000000000001",
        "1.0000000000000001",
    );
    let result = compare(&before, &changed_input);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let spelling = compare(&make("1", "1e0", "1.0"), &make("1", "1e0", "1e0"));
    assert!(spelling.status.success());
    let report: Value = serde_json::from_slice(&spelling.stdout).unwrap();
    assert_eq!(report["cases"][0]["status"], "fixed");
}

#[test]
fn strings_and_object_order_compare_structurally_and_output_is_terminal_safe() {
    let before = r#"{"format":"caac-observation-run-v1","oracle":{"id":"ref","revision":"rev"},"scope":{"a":1,"b":2},"cases":[{"id":"x","input":{"text":"\u0061","obj":{"a":1,"b":2}},"expected":null,"actual":"\u001b[31m한국어"}]}"#;
    let after = r#"{"format":"caac-observation-run-v1","oracle":{"revision":"rev","id":"ref"},"scope":{"b":2,"a":1},"cases":[{"id":"x","input":{"obj":{"b":2,"a":1},"text":"a"},"expected":null,"actual":null}]}"#;
    let result = compare(before, after);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        !result
            .stdout
            .iter()
            .any(|byte| *byte == 0x1b || *byte >= 0x80)
    );
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["cases"][0]["status"], "fixed");
}

#[test]
fn repeated_long_paths_exceed_the_report_budget_before_output() {
    let key = "p".repeat(65_536);
    let mut expected = json!({});
    expected[&key] = json!(vec![0; 1_100]);
    let mut actual = json!({});
    actual[&key] = json!(vec![1; 1_100]);
    let before = run(json!([case("long", actual, expected.clone())]));
    let after = run(json!([case("long", expected.clone(), expected)]));
    let result = invoke(&before, &after);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("report size limit"));
}
