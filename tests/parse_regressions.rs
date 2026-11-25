use caac::codegen::{generate_markdown, parse_json};
use caac::parse::parse_markdown;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn parses_repository_contract_clauses_across_sections() {
    let schema = parse_markdown(include_str!("../CONTRACT.md")).unwrap();
    assert_eq!(schema.clauses.len(), 61);
    assert_eq!(schema.cases.len(), 29);
    assert!(schema.clauses.iter().any(|c| c.id == "CAAC-043"));
    assert_eq!(schema.clauses[0].actor, "The author");
    assert!(schema.clauses[0].obligation.starts_with("must define"));
}

#[test]
fn illustrative_and_nonnormative_content_cannot_supply_clauses() {
    let content = concat!(
        "# Test\n## Clauses\n",
        "<!--\nHIDDEN-001 | Server must erase everything.\n-->\n",
        "```text\nDEMO-001 | Server must erase everything.\n```\n",
        "~~~\nDEMO-002 | Server must erase everything.\n~~~\n",
        "REAL-001 | Server must respond.\n",
        "## Notes (non-normative)\nHIDDEN-002 | Server must stop.\n",
        "### Detail\nHIDDEN-003 | Server must stop.\n",
        "## More rules\nREAL-002 | Server must continue.\n",
        "REAL-003 | Server <!-- illustrative -->must finish.\n"
    );
    let parsed = parse_markdown(content).unwrap();
    assert_eq!(
        parsed
            .clauses
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        ["REAL-001", "REAL-002", "REAL-003"]
    );
    // The checker and parser agree about which clause definitions exist.
    let dir = tempdir().unwrap();
    let path = dir.path().join("contract.md");
    fs::write(&path, content).unwrap();
    assert!(caac::checker::check(&[path]).unwrap().is_empty());
}

#[test]
fn malformed_signatures_return_errors_instead_of_panics() {
    for signature in ["f)(", "f(", "f)", "f()garbage", "f())", "()"] {
        let content = format!("# Test\n## Operation signatures\n- `{signature}` -> `()`\n");
        assert!(parse_markdown(&content).is_err(), "{signature}");
    }
    let parsed =
        parse_markdown("# Test\n## Operation signatures\n- `f(x: (i32, i32))` -> `()`\n").unwrap();
    assert_eq!(parsed.operations[0].params, "x: (i32, i32)");
}

#[test]
fn malformed_input_does_not_create_output() {
    let dir = tempdir().unwrap();
    let input = dir.path().join("bad.md");
    let output = dir.path().join("out.json");
    fs::write(&input, "# Test\n## Operation signatures\n- `f)(` -> `()`\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_caac"))
        .arg("parse")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(!output.exists());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"));
}

#[test]
fn unclosed_exclusions_and_empty_documents_fail_closed() {
    for suffix in ["\n```\n", "\n<!--\n"] {
        assert!(parse_markdown(&format!("RULE-001 | Server must respond.{suffix}")).is_err());
    }
    assert!(parse_markdown("---\ntitle: Missing closure\n").is_err());
    assert!(parse_markdown("# Not a contract\nJust prose.\n").is_err());
}

#[test]
fn generated_markdown_round_trip_retains_schema_fields() {
    let schema = parse_json(r#"{"name":"Example","description":"","extends":["Base@1"],"operations":[{"name":"first","params":"","clauses":["RULE-001"]},{"name":"second","params":"","clauses":["RULE-001"]}],"clauses":[{"id":"RULE-001","actor":"The server","obligation":"must respond."}],"cases":[{"name":"pipe","input":"a|b","expected":"ok","clauses":["RULE-001"]}]}"#).unwrap();
    let markdown = generate_markdown(&schema).unwrap();
    let roundtrip = parse_markdown(&markdown).unwrap();
    assert_eq!(
        serde_json::to_value(roundtrip).unwrap(),
        serde_json::to_value(schema).unwrap()
    );
    assert!(parse_markdown(&markdown.replace("must respond.", "must stop.")).is_err());
}

#[test]
fn unmarked_case_tables_decode_escaped_cells_and_reject_ambiguous_rows() {
    let content = "# X\nRULE-001 | The server must respond.\n## Contract cases\n| Case | Input | Expected result | Clauses |\n| --- | --- | --- | --- |\n| pipes | a\\|b | path\\\\file | RULE-001 |\n";
    let schema = parse_markdown(content).unwrap();
    assert_eq!(schema.cases[0].input, "a|b");
    assert_eq!(schema.cases[0].expected, "path\\file");
    assert!(parse_markdown(&content.replace("a\\|b", "a|b")).is_err());
}

#[test]
fn unicode_line_breaks_match_checker_exclusions() {
    let content = "RULE-001 | Server must respond.\r\n## Notes (non-normative)\u{2028}HIDDEN-001 | Server must stop.\u{2028}## Rules\rRULE-002 | Server must continue.";
    assert_eq!(parse_markdown(content).unwrap().clauses.len(), 2);
}

#[test]
fn mixed_valid_and_malformed_clauses_fail_without_output() {
    let dir = tempdir().unwrap();
    for (index, malformed) in [
        "RULE-02 | Server must stop.",
        "RULE-002 Server must stop.",
        "RULE-002 | Server must not",
        "RULE-002 | Server must not.",
        "RULE-002 | Server must\t not ...",
        "RULE-002 | Server may .",
        "RULE-002 | Server must.",
    ]
    .iter()
    .enumerate()
    {
        let content = format!("RULE-001 | Server must respond.\n{malformed}\n");
        assert!(parse_markdown(&content).is_err(), "accepted {malformed}");
        let input = dir.path().join(format!("bad-{index}.md"));
        let output = dir.path().join(format!("out-{index}.json"));
        fs::write(&input, &content).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_caac"))
            .arg("parse")
            .arg(&input)
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1), "{malformed}");
        assert!(!output.exists(), "created output for {malformed}");
    }
    assert!(parse_markdown("RULE-001 | Server must not stop.\n").is_ok());
    assert!(parse_markdown("RULE-001 | Server must notify.\n").is_ok());
}

#[test]
fn sizing_guidance_references_method_rules_without_clause_count_quotas() {
    let schema = parse_markdown(include_str!("../CONTRACT.md")).unwrap();
    assert_eq!(schema.version, "0.1.4");
    for id in [
        "CAAC-009", "CAAC-044", "CAAC-045", "CAAC-046", "CAAC-047", "CAAC-048", "CAAC-049",
        "CAAC-050", "CAAC-051", "CAAC-052", "CAAC-053", "CAAC-054", "CAAC-055", "CAAC-056",
        "CAAC-057", "CAAC-058", "CAAC-059", "CAAC-060", "CAAC-061",
    ] {
        assert!(schema.clauses.iter().any(|clause| clause.id == id));
        assert!(
            schema
                .cases
                .iter()
                .any(|case| case.clauses.iter().any(|reference| reference == id))
        );
    }
    let skill = include_str!("../.agents/skills/caac/SKILL.md");
    assert!(skill.contains("CAAC-040 through CAAC-045"));
    assert!(!skill.contains("3-5 clauses"));
    assert!(!skill.contains("30+ clauses"));
    let quickstart = include_str!("../docs/quickstart.md");
    assert!(quickstart.contains("../CONTRACT.md#contract-sizing"));
    assert!(quickstart.contains("scope estimate in the Markdown scaffold"));
    assert!(quickstart.contains("candidate-authoring.md#presentation-and-boundaries"));
}

#[test]
fn sizing_instructions_refer_to_the_current_candidate_layouts() {
    let guide = include_str!("../docs/candidate-authoring.md");
    assert!(
        guide
            .lines()
            .any(|line| line == "## Presentation and boundaries")
    );

    let contract = include_str!("../CONTRACT.md");
    let (_, status) = contract
        .split_once("## Sizing implementation status (non-normative)\n")
        .expect("locate the method's CLI sizing note");
    let procedure = include_str!("../docs/scan-to-draft.md");
    let (_, step) = procedure
        .split_once("**Choose a proposed structure.**")
        .expect("locate the scan procedure's sizing step");
    let step = step
        .lines()
        .take_while(|line| {
            !line
                .split_once(". ")
                .is_some_and(|(number, _)| number.parse::<usize>().is_ok())
        })
        .collect::<Vec<_>>()
        .join("\n");
    for (file, text, link) in [
        (
            "CONTRACT.md",
            status.split("\n## ").next().unwrap(),
            "](docs/candidate-authoring.md#presentation-and-boundaries)",
        ),
        (
            "docs/scan-to-draft.md",
            step.as_str(),
            "](candidate-authoring.md#presentation-and-boundaries)",
        ),
    ] {
        let repeats_profile = text
            .split(|c: char| !c.is_ascii_alphanumeric())
            .any(|word| matches!(word, "XS" | "S" | "M" | "L" | "XL"));
        assert!(
            text.contains(link) && !repeats_profile,
            "{file}: link to docs/candidate-authoring.md#presentation-and-boundaries for CLI sizing support. Keep individual profiles in that guide."
        );
    }
}
