use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn create(
    mode: &str,
    destination: &Path,
    title: &str,
    sources: &[PathBuf],
) -> Result<PathBuf, String> {
    if !matches!(mode, "scratch" | "plugin" | "migration") {
        return Err("Unknown authoring mode".to_owned());
    }
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

    let mut seen = HashSet::new();
    let mut source_digests = Vec::new();
    for source in sources {
        let resolved_source = fs::canonicalize(source).map_err(|error| error.to_string())?;
        if !seen.insert(resolved_source.clone()) {
            continue;
        }
        let bytes = fs::read(&resolved_source).map_err(|error| error.to_string())?;
        let digest = Sha256::digest(&bytes);
        let digest = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        source_digests.push((resolved_source, digest));
    }

    let title = title.trim();
    let mut text = format!(
        "# {title}\n\nMode: {mode}. Revision: draft-1. Status: draft. Owner acceptance: pending.\n\n\
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
    if !source_digests.is_empty() {
        let destination_absolute =
            fs::canonicalize(destination).map_err(|error| error.to_string())?;
        let baseline = source_digests
            .iter()
            .map(|(source, digest)| {
                json!({
                    "path": relative_path(&destination_absolute, source),
                    "sha256": digest,
                })
            })
            .collect::<Vec<_>>();
        let baseline_value = json!({
            "mode": mode,
            "files": baseline,
            "execution": "not-run",
            "owner_acceptance": "pending",
        });
        let mut baseline_text =
            serde_json::to_string_pretty(&baseline_value).map_err(|error| error.to_string())?;
        baseline_text.push('\n');
        fs::write(destination.join("baseline.json"), baseline_text)
            .map_err(|error| error.to_string())?;
    }
    Ok(contract_path)
}

fn invalid_title_character(character: char) -> bool {
    (character as u32) < 32 || matches!(character, '\u{0085}' | '\u{2028}' | '\u{2029}')
}

fn relative_path(base: &Path, target: &Path) -> String {
    let base_components = base.components().collect::<Vec<_>>();
    let target_components = target.components().collect::<Vec<_>>();
    let shared = base_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in shared..base_components.len() {
        relative.push("..");
    }
    for component in &target_components[shared..] {
        relative.push(component.as_os_str());
    }
    if relative.as_os_str().is_empty() {
        ".".to_owned()
    } else {
        relative.display().to_string()
    }
}
