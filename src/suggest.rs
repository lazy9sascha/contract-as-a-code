/// Generate a suggestion for an open decision based on source code.
pub fn suggest_for_decision(decision: &str, sources: &[&str]) -> Option<String> {
    let lower = decision.to_lowercase();

    // Collect source snippets
    let mut source_text = String::new();
    for src in sources {
        source_text.push_str(src);
        source_text.push('\n');
    }

    // Heuristic suggestions based on keywords
    if (lower.contains("input domain") || lower.contains("input type"))
        && let Some(params) = extract_params(&source_text)
    {
        return Some(format!("Based on source: input is {}", params));
    }
    if (lower.contains("output") || lower.contains("result"))
        && let Some(ret) = extract_return_type(&source_text)
    {
        return Some(format!("Based on source: output is {}", ret));
    }
    if lower.contains("actor") || lower.contains("responsible") {
        return Some("Based on source: the queue/processor is the actor.".to_owned());
    }
    if lower.contains("error") || lower.contains("failure") {
        return Some(
            "Based on source: errors include invalid input, not found, conflict.".to_owned(),
        );
    }

    None
}

fn extract_params(source: &str) -> Option<String> {
    // Look for function signatures with parameters
    for line in source.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("export ")
            || trimmed.starts_with("function "))
            && let Some(paren) = trimmed.find('(')
            && let Some(close) = trimmed[paren..].find(')')
        {
            let params = &trimmed[paren + 1..paren + close];
            if !params.is_empty() && params != "self" && params != "&self" {
                return Some(params.to_owned());
            }
        }
    }
    None
}

fn extract_return_type(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains("->")
            && (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn "))
            && let Some(arrow) = trimmed.find("->")
        {
            let ret = trimmed[arrow + 2..]
                .trim()
                .trim_end_matches('{')
                .trim()
                .to_owned();
            if !ret.is_empty() {
                return Some(ret);
            }
        }
    }
    None
}

pub fn run_suggest(decision: &str, sources: &[&str]) -> Option<String> {
    suggest_for_decision(decision, sources)
}
