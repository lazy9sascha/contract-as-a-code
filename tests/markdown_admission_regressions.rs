use caac::{codegen, parse};

#[test]
fn unnumbered_obligations_and_open_decisions_cannot_be_demoted_to_description() {
    for extra in [
        "Server must retain audit logs.",
        "OPEN: Select the data retention limit.",
    ] {
        let text = format!("# Demo\nRULE-001 | Server must respond.\n{extra}\n");
        assert!(
            parse::parse_markdown(&text).is_err(),
            "silently accepted {extra}"
        );
    }
}

#[test]
fn conflicting_term_definitions_are_rejected() {
    let text = "# Demo\n## Terms and inputs\nAn `Limit` is 10.\nAn `Limit` is 100.\n## Clauses\nRULE-001 | Server must respond.\n";
    assert!(parse::parse_markdown(text).is_err());
}

#[test]
fn explicit_base_references_survive_markdown_roundtrip() {
    let text = "# Demo\n## Extends\n- Base@1.0\n## Clauses\nRULE-001 | Server must respond.\n";
    let schema = parse::parse_markdown(text).unwrap();
    assert_eq!(schema.extends, ["Base@1.0"]);
    let rendered = codegen::generate_markdown(&schema).unwrap();
    assert_eq!(
        parse::parse_markdown(&rendered).unwrap().extends,
        schema.extends
    );
}

#[test]
fn admitted_numbered_identifier_roundtrips() {
    let text = "# Demo\nRULE2-001 | Server must respond.\n";
    let schema = parse::parse_markdown(text).unwrap();
    let rendered = codegen::generate_markdown(&schema).unwrap();
    assert_eq!(parse::parse_markdown(&rendered).unwrap(), schema);
}

#[test]
fn explicit_shared_clause_application_is_preserved() {
    let text = "# Demo\n## Operation signatures\n- `first()` -> `()`\n- `second()` -> `()`\n## Clauses\n### first\nRULE-001 | Server must respond.\n### second\nApplies: @RULE-001\n";
    let schema = parse::parse_markdown(text).unwrap();
    assert_eq!(schema.operations[1].clauses, ["RULE-001"]);
    assert!(
        parse::parse_markdown(&text.replace("Applies: @RULE-001", "Applies: @MISSING-001"))
            .is_err()
    );
}

#[test]
fn case_named_case_is_not_a_table_header() {
    let text = "# Demo\nRULE-001 | Server must respond.\n## Contract cases\n| Case | Input | Expected result | Clauses |\n| --- | --- | --- | --- |\n| Case | request | response | RULE-001 |\n";
    let schema = parse::parse_markdown(text).unwrap();
    assert_eq!(schema.cases.len(), 1);
    assert_eq!(schema.cases[0].name, "Case");
}

#[test]
fn unsupported_exclusion_syntax_is_not_discarded() {
    let text = "# Demo\nRULE-001 | Server must respond.\n## Exclusions\nPerformance guarantees are outside scope.\n";
    assert!(parse::parse_markdown(text).is_err());
}

#[test]
fn both_term_articles_preserve_definitions_and_unsupported_terms_fail_closed() {
    let text = "# Demo\n## Terms and inputs\nA `Limit` is 10.\n## Clauses\nRULE-001 | Server must enforce Limit.\n";
    assert_eq!(parse::parse_markdown(text).unwrap().terms["Limit"], "10.");
    assert!(parse::parse_markdown(&text.replace("A `Limit` is", "The Limit equals")).is_err());
}

#[test]
fn unsupported_conditions_and_unknown_operation_sections_are_rejected() {
    for text in [
        "# Demo\nRULE-001 | Server must retain data.\nExcept when the owner requests erasure.\n",
        "# Demo\n## Operation signatures\n- `save()` -> `()`\n## Clauses\n### save\n#### Exception: owner erasure\nRULE-001 | Server may erase data.\n",
        "# Demo\n## Operation signatures\n- `save()` -> `()`\n## Clauses\n### load\nRULE-001 | Server must retain data.\n",
    ] {
        assert!(parse::parse_markdown(text).is_err(), "accepted: {text}");
    }
}

#[test]
fn undefined_case_references_are_rejected_before_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("contract.md");
    let output = dir.path().join("schema.json");
    std::fs::write(&input, "# Demo\nRULE-001 | Server must respond.\n## Contract cases\n| missing | input | result | MISSING-001 |\n").unwrap();
    assert_eq!(parse::run_parse(&input, &output), 1);
    assert!(!output.exists());
}

#[test]
fn standalone_invariant_heading_does_not_require_an_operation() {
    let text = "# Demo\n## Clauses\n### Invariants\nRULE-001 | Server must respond.\n";
    let schema = parse::parse_markdown(text).unwrap();
    assert_eq!(schema.clauses.len(), 1);
    assert!(schema.operations.is_empty());
}

#[test]
fn references_to_frontmatter_clauses_cannot_enter_a_schema() {
    let text =
        "---\nRULE-001 | Server must retain logs.\n---\nRULE-002 | Server must apply @RULE-001.\n";
    assert!(parse::parse_markdown(text).is_err());
}

#[test]
fn unsupported_operation_and_case_notation_fails_before_output() {
    for section in [
        "## Operation signatures\n- `read()` -> `()`\n* `delete()` -> `()`\n",
        "## Operation signatures\n+ `read()` -> `()`\n",
        "## Operation signatures\nread(input) -> output\n",
        "## Contract cases\nCase | Input | Expected | Clauses\n--- | --- | --- | ---\none | input | value | RULE-001\n",
        "## Contract cases\nThe input produces a value.\n",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("contract.md");
        let output = dir.path().join("schema.json");
        std::fs::write(
            &input,
            format!("# Demo\nRULE-001 | Server must respond.\n{section}"),
        )
        .unwrap();
        assert_eq!(parse::run_parse(&input, &output), 1, "{section}");
        assert!(!output.exists());
    }
}

#[test]
fn all_clause_reference_positions_use_the_checker_boundary() {
    for sentence in [
        "Under @MISSING-001, Server must respond.",
        "Server must apply @MISSING-001.",
        "Server must display `@MISSING-001`.",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("contract.md");
        let output = dir.path().join("schema.json");
        std::fs::write(&input, format!("# Demo\nRULE-001 | {sentence}\n")).unwrap();
        assert_eq!(parse::run_parse(&input, &output), 1, "{sentence}");
        assert!(!output.exists());
    }
    let text = "# Demo\nRULE-001 | Under @RULE-002, Server must respond.\nRULE-002 | Server must retain logs.\n";
    let schema = parse::parse_markdown(text).unwrap();
    assert_eq!(schema.clauses[0].actor, "Under @RULE-002, Server");
    let md = codegen::generate_markdown(&schema).unwrap();
    assert_eq!(parse::parse_markdown(&md).unwrap(), schema);
}
