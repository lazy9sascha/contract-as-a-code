use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::checker::valid_id;
use crate::codegen::{Case, Clause, ContractSchema, Operation};

/// Parse declared Markdown contract syntax, rejecting malformed structures.
/// This extracts obligations; it does not establish semantic completeness.
pub fn parse_markdown(content: &str) -> Result<ContractSchema, String> {
    if let Some(schema) = crate::codegen::embedded_schema(content, "<!-- caac-schema-v1:", " -->")?
    {
        if crate::codegen::generate_markdown(&schema)? != content {
            return Err("generated Markdown differs from its embedded schema; edit the schema and regenerate".to_owned());
        }
        return Ok(schema);
    }
    let mut schema = ContractSchema {
        name: "Contract".to_owned(),
        version: "0.1.0".to_owned(),
        description: String::new(),
        terms: BTreeMap::new(),
        operations: Vec::new(),
        clauses: Vec::new(),
        cases: Vec::new(),
        exclusions: Vec::new(),
        extends: Vec::new(),
    };
    let mut section = String::new();
    let mut current_op: Option<String> = None;
    let mut saw_title = false;
    let mut description_done = false;
    let projection = crate::markdown::project(content);
    if let Some(issue) = projection.issues.first() {
        return Err(format!("line {}: {}", issue.line, issue.message));
    }
    for record in projection.lines {
        let number = record.number;
        let line = record.text;
        let fail = |message: &str| format!("line {number}: {message}");
        if let crate::markdown::Kind::Heading { level, title } = record.kind {
            let title = title.as_str();
            if level >= 4 {
                return Err(fail(
                    "nested normative headings are unsupported; use explicit clauses or a non-normative section",
                ));
            }
            if level == 1 && !saw_title {
                schema.name = title.to_owned();
                saw_title = true;
            } else {
                description_done = true;
                if level <= 2 {
                    section = title.to_owned();
                    current_op = None;
                } else if level == 3 {
                    if section == "Clauses"
                        && title != "Invariants"
                        && !schema.operations.iter().any(|op| op.name == title)
                    {
                        return Err(fail("clause section must name a declared operation"));
                    }
                    if section == "Clauses"
                        && title == "Invariants"
                        && schema.operations.iter().any(|op| op.name == title)
                    {
                        return Err(fail("Invariants is reserved for standalone clauses"));
                    }
                    current_op =
                        (section == "Clauses" && title != "Invariants").then(|| title.to_owned());
                }
            }
            continue;
        }
        if let Some(revision) = line.strip_prefix("Revision:") {
            if let Some(value) = revision.split_whitespace().next() {
                schema.version = value.trim_end_matches('.').to_owned();
            }
            continue;
        }
        if let Some((left, _)) = line.split_once('|')
            && valid_id(left.trim())
        {
            let clause = parse_clause_line(&line)
                .ok_or_else(|| fail("invalid clause: expected actor and must/may obligation"))?;
            if let Some(op_name) = &current_op
                && let Some(op) = schema.operations.iter_mut().find(|op| &op.name == op_name)
                && !op.clauses.contains(&clause.id)
            {
                op.clauses.push(clause.id.clone());
            }
            if schema.clauses.iter().any(|c| c.id == clause.id) {
                return Err(fail("duplicate clause ID"));
            }
            schema.clauses.push(clause);
            continue;
        }
        if let Some(reference) = line.strip_prefix("Applies:") {
            let id = reference
                .trim()
                .strip_prefix('@')
                .filter(|id| valid_id(id))
                .ok_or_else(|| fail("invalid Applies clause reference"))?;
            let op = current_op
                .as_ref()
                .and_then(|name| schema.operations.iter_mut().find(|op| &op.name == name))
                .ok_or_else(|| fail("Applies requires a declared operation section"))?;
            if !op.clauses.iter().any(|existing| existing == id) {
                op.clauses.push(id.to_owned());
            }
            continue;
        }
        if crate::checker::looks_like_clause(&line) {
            return Err(fail("malformed clause definition: expected ID | sentence"));
        }
        if line.starts_with("OPEN:") {
            return Err(fail(
                "unresolved OPEN decision cannot be represented in the schema",
            ));
        }
        if [
            "except ",
            "exception:",
            "unless ",
            "provided ",
            "subject to ",
        ]
        .iter()
        .any(|prefix| line.to_ascii_lowercase().starts_with(prefix))
        {
            return Err(fail(
                "standalone conditions and exceptions are unsupported; keep the condition in an explicit clause",
            ));
        }
        if crate::checker::has_unnumbered_obligation(&record.marker_text) {
            return Err(fail(
                "unnumbered obligation cannot be converted to descriptive text",
            ));
        }
        // Resolved answers remain annotations, not table rows or signatures.
        if matches!(section.as_str(), "Operation signatures" | "Contract cases")
            && line.starts_with("DECISION:")
        {
            continue;
        }
        match section.as_str() {
            "Terms and inputs" => {
                if let Some(rest) = line
                    .strip_prefix("An `")
                    .or_else(|| line.strip_prefix("A `"))
                    && let Some((term, definition)) = rest.split_once('`')
                    && let Some(definition) = definition.trim().strip_prefix("is ")
                {
                    if schema.terms.contains_key(term) {
                        return Err(fail("duplicate term definition"));
                    }
                    schema
                        .terms
                        .insert(term.to_owned(), definition.trim().to_owned());
                } else {
                    return Err(fail("unsupported term definition syntax"));
                }
            }
            "Operation signatures" => {
                if line.starts_with("- ") {
                    let op =
                        parse_op_sig(&line).ok_or_else(|| fail("malformed operation signature"))?;
                    if schema
                        .operations
                        .iter()
                        .any(|existing| existing.name == op.name)
                    {
                        return Err(fail("duplicate operation name"));
                    }
                    schema.operations.push(op);
                } else {
                    return Err(fail("unsupported operation signature syntax"));
                }
            }
            "Contract cases" => {
                if line.starts_with('|') {
                    let cells =
                        table_cells(&line).ok_or_else(|| fail("malformed case table row"))?;
                    if cells.len() != 4 {
                        return Err(fail(
                            "case table requires exactly four cells; escape literal pipes with a backslash",
                        ));
                    }
                    if cells == ["Case", "Input", "Expected result", "Clauses"]
                        || cells == ["Case", "Input", "Expected", "Clauses"]
                        || cells.iter().all(|cell| {
                            !cell.is_empty() && cell.chars().all(|c| matches!(c, '-' | ':' | ' '))
                        })
                    {
                        continue;
                    }
                    schema.cases.push(Case {
                        name: cells[0].clone(),
                        input: cells[1].clone(),
                        expected: cells[2].clone(),
                        clauses: cells[3]
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_owned)
                            .collect(),
                    });
                } else {
                    return Err(fail("unsupported case table syntax"));
                }
            }
            "Extends" => {
                if let Some(value) = line.strip_prefix("- ") {
                    if value.is_empty() || schema.extends.iter().any(|base| base == value) {
                        return Err(fail("empty or duplicate base reference"));
                    }
                    schema.extends.push(value.to_owned());
                } else {
                    return Err(fail("unsupported base reference syntax"));
                }
            }
            "Exclusions" => {
                if let Some(value) = line.strip_prefix("- ") {
                    schema.exclusions.push(value.to_owned());
                } else {
                    return Err(fail("unsupported exclusion syntax"));
                }
            }
            _ => {}
        }
        if saw_title && !description_done && schema.description.is_empty() && !line.is_empty() {
            schema.description = line;
        }
    }
    if schema.clauses.is_empty() && schema.operations.is_empty() {
        return Err("no clauses or operation signatures found".to_owned());
    }
    for op in &schema.operations {
        for id in &op.clauses {
            if !schema.clauses.iter().any(|clause| &clause.id == id) {
                return Err(format!(
                    "operation {} references unknown clause {id}",
                    op.name
                ));
            }
        }
    }
    crate::codegen::validate_clause_references(&schema.clauses)?;
    for case in &schema.cases {
        for id in &case.clauses {
            if !schema.clauses.iter().any(|clause| &clause.id == id) {
                return Err(format!("case {} references unknown clause {id}", case.name));
            }
        }
    }
    Ok(schema)
}

fn parse_op_sig(line: &str) -> Option<Operation> {
    let line = line.strip_prefix("- ")?.trim().strip_prefix('`')?;
    let (sig, rest) = line.split_once('`')?;
    let open = sig.find('(')?;
    let close = sig.rfind(')')?;
    if open == 0 || close <= open || !sig[close + 1..].trim().is_empty() {
        return None;
    }
    let mut depth = 0usize;
    for (offset, ch) in sig[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 && open + offset != close {
                    return None;
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let rest = rest.trim();
    let returns = if rest.is_empty() {
        "()".to_owned()
    } else {
        let result = rest
            .strip_prefix("->")?
            .trim()
            .strip_prefix('`')?
            .strip_suffix('`')?;
        if result.is_empty() {
            return None;
        }
        result.to_owned()
    };
    Some(Operation {
        name: sig[..open].trim().to_owned(),
        params: sig[open + 1..close].trim().to_owned(),
        returns,
        clauses: Vec::new(),
    })
}

pub(crate) fn parse_clause_line(line: &str) -> Option<Clause> {
    let (id, sentence) = line.split_once('|')?;
    let sentence = sentence.trim();
    let marker = sentence.char_indices().find_map(|(start, _)| {
        if start > 0 && !sentence[..start].chars().next_back()?.is_whitespace() {
            return None;
        }
        for word in ["must", "may"] {
            let tail = &sentence[start..];
            if let Some(rest) = tail.strip_prefix(word)
                && rest.chars().next().is_some_and(char::is_whitespace)
            {
                return Some(start);
            }
        }
        None
    })?;
    let actor = sentence[..marker].trim();
    let obligation = sentence[marker..].trim();
    let (must, remainder) = if let Some(rest) = obligation.strip_prefix("must") {
        (true, rest.trim_start())
    } else {
        (false, obligation.strip_prefix("may")?.trim_start())
    };
    let remainder = if must {
        match remainder.strip_prefix("not") {
            Some(rest)
                if !rest
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_') =>
            {
                rest
            }
            _ => remainder,
        }
    } else {
        remainder
    };
    if actor.is_empty()
        || remainder
            .trim_matches(|c: char| c.is_whitespace() || c == '.')
            .is_empty()
    {
        return None;
    }
    Some(Clause {
        id: id.trim().to_owned(),
        actor: actor.to_owned(),
        obligation: obligation.to_owned(),
    })
}

fn table_cells(line: &str) -> Option<Vec<String>> {
    let body = line.strip_prefix('|')?.strip_suffix('|')?;
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if chars.peek().is_some_and(|next| matches!(next, '|' | '\\')) => {
                cell.push(chars.next()?)
            }
            '|' => {
                cells.push(cell.trim().to_owned());
                cell.clear();
            }
            _ => cell.push(ch),
        }
    }
    cells.push(cell.trim().to_owned());
    Some(cells)
}

pub fn run_parse(input: &Path, output: &Path) -> i32 {
    let content = match fs::read_to_string(input) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: cannot read {}: {}", input.display(), e);
            return 2;
        }
    };
    let schema = match parse_markdown(&content) {
        Ok(schema) => schema,
        Err(error) => {
            eprintln!("ERROR: {}: {}", input.display(), error);
            return 1;
        }
    };
    let json = match serde_json::to_string_pretty(&schema) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("ERROR: cannot serialize: {}", e);
            return 2;
        }
    };
    match crate::output::write_new(output, json.as_bytes()) {
        Ok(()) => {
            println!("Parsed: {}", output.display());
            println!(
                "Operations: {}, Clauses: {}, Cases: {}",
                schema.operations.len(),
                schema.clauses.len(),
                schema.cases.len()
            );
            0
        }
        Err(e) => {
            eprintln!("ERROR: cannot write {}: {}", output.display(), e);
            2
        }
    }
}
