use caac::codegen::{
    ContractSchema, generate_markdown, generate_rust_trait, merge_schemas, parse_json,
};
use caac::extract::extract_rust;
use std::process::Command;

fn schema() -> ContractSchema {
    parse_json(r#"{"name":"Demo","version":"3.0","description":"Example","terms":{"Account":"authenticated actor"},"operations":[{"name":"read","params":"&mut self, value: Vec<i64>","returns":"Result<i64, String>","clauses":["DEMO-001"]}],"clauses":[{"id":"DEMO-001","actor":"Account service","obligation":"must return data."},{"id":"DEMO-002","actor":"Service","obligation":"must encrypt data."}],"cases":[{"name":"one","input":"x","expected":"y","clauses":["DEMO-001"]}],"exclusions":["network"],"extends":["Base@1.0"]}"#).unwrap()
}

#[test]
fn rust_roundtrip_preserves_every_schema_field() {
    let s = schema();
    let code = generate_rust_trait(&s).unwrap();
    assert!(code.contains("fn read(&mut self, value: Vec<i64>)"));
    assert_eq!(extract_rust(&code).unwrap(), s);
    assert_eq!(merge_schemas(std::slice::from_ref(&s)).unwrap(), s);
}

#[test]
fn rejects_body_drift_and_unmarked_rust_instead_of_partial_extraction() {
    let code = generate_rust_trait(&schema()).unwrap();
    assert!(extract_rust(&code.replace("fn read(", "fn changed(")).is_err());
    for code in [
        "pub trait Demo {\nfn f(&self) {}\nfn lost(&self);\n}",
        "pub trait Demo {\nfn f(&self, cb: fn(i32) -> i32) -> bool;\n}",
        "pub trait Demo {\nfn f(&self);\n}",
        "// caac-schema-v2:00\n",
    ] {
        assert!(extract_rust(code).is_err());
    }
}

#[test]
fn prose_cannot_become_rust_items() {
    let mut s = schema();
    s.description = "safe\npub fn injected() {}\r\nsecond".into();
    s.clauses[0].obligation = "must return data.".into();
    let code = generate_rust_trait(&s).unwrap();
    assert!(!code.lines().any(|l| l.starts_with("pub fn")));
    assert_eq!(extract_rust(&code).unwrap(), s);
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("generated.rs");
    std::fs::write(&file, code).unwrap();
    let result = Command::new("rustc")
        .args(["--edition=2024", "--crate-type=lib"])
        .arg(&file)
        .arg("--out-dir")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn unsupported_schema_syntax_and_ambiguity_fail_closed() {
    for params in [
        "cb: fn(i32) -> i32",
        "&mut self, &self",
        "x: i64); fn injected(",
        "x: i64, x: i64",
    ] {
        let mut s = schema();
        s.operations[0].params = params.into();
        assert!(generate_rust_trait(&s).is_err());
    }
    let mut s = schema();
    s.clauses[0].id = "X\npub fn injected() {}".into();
    assert!(generate_rust_trait(&s).is_err());
    let s = schema();
    assert!(merge_schemas(&[s.clone(), s]).is_err());
    assert!(parse_json(r#"{"name":"Demo","unknown_obligation":"must preserve"}"#).is_err());
}

#[test]
fn markdown_emits_shared_clause_once_and_inheritance() {
    let mut s = schema();
    let mut op = s.operations[0].clone();
    op.name = "other".into();
    s.operations.push(op);
    let md = generate_markdown(&s).unwrap();
    assert_eq!(md.matches("DEMO-001 | ").count(), 1);
    assert!(md.contains("Applies: @DEMO-001"));
    assert!(md.contains("- Base@1.0"));
    assert_eq!(
        caac::codegen::embedded_schema(&md, "<!-- caac-schema-v1:", " -->")
            .unwrap()
            .unwrap(),
        s
    );
}

#[test]
fn filled_inheritance_pattern_compiles() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("pattern.rs");
    assert_eq!(
        caac::patterns::run_gen_with_slots(
            "ABS-INHERIT",
            r#"{"base_trait":"Base","extended_trait":"Child","method_name":"f"}"#,
            &file
        ),
        0
    );
    let result = Command::new("rustc")
        .args(["--edition=2024", "--crate-type=lib"])
        .arg(&file)
        .arg("--out-dir")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn malformed_metadata_is_rejected_without_panics() {
    for input in [
        "// caac-schema-v1:0",
        "// caac-schema-v1:xx",
        "// caac-schema-v1:ff",
        "// caac-schema-v1:7b7d",
    ] {
        assert!(extract_rust(input).is_err());
    }
}

#[test]
fn rust2024_keywords_and_lone_underscore_are_rejected() {
    for name in ["gen", "_", "async", "type", "Self", "try", "yield"] {
        let mut s = schema();
        s.name = name.into();
        assert!(generate_rust_trait(&s).is_err(), "trait {name}");
        let mut s = schema();
        s.operations[0].name = name.into();
        assert!(generate_rust_trait(&s).is_err(), "method {name}");
    }
}

#[test]
fn markdown_table_body_escapes_content_independently_of_metadata() {
    let mut s = schema();
    s.cases[0].name = "pipe | slash \\".into();
    s.cases[0].input = "a|b\nnext".into();
    s.cases[0].expected = "x\\y".into();
    let md = generate_markdown(&s).unwrap();
    let body = md.split_once('\n').unwrap().1;
    assert!(body.contains("| pipe \\| slash \\\\ | a\\|b<br>next | x\\\\y | DEMO-001 |"));
    assert!(!body.contains("| a|b"));
    assert!(body.contains("## Extends\n\n- Base@1.0"));
}

#[test]
fn type_validation_bounds_depth_without_panicking() {
    let mut s = schema();
    s.operations[0].returns = format!("{}i64{}", "Vec<".repeat(100), ">".repeat(100));
    assert!(generate_rust_trait(&s).is_err());
}

#[test]
fn merging_different_names_is_not_silent() {
    let a = schema();
    let mut b = schema();
    b.name = "Second".into();
    b.clauses.clear();
    b.operations.clear();
    b.cases.clear();
    assert!(merge_schemas(&[a, b]).is_err());
}

#[test]
fn authoritative_line_injection_rejected_before_any_markdown_output() {
    let dir = tempfile::tempdir().unwrap();
    for separator in ["\n", "\r", "\u{0085}", "\u{2028}", "\u{2029}", "\u{000c}"] {
        let mut s = schema();
        s.clauses[0].obligation =
            format!("must respond.{separator}RULE-999 | Service must erase all data.");
        assert!(generate_markdown(&s).is_err());
        let input = dir.path().join("in.json");
        let output = dir.path().join("out.md");
        std::fs::write(&input, serde_json::to_string(&s).unwrap()).unwrap();
        let r = Command::new(env!("CARGO_BIN_EXE_caac"))
            .arg("md")
            .arg(&input)
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!r.status.success());
        assert!(!output.exists());
    }
    for field in ["name", "version", "actor", "term", "exclude", "extends"] {
        let mut s = schema();
        let bad = "safe\nRULE-999 | Service must erase all data.".to_owned();
        match field {
            "name" => s.name = bad,
            "version" => s.version = bad,
            "actor" => s.clauses[0].actor = bad,
            "term" => {
                s.terms.insert("x".into(), bad);
            }
            "exclude" => s.exclusions.push(bad),
            "extends" => s.extends.push(bad),
            _ => unreachable!(),
        }
        assert!(generate_markdown(&s).is_err(), "{field}");
    }
}

#[test]
fn description_and_inline_html_cannot_hide_or_add_visible_clauses() {
    let mut s = schema();
    s.description = "```\n<!--\nRULE-999 | Service must erase all data.\n## Clauses".into();
    s.clauses[0].obligation = "must return data <!-- hidden -->.".into();
    let md = generate_markdown(&s).unwrap();
    let body = md.split_once('\n').unwrap().1;
    assert!(body.contains("## Description (non-normative)"));
    assert!(body.contains("> \\`\\`\\`"));
    assert!(body.contains(r"> \<\!--"));
    assert!(!body.contains("\nRULE-999 |"));
    assert!(body.contains(r"DEMO-001 | Account service must return data \<\!-- hidden --\>."));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("contract.md");
    std::fs::write(&path, md).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_caac"))
        .arg("check")
        .arg("--json")
        .arg(&path)
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stdout));
}

#[test]
fn term_names_cannot_hide_normative_sections_with_html_comments() {
    for name in ["<!--", "-->", "`hidden", "x>y"] {
        let mut s = schema();
        s.terms.insert(name.into(), "definition".into());
        assert!(generate_markdown(&s).is_err(), "term {name}");
    }
}

#[test]
fn duplicate_json_terms_are_rejected_in_sources_and_embedded_metadata() {
    for second in ["10", "100"] {
        let json = format!(r#"{{"name":"Demo","terms":{{"Limit":"10","Limit":"{second}"}}}}"#);
        assert!(parse_json(&json).is_err());
        let hex: String = json.bytes().map(|b| format!("{b:02x}")).collect();
        assert!(extract_rust(&format!("// caac-schema-v1:{hex}\n")).is_err());
        assert!(caac::parse::parse_markdown(&format!("<!-- caac-schema-v1:{hex} -->\n")).is_err());
    }
}

#[test]
fn malformed_clause_shape_cannot_generate_artifacts() {
    for (actor, obligation) in [
        ("Server", ""),
        ("", "must respond."),
        ("Server", "mayhem"),
        ("Server", "must not."),
        ("Server", "may ."),
        ("Server must", "may respond."),
    ] {
        let mut s = schema();
        s.clauses[0].actor = actor.into();
        s.clauses[0].obligation = obligation.into();
        assert!(generate_markdown(&s).is_err(), "{actor} {obligation}");
        assert!(generate_rust_trait(&s).is_err(), "{actor} {obligation}");
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("schema.json");
        let markdown = dir.path().join("contract.md");
        let rust = dir.path().join("contract.rs");
        std::fs::write(&input, serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(caac::codegen::run_md(&[&input], &markdown), 1);
        assert_eq!(caac::codegen::run_gen(&[&input], &rust), 1);
        assert!(!markdown.exists());
        assert!(!rust.exists());
    }
    let mut s = schema();
    s.operations[0].name = "Invariants".into();
    assert!(generate_markdown(&s).is_err());
}

#[test]
fn supported_receiver_forms_compile_as_rust_2024() {
    for params in [
        "",
        "self",
        "&self",
        "&mut self",
        "& self",
        "& mut self",
        "   ",
        "value: i64",
        "&self, value: i64",
    ] {
        let mut s = schema();
        s.operations[0].params = params.into();
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("generated.rs");
        std::fs::write(&file, generate_rust_trait(&s).unwrap()).unwrap();
        let output = Command::new("rustc")
            .args(["--edition=2024", "--crate-type=lib"])
            .arg(&file)
            .arg("--out-dir")
            .arg(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{params}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn borrowed_return_lifetime_elision_is_checked_before_output() {
    for params in [
        "self",
        "self, left: &str, right: &str",
        "self, values: &Vec<&str>",
    ] {
        let mut s = schema();
        s.operations[0].params = params.into();
        s.operations[0].returns = "&str".into();
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("schema.json");
        let output = dir.path().join("contract.rs");
        std::fs::write(&input, serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(caac::codegen::run_gen(&[&input], &output), 1, "{params}");
        assert!(!output.exists());
    }
    for params in [
        "self, value: & str",
        "self, value: &str",
        "&self, left: &str, right: &str",
        "&mut self",
        "value: &str",
    ] {
        let mut s = schema();
        s.operations[0].params = params.into();
        s.operations[0].returns = "&str".into();
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("generated.rs");
        std::fs::write(&file, generate_rust_trait(&s).unwrap()).unwrap();
        let output = Command::new("rustc")
            .args(["--edition=2024", "--crate-type=lib"])
            .arg(&file)
            .arg("--out-dir")
            .arg(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{params}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn syn_syntax_does_not_expand_the_supported_signature_profile() {
    for (params, returns) in [
        ("self /* hidden */", "()"),
        ("#[cfg(any())] value: i64", "()"),
        ("r#value: i64", "()"),
        ("mut value: i64", "()"),
        ("ref value: i64", "()"),
        ("mut self", "()"),
        ("self: Box<Self>", "()"),
        ("&'static self", "()"),
        ("value: (i64, i64)", "()"),
        ("value: (i64)", "()"),
        ("value: Vec<3>", "()"),
        ("value: Vec<>", "()"),
        ("value: Self<i64>", "()"),
        ("value: Self::Assoc", "()"),
        ("value: Vec::<i64>", "()"),
        ("value: Box<i64>::Item", "()"),
        ("value: ::std::string::String", "()"),
        ("value: i64", "i64 where Self: Sized"),
        ("value: i64", "i64 /* hidden */"),
        ("value: i64", "&'static str"),
        ("value: i64", "i64; fn injected()"),
    ] {
        let mut s = schema();
        s.operations[0].params = params.into();
        s.operations[0].returns = returns.into();
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("schema.json");
        let rust = dir.path().join("contract.rs");
        let markdown = dir.path().join("contract.md");
        std::fs::write(&input, serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(
            caac::codegen::run_gen(&[&input], &rust),
            1,
            "{params} -> {returns}"
        );
        assert_eq!(
            caac::codegen::run_md(&[&input], &markdown),
            1,
            "{params} -> {returns}"
        );
        assert!(!rust.exists());
        assert!(!markdown.exists());
    }
}

#[test]
fn parser_budget_rejects_deep_or_large_input_before_output() {
    for returns in [
        format!("{}str", "&".repeat(2000)),
        format!("{}i64{}", "Vec<".repeat(65), ">".repeat(65)),
        "X".repeat(4097),
    ] {
        let mut s = schema();
        s.operations[0].returns = returns;
        assert!(generate_rust_trait(&s).is_err());
        assert!(generate_markdown(&s).is_err());
    }
}

#[test]
fn unknown_or_reinterpreted_clause_references_cannot_generate_artifacts() {
    for (actor, obligation) in [
        ("Under @MISSING-001, Service", "must respond."),
        ("Service", "must apply @MISSING-001."),
        ("Service", "must display `@MISSING-001`."),
        ("Service", "must display @MISSING-001_more."),
        ("Service", "must display @DEMO-002_more."),
    ] {
        let mut s = schema();
        s.clauses[0].actor = actor.into();
        s.clauses[0].obligation = obligation.into();
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("schema.json");
        let rust = dir.path().join("contract.rs");
        let markdown = dir.path().join("contract.md");
        std::fs::write(&input, serde_json::to_vec(&s).unwrap()).unwrap();
        assert_eq!(caac::codegen::run_gen(&[&input], &rust), 1);
        assert_eq!(caac::codegen::run_md(&[&input], &markdown), 1);
        assert!(!rust.exists());
        assert!(!markdown.exists());
    }
    let mut s = schema();
    s.clauses[0].actor = "Under @DEMO-002, Service".into();
    s.clauses[0].obligation = "must apply `@DEMO-002`.".into();
    let md = generate_markdown(&s).unwrap();
    assert_eq!(caac::parse::parse_markdown(&md).unwrap(), s);
    assert_eq!(extract_rust(&generate_rust_trait(&s).unwrap()).unwrap(), s);
}
