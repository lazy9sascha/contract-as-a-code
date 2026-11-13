use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const STATE_VERSION: u32 = 1;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize, Default)]
pub struct InterviewState {
    pub format_version: u32,
    pub file: String,
    pub revision: String,
    pub position: usize,
    pub resolved: Vec<String>,
    pub skipped: Vec<String>,
    pending: Option<PendingAnswer>,
}

#[derive(Serialize, Deserialize)]
struct PendingAnswer {
    question_id: String,
    before_revision: String,
    after_revision: String,
    content: String,
}

pub fn run_interview(path: &Path, auto: bool) -> i32 {
    run(path, auto, false)
}

pub fn run_resume(path: &Path) -> i32 {
    run(path, false, true)
}

fn run(path: &Path, auto: bool, resume: bool) -> i32 {
    match interview(path, auto, resume) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("ERROR: {error}");
            2
        }
    }
}

fn validate_answer(answer: &str) -> Result<(), String> {
    if answer
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{0085}' | '\u{2028}' | '\u{2029}'))
    {
        return Err("Answers must be a single line without control characters.".into());
    }
    if answer.contains("<!--") || answer.contains("-->") {
        return Err(
            "Answers cannot contain HTML comment delimiters that change contract visibility."
                .into(),
        );
    }
    Ok(())
}

fn interview(path: &Path, auto: bool, resume: bool) -> Result<(), String> {
    // Do not follow a file symlink when changing contracts or checkpoints.
    regular_file(path)?;
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    let state_path = state_path(&path);
    let mut content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut stored = match fs::symlink_metadata(&state_path) {
        Ok(_) => {
            regular_file(&state_path)?;
            Some(fs::read_to_string(&state_path).map_err(|e| e.to_string())?)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    let mut state = if let Some(json) = &stored {
        let state: InterviewState = serde_json::from_str(json)
            .map_err(|e| format!("Invalid or unsupported interview checkpoint: {e}"))?;
        if state.format_version != STATE_VERSION || state.file != path.to_string_lossy() {
            return Err(
                "Checkpoint version or contract identity mismatch; no files changed.".into(),
            );
        }
        state
    } else {
        if resume {
            return Err(format!(
                "No checkpoint found for {}. Run 'caac interview' first.",
                path.display()
            ));
        }
        InterviewState {
            format_version: STATE_VERSION,
            file: path.to_string_lossy().into_owned(),
            revision: digest(&content),
            ..Default::default()
        }
    };

    // A pending journal entry is durable before its contract replacement. Recovery
    // accepts only its exact before or after bytes, never a third revision.
    if let Some(pending) = &state.pending {
        if digest(&pending.content) != pending.after_revision
            || state.revision != pending.before_revision
        {
            return Err("Invalid pending checkpoint; no files changed.".into());
        }
        let line_index = state
            .position
            .checked_sub(1)
            .ok_or("Invalid pending answer position")?;
        let (after_start, after_end) = *line_spans(&pending.content)
            .get(line_index)
            .ok_or("Invalid pending answer position")?;
        let decision = &pending.content[after_start..after_end];
        let answer = decision
            .strip_prefix("DECISION: ")
            .ok_or("Pending answer is not a decision record")?;
        validate_answer(answer)?;
        let current = digest(&content);
        if current == pending.before_revision {
            let (start, end) = *line_spans(&content)
                .get(line_index)
                .ok_or("Invalid pending answer position")?;
            if !content[start..end].trim().starts_with("OPEN:") {
                return Err("Pending answer does not replace an OPEN decision".into());
            }
            let mut expected = content.clone();
            expected.replace_range(start..end, decision);
            if expected != pending.content {
                return Err("Pending answer changes content outside its decision line".into());
            }
            atomic_replace(&path, pending.content.as_bytes(), Some(content.as_bytes()))?;
            content = pending.content.clone();
        } else if current != pending.after_revision {
            return Err(format!(
                "Contract changed since the checkpoint; refusing to overwrite edits. Preserve and move {} aside, then run interview to start a new checkpoint.",
                state_path.display()
            ));
        }
        state.revision = pending.after_revision.clone();
        state.pending = None;
        save_state(&state_path, &state, &mut stored)?;
    }
    if digest(&content) != state.revision {
        return Err(format!(
            "Contract changed since the checkpoint; refusing to overwrite edits. Preserve and move {} aside, then run interview to start a new checkpoint.",
            state_path.display()
        ));
    }
    // Persist even an auto interview or an interview interrupted before an answer.
    save_state(&state_path, &state, &mut stored)?;

    let diagnostics = crate::checker::check(std::slice::from_ref(&path))?;
    let open_lines: Vec<usize> = diagnostics
        .iter()
        .filter(|d| d.code == "open-decision")
        .map(|d| d.line - 1)
        .collect();
    if fs::read_to_string(&path).map_err(|e| e.to_string())? != content {
        return Err("Contract changed while preparing interview; no answers applied.".into());
    }
    if open_lines.is_empty() {
        println!("No open decisions found in {}.", path.display());
        return Ok(());
    }
    println!(
        "Found {} open decision(s) in {}.",
        open_lines.len(),
        path.display()
    );
    println!("Answer each question, or type 'skip' to defer.\n");
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    for (index, line_index) in open_lines.iter().copied().enumerate() {
        let spans = line_spans(&content);
        let (start, end) = *spans
            .get(line_index)
            .ok_or("Contract line changed; refusing an ambiguous answer location.")?;
        let question = content[start..end]
            .trim()
            .strip_prefix("OPEN:")
            .ok_or("OPEN line mixes comments and text; put the decision on its own line.")?
            .trim()
            .to_owned();
        println!(
            "Decision {} of {}: {}",
            index + 1,
            open_lines.len(),
            question
        );
        let answer = if auto {
            println!("[auto mode: skipping]");
            "skip".to_owned()
        } else {
            print!("Your answer (or 'skip'): ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let mut answer = String::new();
            if reader.read_line(&mut answer).map_err(|e| e.to_string())? == 0 {
                println!("\nInput ended; unanswered decisions remain OPEN.");
                break;
            }
            answer.trim().to_owned()
        };
        if fs::read_to_string(&path).map_err(|e| e.to_string())? != content {
            return Err(
                "Contract changed while awaiting an answer; refusing to overwrite edits.".into(),
            );
        }
        validate_answer(&answer)?;
        state.position = line_index + 1;
        if answer.eq_ignore_ascii_case("skip") || answer.is_empty() {
            if !state.skipped.contains(&question) {
                state.skipped.push(question);
            }
            save_state(&state_path, &state, &mut stored)?;
            println!("  -> skipped\n");
            continue;
        }
        let before = digest(&content);
        let question_id = digest(&format!("{before}:{line_index}:{question}"));
        let mut updated = content.clone();
        updated.replace_range(start..end, &format!("DECISION: {answer}"));
        state
            .resolved
            .push(format!("{question_id}: {question} => {answer}"));
        state.skipped.retain(|skipped| skipped != &question);
        state.pending = Some(PendingAnswer {
            question_id,
            before_revision: before,
            after_revision: digest(&updated),
            content: updated.clone(),
        });
        save_state(&state_path, &state, &mut stored)?;
        atomic_replace(&path, updated.as_bytes(), Some(content.as_bytes()))?;
        content = updated;
        state.revision = digest(&content);
        state.pending = None;
        save_state(&state_path, &state, &mut stored)?;
        println!("  -> recorded\n");
    }
    println!(
        "Resolved: {}, Skipped: {}",
        state.resolved.len(),
        state.skipped.len()
    );
    Ok(())
}

fn digest(content: &str) -> String {
    Sha256::digest(content.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// Match the checker's physical-line rules while preserving every separator byte.
fn line_spans(content: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = 0;
    let mut iter = content.char_indices().peekable();
    while let Some((index, character)) = iter.next() {
        if matches!(
            character,
            '\n' | '\r'
                | '\u{000b}'
                | '\u{000c}'
                | '\u{001c}'
                | '\u{001d}'
                | '\u{001e}'
                | '\u{0085}'
                | '\u{2028}'
                | '\u{2029}'
        ) {
            spans.push((start, index));
            start = index + character.len_utf8();
            if character == '\r' && iter.peek().is_some_and(|(_, c)| *c == '\n') {
                start = iter.next().unwrap().0 + 1;
            }
        }
    }
    if start < content.len() {
        spans.push((start, content.len()));
    }
    spans
}

fn state_path(contract: &Path) -> PathBuf {
    let mut path = contract.to_path_buf();
    let mut name = contract.file_name().unwrap_or_default().to_os_string();
    name.push(".caac-state.json");
    path.set_file_name(name);
    path
}

fn regular_file(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "{} is not a regular file; refusing to follow or replace it.",
            path.display()
        ));
    }
    Ok(())
}

fn save_state(
    path: &Path,
    state: &InterviewState,
    stored: &mut Option<String>,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    atomic_replace(path, json.as_bytes(), stored.as_ref().map(|s| s.as_bytes()))?;
    *stored = Some(json);
    Ok(())
}

fn atomic_replace(path: &Path, bytes: &[u8], expected: Option<&[u8]>) -> Result<(), String> {
    if expected.is_none() {
        return crate::output::write_new(path, bytes);
    }
    let check_current = || -> Result<(), String> {
        match expected {
            Some(expected) => {
                regular_file(path)?;
                if fs::read(path).map_err(|e| e.to_string())? != expected {
                    return Err(format!(
                        "{} changed concurrently; refusing to overwrite it.",
                        path.display()
                    ));
                }
            }
            None => match fs::symlink_metadata(path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
                Ok(_) => {
                    return Err(format!(
                        "{} appeared concurrently; refusing to overwrite it.",
                        path.display()
                    ));
                }
            },
        }
        Ok(())
    };
    check_current()?;
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(format!(
        ".tmp-{}-{}",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let temporary = PathBuf::from(temporary);
    let mut created = false;
    let result = (|| -> Result<(), String> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary).map_err(|e| e.to_string())?;
        created = true;
        if expected.is_some() {
            file.set_permissions(fs::metadata(path).map_err(|e| e.to_string())?.permissions())
                .map_err(|e| e.to_string())?;
        }
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        check_current()?;
        if expected.is_some() {
            fs::rename(&temporary, path).map_err(|e| e.to_string())?;
        } else {
            // Create the first checkpoint without overwriting a concurrent creator.
            fs::hard_link(&temporary, path).map_err(|e| e.to_string())?;
            fs::remove_file(&temporary).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() && created {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_temp_creation_preserves_unrelated_existing_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("contract.md");
        fs::write(&path, b"original").unwrap();
        let sequence = TEMP_SEQUENCE.load(Ordering::Relaxed);
        let temporary = PathBuf::from(format!(
            "{}.tmp-{}-{}",
            path.display(),
            std::process::id(),
            sequence
        ));
        fs::write(&temporary, b"unrelated").unwrap();
        assert!(atomic_replace(&path, b"replacement", Some(b"original")).is_err());
        assert_eq!(fs::read(&temporary).unwrap(), b"unrelated");
        assert_eq!(fs::read(&path).unwrap(), b"original");
    }

    #[test]
    fn first_checkpoint_never_replaces_an_existing_target() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("checkpoint.json");
        fs::write(&path, b"another writer").unwrap();
        assert!(atomic_replace(&path, b"replacement", None).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"another writer");
    }
}
