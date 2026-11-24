use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn create(
    mode: &str,
    destination: &Path,
    title: &str,
    sources: &[PathBuf],
) -> Result<PathBuf, String> {
    create_with_scope_size(mode, destination, title, sources, None)
}

pub fn create_with_scope_size(
    mode: &str,
    destination: &Path,
    title: &str,
    sources: &[PathBuf],
    scope_size: Option<&str>,
) -> Result<PathBuf, String> {
    if !matches!(mode, "scratch" | "plugin" | "migration") {
        return Err("Unknown authoring mode".to_owned());
    }
    let scope_note = match scope_size {
        None => String::new(),
        Some(size @ ("XS" | "S" | "M")) => format!("Proposed scope size: {size}.\n"),
        Some(_) => return Err("Unsupported --size value. Choose XS, S, or M.".into()),
    };
    if title.trim().is_empty() || title.chars().any(invalid_title_character) {
        return Err("Title must be nonempty and contain no control characters".to_owned());
    }
    match fs::symlink_metadata(destination) {
        Ok(_) => return Err("Output path already exists".to_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    if mode != "scratch" && sources.is_empty() {
        return Err("Select at least one source file with --source".to_owned());
    }
    if mode == "scratch" && !sources.is_empty() {
        return Err("Scratch does not accept implementation sources".to_owned());
    }

    let captured = crate::source_intake::capture_sources(sources)?;

    let baseline_text = if captured.is_empty() {
        None
    } else {
        let base = crate::output::destination_key(destination)?;
        if base.to_str().is_none() {
            return Err("Destination identity must use a UTF-8 path".into());
        }
        Some(crate::source_intake::encode(
            &crate::source_intake::make_baseline(mode, &base, captured),
        )?)
    };
    let title = title.trim();
    let mut text = format!(
        "# {title}\n\nMode: {mode}. Revision: draft-1. Status: draft. Owner acceptance: pending.\n{scope_note}\n\
## Scope\n\nOPEN: Define the actor, boundary, observables, and exclusions for {title}.\n\n\
## Terms and inputs\n\nOPEN: Define terms, input types, ranges, and invalid input handling.\n\n\
## Results and failures\n\nOPEN: Define outputs, effects, completion, and failure behavior.\n\n\
## Clauses\n\nOPEN: Write ID | actor must obligation clauses after resolving the required decisions.\n\n\
## Cases\n\nOPEN: Record inputs, expected results, clause IDs, and boundary or failure cases.\n\n\
## Decisions\n\nOPEN: Name the owner and record acceptance of the exact contract revision.\n"
    );
    if mode != "scratch" {
        text.push_str(
            "\n## Observed behavior\n\nOPEN: Inspect the selected baseline files. Separate observed behavior from approved requirements.\n\n\
## Proposed changes\n\nOPEN: Compare the current behavior with the required target behavior.\n",
        );
    }
    if mode == "migration" {
        text.push_str(
            "\n## Keep, change, or remove\n\nOPEN: Record an owner decision for each observed behavior in the selected scope.\n\n\
## Cutover and rollback\n\nOPEN: Define affected state, external effects, cutover checks, and rollback triggers or an explicit no-cutover decision.\n",
        );
    }

    if mode != "scratch" {
        text.push_str("\n## Source intake (non-normative)\n\n`baseline.json` records exact source revisions and bounded syntax observations, not requirements. Run `caac baseline check baseline.json` to recheck selected source identities. Runtime behavior, dependencies, and owner acceptance remain unverified.\n");
    }

    if let Some(parent) = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::create_dir(destination).map_err(|error| error.to_string())?;
    let contract_path = destination.join("CONTRACT.md");
    let mut contract_file = fs::File::create(&contract_path).map_err(|error| error.to_string())?;
    contract_file
        .write_all(text.as_bytes())
        .map_err(|error| error.to_string())?;
    if let Some(baseline_text) = baseline_text {
        fs::write(destination.join("baseline.json"), baseline_text).map_err(|e| e.to_string())?;
    }
    Ok(contract_path)
}

fn invalid_title_character(character: char) -> bool {
    (character as u32) < 32 || matches!(character, '\u{0085}' | '\u{2028}' | '\u{2029}')
}
