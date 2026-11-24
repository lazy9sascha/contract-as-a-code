use serde_json::{Value, json};
use std::process::Command;

fn schema() -> Value {
    serde_json::to_value(caac::candidate::entity_schema()).unwrap()
}

#[test]
fn cli_is_readonly_json_and_matches_public_api() {
    let directory = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(directory.path())
        .args(["candidate", "schema"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        schema()
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    assert_eq!(schema(), schema());
}

#[test]
fn entity_variants_are_closed_and_allow_only_declared_optional_fields() {
    let s = schema();
    assert_eq!(s["$schema"], "https://json-schema.org/draft/2020-12/schema");
    let variants = s["oneOf"].as_array().unwrap();
    let kinds: Vec<_> = variants
        .iter()
        .map(|v| v["properties"]["kind"]["const"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "definition",
            "clause",
            "operation",
            "exception",
            "case",
            "annex",
            "form",
            "sentence-part",
            "expression",
        ]
    );
    for variant in variants {
        assert_eq!(variant["additionalProperties"], false);
        let properties = variant["properties"].as_object().unwrap();
        let required = variant["required"].as_array().unwrap();
        let optional: &[&str] = match variant["properties"]["kind"]["const"].as_str().unwrap() {
            "clause" => &["annex_refs", "form_refs"],
            "exception" => &["sentence"],
            "case" => &["operation", "expression"],
            _ => &[],
        };
        assert_eq!(properties.len(), required.len() + optional.len());
        for key in properties.keys() {
            assert_eq!(
                required.contains(&json!(key)),
                !optional.contains(&key.as_str())
            );
        }
    }
    assert_eq!(variants[3]["properties"]["sentence"]["type"], "string");
    for nested in ["AttachmentRef", "FormField"] {
        let v = &s["$defs"][nested];
        assert_eq!(v["additionalProperties"], false);
        assert_eq!(
            v["properties"].as_object().unwrap().len(),
            v["required"].as_array().unwrap().len(),
        );
    }
    for nested in ["Applicability", "Effect"] {
        let v = &s["$defs"][nested]["oneOf"][0];
        assert_eq!(v["additionalProperties"], false);
        assert_eq!(
            v["properties"].as_object().unwrap().len(),
            v["required"].as_array().unwrap().len()
        );
    }
}

#[test]
fn case_target_schema_requires_one_nonnull_field() {
    let s = schema();
    let case = &s["oneOf"][4];
    assert_eq!(
        case["oneOf"],
        json!([{"required":["operation"]}, {"required":["expression"]}])
    );
    for target in ["operation", "expression"] {
        assert_eq!(case["properties"][target], json!({"type":"string"}));
    }
    let expression = &s["oneOf"][8];
    assert_eq!(
        expression["required"],
        json!(["kind", "id", "text", "inputs", "output", "clause_refs"])
    );
    assert_eq!(
        expression["properties"]["inputs"],
        s["oneOf"][2]["properties"]["inputs"]
    );
    assert_eq!(
        expression["properties"]["output"],
        s["oneOf"][2]["properties"]["output"]
    );
}

#[test]
fn scalar_bounds_and_custom_deserializer_maps_remain_structural() {
    let s = schema();
    assert_eq!(
        s["$defs"]["Literal"]["anyOf"],
        json!([
            {"type":"integer", "format":"int32", "minimum":i32::MIN, "maximum":i32::MAX},
            {"type":"boolean"}, {"type":"string"}
        ])
    );
    assert_eq!(s["$defs"]["Type"]["enum"], json!(["i32", "bool", "string"]));
    assert_eq!(
        s["$defs"]["Modality"]["enum"],
        json!(["obligation", "prohibition", "permission"])
    );
    assert_eq!(
        s["oneOf"][2]["properties"]["inputs"],
        json!({"type":"object", "additionalProperties":{"$ref":"#/$defs/Type"}})
    );
    assert_eq!(
        s["oneOf"][4]["properties"]["input"],
        json!({"type":"object", "additionalProperties":{"$ref":"#/$defs/Literal"}})
    );
    // No invented semantic constraints: references/blank text remain runtime concerns.
    assert_eq!(
        s["oneOf"][0]["properties"]["text"],
        json!({"type":"string"})
    );
}

#[test]
fn references_are_local_and_resolve() {
    fn visit(v: &Value, root: &Value) {
        match v {
            Value::Object(map) => {
                if let Some(reference) = map.get("$ref") {
                    let reference = reference.as_str().unwrap();
                    assert!(reference.starts_with("#/$defs/"));
                    assert!(root.pointer(&reference[1..]).is_some());
                }
                for child in map.values() {
                    visit(child, root);
                }
            }
            Value::Array(items) => {
                for item in items {
                    visit(item, root);
                }
            }
            _ => {}
        }
    }
    let s = schema();
    visit(&s, &s);
}

#[test]
fn schema_cli_rejects_options_and_help_is_discoverable() {
    for args in [
        vec!["candidate", "schema", "--output", "unused"],
        vec!["candidate", "schema", "entity"],
        vec!["candidate", "schema", "--kind", "candidate"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_caac"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    for args in [
        vec!["--help"],
        vec!["candidate", "--help"],
        vec!["candidate", "schema", "--help"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_caac"))
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("caac candidate schema")
        );
    }
}

#[test]
fn interview_points_to_entity_schema_without_marking_eof_success() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("base.json"),
        include_bytes!("../examples/candidate/draft.json"),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_caac"))
        .current_dir(directory.path())
        .args([
            "candidate",
            "interview",
            "--base",
            "base.json",
            "--answers",
            "journal.json",
            "--output",
            "candidate",
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("caac candidate schema")
    );
    assert!(!directory.path().join("candidate").exists());
}
