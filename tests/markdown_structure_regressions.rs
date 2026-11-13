use caac::{checker, parse::parse_markdown};
use std::{fs, process::Command};

fn diagnostics(content: &str) -> Vec<checker::Diagnostic> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("contract.md");
    fs::write(&path, content).unwrap();
    checker::check(&[path]).unwrap()
}

#[test]
fn ast_comment_spans_preserve_unmasked_bytes_and_unicode() {
    let text = "# Café\nRULE-001 | Café <!-- explanation -->must respond.\nRULE-002 | Café must re<!-- detail -->turn.\n";
    let schema = parse_markdown(text).unwrap();
    assert_eq!(schema.name, "Café");
    assert_eq!(schema.clauses[0].actor, "Café");
    assert_eq!(schema.clauses[0].obligation, "must respond.");
    assert_eq!(schema.clauses[1].obligation, "must return.");
    assert!(diagnostics(text).is_empty());
}

#[test]
fn ast_heading_titles_preserve_source_escapes_and_entities() {
    let text = "# Café &amp; A\\*B\nRULE-001 | Server must respond.\n";
    assert_eq!(parse_markdown(text).unwrap().name, "Café &amp; A\\*B");
    assert!(diagnostics(text).is_empty());
}

#[test]
fn unsupported_peer_headings_cannot_extend_nonnormative_exclusion() {
    for peer in ["## Rules ##\n", "Rules\n-----\n", "## **Rules**\n"] {
        let text = format!(
            "# Test\nRULE-001 | Server must respond.\n## Notes (non-normative)\nExample.\n{peer}RULE-002 | Server must stop.\n"
        );
        assert!(parse_markdown(&text).is_err(), "{peer}");
        assert!(
            diagnostics(&text)
                .iter()
                .any(|d| d.code == "unsupported-markdown"),
            "{peer}"
        );
    }
}

#[test]
fn unsupported_structure_cannot_hide_normative_content() {
    for body in [
        "    HIDDEN-001 | Server must erase data.\n",
        "> HIDDEN-001 | Server must erase data.\n",
        "- ```text\n  HIDDEN-001 | Server must erase data.\n  ```\n",
        "<div>\nHIDDEN-001 | Server must erase data.\n</div>\n",
        "<!-- explanation -->HIDDEN-001 | Server must erase data.\n",
        "<span>HIDDEN-001 | Server must erase data.</span>\n",
    ] {
        let text = format!("# Test\nRULE-001 | Server must respond.\n\n{body}");
        assert!(parse_markdown(&text).is_err(), "{body}");
        assert!(!diagnostics(&text).is_empty(), "{body}");
    }
}

#[test]
fn library_fence_closure_and_original_locations_are_required() {
    for separator in [
        "\n", "\r\n", "\r", "\u{000b}", "\u{000c}", "\u{001c}", "\u{001d}", "\u{001e}", "\u{0085}",
        "\u{2028}", "\u{2029}",
    ] {
        let text = [
            "RULE-001 | Server must respond.",
            "```text",
            "HIDDEN-001 | Server must stop.",
            "",
        ]
        .join(separator);
        assert!(parse_markdown(&text).is_err());
        assert!(
            diagnostics(&text)
                .iter()
                .any(|d| d.code == "unclosed-fence" && d.line == 2)
        );
        let closed = format!("{text}```{separator}RULE-002 | Server must continue.");
        assert_eq!(parse_markdown(&closed).unwrap().clauses.len(), 2);
        assert!(diagnostics(&closed).is_empty());
    }
}

#[test]
fn known_metadata_prefix_keeps_frontmatter_and_diagnostic_offsets() {
    let schema = caac::codegen::parse_json(r#"{"name":"Example","clauses":[{"id":"RULE-001","actor":"Server","obligation":"must respond."}]}"#).unwrap();
    let generated = caac::codegen::generate_markdown(&schema).unwrap();
    assert!(diagnostics(&generated).is_empty());
    let injected = format!("{generated}\n## More rules\nOPEN: Choose behavior.\n");
    let line = injected
        .lines()
        .position(|line| line.starts_with("OPEN:"))
        .unwrap()
        + 1;
    assert!(
        diagnostics(&injected)
            .iter()
            .any(|d| d.code == "open-decision" && d.line == line)
    );
    let bad = generated.replacen("caac-schema-v1:", "caac-schema-v1:Z", 1);
    assert!(
        diagnostics(&bad)
            .iter()
            .any(|d| d.code == "invalid-metadata" && d.line == 1)
    );
    assert!(parse_markdown(&bad).is_err());
}

#[test]
fn unsupported_structure_fails_cli_before_output() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("contract.md");
    let output = directory.path().join("schema.json");
    fs::write(
        &input,
        "# Test\nRULE-001 | Server must respond.\n\n<div>\nRULE-002 | Server must stop.\n</div>\n",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_caac"))
        .arg("parse")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(!output.exists());
}

#[test]
fn ast_code_spans_drive_markers_but_do_not_suppress_references() {
    for line in [
        "``must`` is a token.",
        "``text `must` text`` is a token.",
        "`<!--` is a token.",
    ] {
        let text = format!("# Test\nRULE-001 | Server must respond.\n{line}\n");
        assert!(diagnostics(&text).is_empty(), "{line}");
        assert!(parse_markdown(&text).is_ok(), "{line}");
    }
    for line in ["`Server must stop.", "\\`Server must stop.\\`"] {
        let text = format!("# Test\nRULE-001 | Server must respond.\n{line}\n");
        assert!(
            diagnostics(&text)
                .iter()
                .any(|d| d.code == "unnumbered-obligation"),
            "{line}"
        );
        assert!(parse_markdown(&text).is_err(), "{line}");
    }
    let quoted = "# Test\nRULE-001 | Server must display `@MISSING-001`.\n";
    assert!(diagnostics(quoted).iter().any(|d| d.code == "unknown-id"));
    assert!(parse_markdown(quoted).is_err());
}

#[test]
fn multiline_inline_code_is_rejected_unless_explicitly_nonnormative() {
    let raw =
        "# Test\nRULE-001 | Server must respond.\n`example\nRULE-002 | Server must stop.\n`\n";
    assert!(parse_markdown(raw).is_err());
    assert!(
        diagnostics(raw)
            .iter()
            .any(|d| d.code == "unsupported-markdown")
    );
    let excluded = raw.replace("`example", "## Examples (non-normative)\n`example");
    assert_eq!(parse_markdown(&excluded).unwrap().clauses.len(), 1);
    assert!(diagnostics(&excluded).is_empty());
}

#[test]
fn literal_code_comments_are_preserved_but_incomplete_comments_fail() {
    let literal = "# Test\nRULE-001 | Server must retain `<!-- literal -->` text.\n";
    assert_eq!(
        parse_markdown(literal).unwrap().clauses[0].obligation,
        "must retain `<!-- literal -->` text."
    );
    assert!(diagnostics(literal).is_empty());
    for suffix in [
        "Text <!-- missing closure",
        "## Notes (non-normative)\nText <!-- missing closure",
    ] {
        let text = format!("# Test\nRULE-001 | Server must respond.\n{suffix}\n");
        assert!(parse_markdown(&text).is_err());
        assert!(
            diagnostics(&text)
                .iter()
                .any(|d| d.code == "unclosed-comment")
        );
    }
}

#[test]
fn case_comments_preserve_exact_cells_after_escaped_pipes() {
    let directory = tempfile::tempdir().unwrap();
    for (prefix_index, (prefix, decoded_prefix)) in [
        ("ascii", "ascii"),
        ("é", "é"),
        ("한", "한"),
        ("𐀀", "𐀀"),
        (r"a\|b", "a|b"),
    ]
    .into_iter()
    .enumerate()
    {
        for (cell_index, (cell, expected)) in [
            ("x<!-- note -->y", "xy"),
            ("`<!-- literal -->`", "`<!-- literal -->`"),
            (r"a\|b<!-- note -->c", "a|bc"),
            (r"`a\|b`<!-- note -->c", "`a|b`c"),
        ]
        .into_iter()
        .enumerate()
        {
            let text = format!(
                "# Demo\nRULE-001 | Service must return a value.\n## Contract cases\n| Case | Input | Expected result | Clauses |\n| --- | --- | --- | --- |\n| one | {prefix} | {cell} | RULE-001 |\n"
            );
            let schema = parse_markdown(&text).unwrap();
            assert_eq!(schema.cases[0].input, decoded_prefix);
            assert_eq!(schema.cases[0].expected, expected);
            assert!(diagnostics(&text).is_empty());
            let input = directory
                .path()
                .join(format!("{prefix_index}-{cell_index}.md"));
            let output = directory
                .path()
                .join(format!("{prefix_index}-{cell_index}.json"));
            fs::write(&input, &text).unwrap();
            let result = Command::new(env!("CARGO_BIN_EXE_caac"))
                .arg("parse")
                .arg(&input)
                .arg("--output")
                .arg(&output)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let recovered =
                caac::codegen::parse_json(&fs::read_to_string(output).unwrap()).unwrap();
            assert_eq!(recovered, schema);
        }
    }
}

#[test]
fn unicode_case_comments_never_cut_utf8_or_corrupt_following_code() {
    for unicode in ["é", "한", "𐀀"] {
        for code in ["", " `must`"] {
            let text = format!(
                "# Demo\nRULE-001 | Service must return a value.\n## Contract cases\n| Case | Input | Expected result | Clauses |\n| --- | --- | --- | --- |\n| one | input | a\\|{unicode}<!-- note -->c{code} | RULE-001 |\n"
            );
            let schema = parse_markdown(&text).unwrap();
            assert_eq!(schema.cases[0].expected, format!("a|{unicode}c{code}"));
            assert!(diagnostics(&text).is_empty());
        }
    }
}
