use std::fs;
use std::path::Path;
use std::time::Duration;

/// Bidirectional sync: regenerate code and docs from a schema.
/// If --watch is set, poll for changes and regenerate.
pub fn run_sync(
    input: &Path,
    output_code: Option<&Path>,
    output_docs: Option<&Path>,
    watch: bool,
    interval: u64,
) -> i32 {
    let mut last_mtime = file_mtime(input);

    loop {
        // Read schema
        let content = match fs::read_to_string(input) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("ERROR: cannot read {}: {}", input.display(), e);
                return 2;
            }
        };
        let schema = match crate::codegen::parse_json(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                return 1;
            }
        };

        // Generate code
        if let Some(out) = output_code {
            let code = crate::codegen::generate_rust_trait(&schema);
            if let Some(parent) = out.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(out, &code) {
                eprintln!("ERROR: cannot write {}: {}", out.display(), e);
                return 2;
            }
            println!("Synced code: {}", out.display());
        }

        // Generate docs
        if let Some(out) = output_docs {
            let md = crate::codegen::generate_markdown(&schema);
            if let Some(parent) = out.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(out, &md) {
                eprintln!("ERROR: cannot write {}: {}", out.display(), e);
                return 2;
            }
            println!("Synced docs: {}", out.display());
        }

        if !watch {
            println!("Sync complete.");
            return 0;
        }

        // Watch mode: poll for changes
        println!("Watching for changes (Ctrl+C to stop)...");
        loop {
            std::thread::sleep(Duration::from_secs(interval));
            let current = file_mtime(input);
            if current != last_mtime {
                last_mtime = current;
                println!("\n--- file changed, re-syncing ---");
                break;
            }
        }
    }
}

fn file_mtime(path: &Path) -> std::time::SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}
