use caac::patterns::run_gen_with_slots;
use serde_json::json;

#[test]
fn identifier_injection_is_rejected_without_output() {
    for name in [
        "ok() {}\npub fn injected",
        "ok\rpub fn injected",
        "fn",
        "",
        "_",
        "a::{b}",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("generated.rs");
        assert_eq!(
            run_gen_with_slots(
                "FN-OPERATION",
                &json!({"fn_name": name}).to_string(),
                &output
            ),
            2
        );
        assert!(!output.exists());
    }
}

#[test]
fn unsupported_and_unknown_slots_are_not_silently_ignored() {
    for slots in [
        json!({"fn_name":"compose", "f_type":"String", "g_type":"bool"}),
        json!({"fn_name":"compose", "typo":"value"}),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("generated.rs");
        assert_eq!(
            run_gen_with_slots("ABS-COMPOSE", &slots.to_string(), &output),
            2
        );
        assert!(!output.exists());
    }
}

#[test]
fn bound_generation_requires_all_used_placeholders() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("generated.rs");
    assert_eq!(
        run_gen_with_slots(
            "ABS-INTERFACE",
            &json!({"trait_name":"Store", "method_name":"get"}).to_string(),
            &output
        ),
        2
    );
    assert!(!output.exists());
}

#[test]
fn supported_identifiers_and_types_are_rendered() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("generated.rs");
    assert_eq!(run_gen_with_slots("ABS-INTERFACE", &json!({"trait_name":"Store", "method_name":"get", "input_type":"String", "output_type":"Option<String>"}).to_string(), &output), 0);
    let code = std::fs::read_to_string(output).unwrap();
    assert!(code.contains("fn get(&self, input: String) -> Option<String>"));
    assert!(!code.contains("{input_type}"));
}

#[test]
fn type_injection_and_nested_placeholder_injection_are_rejected() {
    for value in ["String); }\npub fn injected() {} //", "{output_type}"] {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("generated.rs");
        assert_eq!(run_gen_with_slots("ABS-INTERFACE", &json!({"trait_name":"Store", "method_name":"get", "input_type":value, "output_type":"String"}).to_string(), &output), 2);
        assert!(!output.exists());
    }
}

#[test]
fn cli_rejects_unsafe_bindings_and_preserves_existing_output() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("generated.rs");
    std::fs::write(&output, "existing output").unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["pattern", "gen", "FN-OPERATION", "--slots"])
        .arg(json!({"fn_name":"safe() {}\npub fn injected"}).to_string())
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("invalid or unsupported"));
    assert_eq!(std::fs::read_to_string(output).unwrap(), "existing output");
}

#[test]
fn discovery_marks_unsupported_bindings_and_scaffold_scope() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_caac"))
        .args(["pattern", "list"])
        .output()
        .unwrap();
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("ABS-COMPOSE - connect typed operations (scaffold only)"));
    assert!(text.contains("Unsupported slots: f_type, g_type"));
}
