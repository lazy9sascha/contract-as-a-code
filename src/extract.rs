use crate::codegen::ContractSchema;
use std::fs;
use std::path::Path;

/// Extract exact CAAC-generated schemas. Unmarked Rust is deliberately unsupported:
/// signatures alone cannot recover invariants, cases, acceptance or source authority.
pub fn extract_rust(content: &str) -> Result<ContractSchema, String> {
    let schema = crate::codegen::embedded_schema(content, "// caac-schema-v1:", "")?
        .ok_or("unmarked Rust cannot be extracted losslessly; use a versioned CAAC-generated artifact (arbitrary Rust extraction is unsupported)")?;
    let expected = crate::codegen::generate_rust_trait(&schema)?;
    if content != expected {
        return Err("generated Rust differs from its embedded schema; reconcile changes in the schema and regenerate".into());
    }
    Ok(schema)
}

pub fn run_extract(input: &Path, output: &Path) -> i32 {
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: cannot read {}: {}", input.display(), e);
            return 2;
        }
    };
    let schema = match extract_rust(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return 1;
        }
    };
    let json = match serde_json::to_string_pretty(&schema) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("ERROR: cannot serialize: {e}");
            return 2;
        }
    };
    match crate::output::write_new(output, json.as_bytes()) {
        Ok(()) => {
            println!("Extracted: {}", output.display());
            0
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            2
        }
    }
}
