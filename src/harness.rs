use std::fs;
use std::path::PathBuf;

/// Evaluate a syntax and case-reference precheck, not semantic completion.
/// Returns a list of unmet conditions.
pub fn evaluate_completion(files: &[PathBuf]) -> Vec<String> {
    let mut issues = Vec::new();

    let mut clause_ids = Vec::new();
    let mut case_clause_refs = std::collections::HashSet::new();
    for file in files {
        let content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(error) => {
                issues.push(format!("{}: {error}", file.display()));
                continue;
            }
        };
        match precheck_schema(&content) {
            Ok(schema) => {
                clause_ids.extend(schema.clauses.into_iter().map(|clause| clause.id));
                for case in schema.cases {
                    case_clause_refs.extend(
                        case.clauses
                            .into_iter()
                            .map(|reference| reference.trim_start_matches('@').to_owned()),
                    );
                }
            }
            Err(error) => issues.push(format!("{}: {error}", file.display())),
        }
    }
    if clause_ids.is_empty() {
        issues.push("No clauses defined.".to_owned());
    }
    for id in clause_ids {
        if !case_clause_refs.contains(&id) {
            issues.push(format!("Clause {id} has no acceptance case."));
        }
    }
    match crate::checker::check(files) {
        Ok(diagnostics) => {
            let open_count = diagnostics
                .iter()
                .filter(|d| d.code == "open-decision")
                .count();
            if open_count > 0 {
                issues.push(format!("{open_count} open decision(s) remain."));
            }
            if !diagnostics.is_empty() {
                issues.push(format!("{} syntax finding(s).", diagnostics.len()));
            }
        }
        Err(error) => issues.push(format!("Check error: {error}")),
    }
    issues
}

// Verify metadata consistency first, but derive coverage from the visible body.
// This prevents a rendering bug from letting metadata conceal extra obligations.
fn precheck_schema(content: &str) -> Result<crate::codegen::ContractSchema, String> {
    let verified = crate::parse::parse_markdown(content)?;
    if content.starts_with("<!-- caac-schema-v1:") {
        let (_, body) = content
            .split_once('\n')
            .ok_or("metadata has no visible contract body")?;
        visible_precheck_schema(body, &verified)
    } else {
        Ok(verified)
    }
}

fn visible_precheck_schema(
    body: &str,
    verified: &crate::codegen::ContractSchema,
) -> Result<crate::codegen::ContractSchema, String> {
    let visible = crate::parse::parse_markdown(body)?;
    let expected: std::collections::BTreeSet<_> =
        verified.clauses.iter().map(|clause| &clause.id).collect();
    let observed: std::collections::BTreeSet<_> =
        visible.clauses.iter().map(|clause| &clause.id).collect();
    if expected != observed {
        return Err(
            "visible clause IDs differ from embedded schema; review the rendered contract".into(),
        );
    }
    Ok(visible)
}

/// Run the contract completion loop: check → repair → interview → re-check.
/// Evaluates the pre-review check on each iteration.
pub fn run_loop(files: &[PathBuf], sources: &[String], max_iterations: u32) -> i32 {
    if max_iterations == 0 {
        eprintln!("ERROR: --max-iterations must be greater than zero.");
        return 2;
    }
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
            println!("PASS: no syntax findings. Semantic review pending.");
        } else {
            println!("FAIL: {} finding(s):", diagnostics.len());
            for d in &diagnostics {
                println!("  {}:{}: {}: {}", d.path, d.line, d.code, d.message);
            }
        }

        // Step 2: Evaluate completion gate
        let issues = evaluate_completion(files);
        if issues.is_empty() {
            println!("\nPRE-REVIEW CHECK: PASSED");
            println!("  - All clauses have textual case references.");
            println!("  - No open decisions remain.");
            println!("  - Syntax check passed.");
            println!(
                "  - Semantic review, executed case results, and owner acceptance remain pending."
            );
            return 0;
        } else {
            println!("\nPRE-REVIEW CHECK: NOT PASSED");
            for issue in &issues {
                println!("  - {}", issue);
            }
        }

        if iteration >= max_iterations {
            println!("\nBUDGET EXHAUSTED: {} iteration(s) used.", iteration);
            return 1;
        }

        // Step 3: Interview (if open decisions)
        let before: Vec<_> = files.iter().map(fs::read).collect();
        let has_open = diagnostics.iter().any(|d| d.code == "open-decision");
        if has_open {
            let source_contents: Vec<String> = sources
                .iter()
                .filter_map(|p| fs::read_to_string(p).ok())
                .collect();
            let source_refs: Vec<&str> = source_contents.iter().map(|s| s.as_str()).collect();

            for file in files {
                let path = file.display().to_string();
                let decisions: Vec<_> = diagnostics
                    .iter()
                    .filter(|d| d.code == "open-decision" && d.path == path)
                    .collect();
                if !decisions.is_empty() {
                    if !source_refs.is_empty() {
                        println!("\nSuggestions for {}:", file.display());
                        for diagnostic in &decisions {
                            let decision = diagnostic.message.as_str();
                            if let Some(suggestion) =
                                crate::suggest::suggest_for_decision(decision, &source_refs)
                            {
                                println!("  Q: {}", decision);
                                println!("  Suggested: {}", suggestion);
                            }
                        }
                    }
                    let result = crate::interview::run_interview(file, false);
                    if result != 0 {
                        return result;
                    }
                }
            }
        }

        // Our own changes require an immediate recheck, not another external save.
        let changed =
            files
                .iter()
                .zip(before)
                .any(|(path, previous)| match (previous, fs::read(path)) {
                    (Ok(previous), Ok(current)) => previous != current,
                    _ => true,
                });
        if changed {
            continue;
        }
        println!(
            "\nNO PROGRESS: decisions or manual repair remain. Update the contract and rerun."
        );
        return 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_precheck_rejects_extra_rendered_obligations_and_uses_visible_cases() {
        let verified: crate::codegen::ContractSchema = serde_json::from_value(serde_json::json!({
            "name": "Example",
            "clauses": [{"id": "RULE-001", "actor": "Service", "obligation": "must return a value."}],
            "cases": [{"name": "one", "input": "input", "expected": "value", "clauses": ["RULE-001"]}]
        })).unwrap();
        let body = "RULE-001 | Service must return a value.\n";
        let visible = visible_precheck_schema(body, &verified).unwrap();
        assert!(
            visible.cases.is_empty(),
            "metadata cases must not supply visible coverage"
        );
        let injected = format!("{body}RULE-999 | Service must erase data.\n");
        assert!(
            visible_precheck_schema(&injected, &verified)
                .unwrap_err()
                .contains("visible clause IDs differ")
        );
    }
}
