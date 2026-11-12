use std::fs;
use std::path::PathBuf;

/// Evaluate the completion gate for a contract.
/// Returns a list of unmet conditions.
pub fn evaluate_completion(files: &[PathBuf]) -> Vec<String> {
    let mut issues = Vec::new();

    // Read all files
    let mut all_content = String::new();
    for file in files {
        if let Ok(c) = fs::read_to_string(file) {
            all_content.push_str(&c);
            all_content.push('\n');
        }
    }

    // Parse clauses and cases from the combined content
    // (Simple heuristic: count clause definitions and case references)
    let clause_ids: Vec<String> = all_content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() == 2 {
                let id = parts[0].trim();
                if is_clause_id(id) {
                    return Some(id.to_owned());
                }
            }
            None
        })
        .collect();

    let case_clause_refs: Vec<String> = all_content
        .lines()
        .filter(|l| l.trim().starts_with('|') && !l.contains("---") && !l.contains("Case |"))
        .flat_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                parts[4]
                    .split(',')
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
                    .collect()
            } else {
                Vec::new()
            }
        })
        .collect();

    // Check 1: clauses exist
    if clause_ids.is_empty() {
        issues.push("No clauses defined.".to_owned());
    }

    // Check 2: all clauses have at least one case referencing them
    let referenced: std::collections::HashSet<&str> =
        case_clause_refs.iter().map(|s| s.as_str()).collect();
    for cid in &clause_ids {
        if !referenced.contains(cid.as_str()) {
            issues.push(format!("Clause {} has no acceptance case.", cid));
        }
    }

    // Check 3: open decisions
    let open_count = all_content
        .lines()
        .filter(|l| l.trim().starts_with("OPEN:"))
        .count();
    if open_count > 0 {
        issues.push(format!("{} open decision(s) remain.", open_count));
    }

    // Check 4: checker passes
    match crate::checker::check(files) {
        Ok(diagnostics) if !diagnostics.is_empty() => {
            issues.push(format!("{} syntax finding(s).", diagnostics.len()));
        }
        Err(e) => issues.push(format!("Check error: {}", e)),
        _ => {}
    }

    issues
}

fn is_clause_id(s: &str) -> bool {
    let mut parts = s.split('-').collect::<Vec<_>>();
    if parts.len() < 2 {
        return false;
    }
    let seq = parts.pop().unwrap();
    if seq.len() != 3 || !seq.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    parts.iter().all(|p| {
        let mut b = p.bytes();
        matches!(b.next(), Some(b'A'..=b'Z'))
            && b.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    })
}

/// Run the contract completion loop: check → repair → interview → re-check.
/// Evaluates completion gate on each iteration.
pub fn run_loop(files: &[PathBuf], sources: &[String], max_iterations: u32) -> i32 {
    let mut iteration = 0u32;

    loop {
        iteration += 1;
        println!("=== harness iteration {} ===", iteration);

        // Step 1: Check
        let diagnostics = match crate::checker::check(files) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                return 2;
            }
        };

        if diagnostics.is_empty() {
            println!("PASS: no syntax findings.");
        } else {
            println!("FAIL: {} finding(s):", diagnostics.len());
            for d in &diagnostics {
                println!("  {}:{}: {}: {}", d.path, d.line, d.code, d.message);
            }
        }

        // Step 2: Evaluate completion gate
        let issues = evaluate_completion(files);
        if issues.is_empty() {
            println!("\nCOMPLETION GATE: PASSED");
            println!("  - All clauses have acceptance cases.");
            println!("  - No open decisions remain.");
            println!("  - Syntax check passed.");
            println!("  - Ready for independent review.");
            return 0;
        } else {
            println!("\nCOMPLETION GATE: NOT PASSED");
            for issue in &issues {
                println!("  - {}", issue);
            }
        }

        if iteration >= max_iterations {
            println!("\nBUDGET EXHAUSTED: {} iteration(s) used.", iteration);
            return 1;
        }

        // Step 3: Interview (if open decisions)
        let has_open = issues.iter().any(|i| i.contains("open decision"));
        if has_open {
            let source_contents: Vec<String> = sources
                .iter()
                .filter_map(|p| fs::read_to_string(p).ok())
                .collect();
            let source_refs: Vec<&str> = source_contents.iter().map(|s| s.as_str()).collect();

            for file in files {
                let content = fs::read_to_string(file).unwrap_or_default();
                let open_lines: Vec<&str> = content
                    .lines()
                    .filter(|l| l.trim().starts_with("OPEN:"))
                    .collect();
                if !open_lines.is_empty() {
                    if !source_refs.is_empty() {
                        println!("\nSuggestions for {}:", file.display());
                        for ol in &open_lines {
                            let decision = ol.trim().trim_start_matches("OPEN:").trim();
                            if let Some(suggestion) =
                                crate::suggest::suggest_for_decision(decision, &source_refs)
                            {
                                println!("  Q: {}", decision);
                                println!("  Suggested: {}", suggestion);
                            }
                        }
                    }
                    crate::interview::run_interview(file, false);
                }
            }
        }

        // Step 4: Watch for changes
        println!("\nWaiting for file changes (Ctrl+C to stop)...");
        let mut last_mtimes: Vec<std::time::SystemTime> = files.iter().map(file_mtime).collect();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let current: Vec<std::time::SystemTime> = files.iter().map(file_mtime).collect();
            if current != last_mtimes {
                last_mtimes = current;
                break;
            }
        }
        println!("Files changed. Re-checking...");
    }
}

fn file_mtime(path: &PathBuf) -> std::time::SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}
