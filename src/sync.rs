use std::fs;
use std::path::Path;
use std::time::Duration;

/// Regenerate artifacts from one authoritative schema. This is not reverse sync.
/// Existing outputs are rejected. Watch only updates files created by this invocation.
pub fn run_sync(
    input: &Path,
    output_code: Option<&Path>,
    output_docs: Option<&Path>,
    watch: bool,
    interval: u64,
) -> i32 {
    if output_code.is_none() && output_docs.is_none() {
        eprintln!("ERROR: at least one of --code or --docs is required");
        return 2;
    }
    if interval == 0 {
        eprintln!("ERROR: --interval must be greater than zero");
        return 2;
    }
    let outputs: Vec<&Path> = [output_code, output_docs].into_iter().flatten().collect();
    let mut keys = std::collections::HashSet::new();
    for output in &outputs {
        let key = match crate::output::destination_key(output) {
            Ok(key) => key,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 2;
            }
        };
        if keys
            .iter()
            .any(|other: &std::path::PathBuf| key.starts_with(other) || other.starts_with(&key))
        {
            eprintln!("ERROR: outputs must be distinct files, not ancestors of one another");
            return 2;
        }
        keys.insert(key);
        if let Err(e) = crate::output::require_absent(output) {
            eprintln!("ERROR: {e}");
            return 2;
        }
    }
    let mut previous: Vec<Vec<u8>> = Vec::new();
    loop {
        let content = match fs::read_to_string(input) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("ERROR: cannot read {}: {e}", input.display());
                return 2;
            }
        };
        let schema = match crate::codegen::parse_json(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 1;
            }
        };
        // Validate before any output is published, including docs-only syncs.
        if let Err(e) = crate::codegen::validate_schema(&schema) {
            eprintln!("ERROR: {e}");
            return 1;
        }
        let code = match crate::codegen::generate_rust_trait(&schema) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return 1;
            }
        };
        let mut generated = Vec::new();
        if output_code.is_some() {
            generated.push(code.into_bytes());
        }
        if output_docs.is_some() {
            match crate::codegen::generate_markdown(&schema) {
                Ok(markdown) => generated.push(markdown.into_bytes()),
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    return 1;
                }
            }
        }
        // Preflight the whole output set before changing any member.
        for (index, output) in outputs.iter().enumerate() {
            if previous.is_empty() {
                if let Err(e) = crate::output::require_absent(output) {
                    eprintln!("ERROR: {e}");
                    return 2;
                }
            } else {
                let unchanged = fs::symlink_metadata(output).is_ok_and(|m| m.file_type().is_file())
                    && fs::read(output).is_ok_and(|bytes| bytes == previous[index]);
                if !unchanged {
                    eprintln!(
                        "ERROR: output changed outside this sync session: {}",
                        output.display()
                    );
                    return 2;
                }
            }
        }
        for (index, output) in outputs.iter().enumerate() {
            let result = if previous.is_empty() {
                crate::output::write_new(output, &generated[index])
            } else {
                crate::output::replace_owned(output, &previous[index], &generated[index])
            };
            if let Err(e) = result {
                eprintln!(
                    "ERROR: {e}. Earlier outputs in this operation may already be published."
                );
                return 2;
            }
            println!("Synced: {}", output.display());
        }
        previous = generated;
        if !watch {
            println!("Sync complete (schema to outputs only).");
            return 0;
        }
        println!("Watching for schema changes (Ctrl+C to stop)...");
        loop {
            std::thread::sleep(Duration::from_secs(interval));
            match fs::read_to_string(input) {
                Ok(current) if current == content => {}
                _ => break,
            }
        }
    }
}
