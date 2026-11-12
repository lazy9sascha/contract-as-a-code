use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Default)]
pub struct InterviewState {
    pub file: String,
    pub position: usize,
    pub resolved: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn run_interview(path: &Path, auto: bool) -> i32 {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: cannot read {}: {}", path.display(), e);
            return 2;
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let open_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.trim().starts_with("OPEN:"))
        .map(|(i, _)| i)
        .collect();

    if open_indices.is_empty() {
        println!("No open decisions found in {}.", path.display());
        return 0;
    }

    let state_path = state_path(path);
    let mut state = load_state(&state_path).unwrap_or_default();
    state.file = path.display().to_string();

    println!(
        "Found {} open decision(s) in {}.",
        open_indices.len(),
        path.display()
    );
    println!("Answer each question, or type 'skip' to defer.\n");

    let mut new_lines = lines.clone();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    for (idx, &line_idx) in open_indices.iter().enumerate() {
        let question = lines[line_idx].trim().trim_start_matches("OPEN:").trim();
        println!(
            "Decision {} of {}: {}",
            idx + 1,
            open_indices.len(),
            question
        );
        print!("Your answer (or 'skip'): ");
        io::stdout().flush().unwrap();

        if auto {
            println!("[auto mode: skipping]");
            state.skipped.push(question.to_owned());
            continue;
        }

        let mut answer = String::new();
        if reader.read_line(&mut answer).is_err() {
            break;
        }
        let answer = answer.trim();

        if answer.eq_ignore_ascii_case("skip") || answer.is_empty() {
            state.skipped.push(question.to_owned());
            println!("  -> skipped\n");
        } else {
            new_lines[line_idx] = Box::leak(format!("DECISION: {}", answer).into_boxed_str());
            state.resolved.push(format!("{} => {}", question, answer));
            println!("  -> recorded\n");
        }
        save_state(&state_path, &state);
    }

    // Write back if any changes
    if new_lines != lines {
        let new_content = new_lines.join("\n") + "\n";
        if let Err(e) = fs::write(path, new_content) {
            eprintln!("ERROR: cannot write {}: {}", path.display(), e);
            return 2;
        }
        println!("Contract updated: {}", path.display());
    }

    println!();
    println!(
        "Resolved: {}, Skipped: {}",
        state.resolved.len(),
        state.skipped.len()
    );
    if !state.skipped.is_empty() {
        println!("Skipped decisions remain as OPEN: lines.");
    }

    0
}

pub fn run_resume(path: &Path) -> i32 {
    let state_path = state_path(path);
    match load_state(&state_path) {
        Some(state) => {
            println!("Resuming {} from position {}.", state.file, state.position);
            println!(
                "Previously resolved: {}, skipped: {}",
                state.resolved.len(),
                state.skipped.len()
            );
            run_interview(path, false)
        }
        None => {
            eprintln!(
                "ERROR: no state file found for {}. Run 'caac interview' first.",
                path.display()
            );
            2
        }
    }
}

fn state_path(contract: &Path) -> PathBuf {
    let mut p = contract.to_path_buf();
    let name = p
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    p.set_file_name(format!("{}.caac-state.json", name));
    p
}

fn load_state(path: &Path) -> Option<InterviewState> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_state(path: &Path, state: &InterviewState) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, json);
    }
}
