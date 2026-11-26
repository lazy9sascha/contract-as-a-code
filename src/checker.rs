use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::markdown::{Kind, Line, table_cells};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct Diagnostic {
    pub path: String,
    pub line: usize,
    pub code: String,
    pub message: String,
}

pub fn check(paths: &[PathBuf]) -> Result<Vec<Diagnostic>, String> {
    analyze(paths, false)
}

pub fn lint(paths: &[PathBuf]) -> Result<Vec<Diagnostic>, String> {
    analyze(paths, true)
}

pub fn lint_documents<'a>(
    documents: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<Diagnostic> {
    let mut analysis = Analysis::default();
    for (name, contents) in documents {
        analysis.inspect(name, contents, true);
    }
    analysis.finish()
}

fn analyze(paths: &[PathBuf], lint_cases: bool) -> Result<Vec<Diagnostic>, String> {
    let mut analysis = Analysis::default();
    for path in paths {
        let path_text = path.display().to_string();
        let contents = fs::read_to_string(path).map_err(|error| format!("{path_text}: {error}"))?;
        analysis.inspect(&path_text, &contents, lint_cases);
    }
    Ok(analysis.finish())
}

#[derive(Default)]
struct Analysis {
    diagnostics: Vec<Diagnostic>,
    definitions: HashMap<String, String>,
    references: Vec<(String, String, usize)>,
}

impl Analysis {
    fn inspect(&mut self, path_text: &str, contents: &str, lint_cases: bool) {
        let Self {
            diagnostics,
            definitions,
            references,
        } = self;
        let mut clause_count = 0;
        let projection = crate::markdown::project(contents);
        for issue in projection.issues {
            report(
                diagnostics,
                path_text,
                issue.line,
                issue.code,
                issue.message,
            );
        }
        if lint_cases {
            check_cases(&projection.lines, path_text, diagnostics, references);
        }
        for record in projection.lines {
            let line_number = record.number;
            let line = record.text.as_str();

            if let Some(decision) = line.strip_prefix("OPEN:") {
                let message = decision.trim();
                report(
                    diagnostics,
                    path_text,
                    line_number,
                    "open-decision",
                    if message.is_empty() {
                        "Unspecified decision"
                    } else {
                        message
                    },
                );
            }

            let mut body = line;
            if let Some((identifier, definition_body)) = parse_definition(line) {
                clause_count += 1;
                if let Some(first_location) = definitions.get(identifier) {
                    report(
                        diagnostics,
                        path_text,
                        line_number,
                        "duplicate-id",
                        &format!("{identifier} already defined at {first_location}"),
                    );
                } else {
                    definitions.insert(identifier.to_owned(), format!("{path_text}:{line_number}"));
                }
                body = definition_body;

                let marker = find_marker(body);
                let has_actor = marker.is_some_and(|(start, _)| !body[..start].trim().is_empty());
                let has_obligation = marker.is_some_and(|(_, end)| {
                    !body[end..]
                        .trim_matches(|character: char| {
                            character.is_whitespace() || character == '.'
                        })
                        .is_empty()
                });
                if !has_actor || !has_obligation {
                    report(
                        diagnostics,
                        path_text,
                        line_number,
                        "clause-shape",
                        "Expected actor, normative marker, and obligation",
                    );
                }
                if body.split_whitespace().count() > 20 {
                    report(
                        diagnostics,
                        path_text,
                        line_number,
                        "clause-length",
                        "Clause exceeds 20 whitespace-separated words",
                    );
                }
                if body.contains(';') {
                    report(
                        diagnostics,
                        path_text,
                        line_number,
                        "semicolon",
                        "Split the clause without a semicolon",
                    );
                }
            } else if looks_like_clause(line) {
                report(
                    diagnostics,
                    path_text,
                    line_number,
                    "clause-shape",
                    "Expected ID | sentence",
                );
            } else if has_unnumbered_obligation(&record.marker_text) {
                report(
                    diagnostics,
                    path_text,
                    line_number,
                    "unnumbered-obligation",
                    "Normative line needs a clause ID",
                );
            }

            references.extend(
                find_references(body)
                    .into_iter()
                    .map(|identifier| (identifier, path_text.to_owned(), line_number)),
            );
        }

        if clause_count == 0 {
            report(
                diagnostics,
                path_text,
                1,
                "no-clauses",
                "File contains no clause definitions",
            );
        }
    }

    fn finish(mut self) -> Vec<Diagnostic> {
        for (identifier, path, line_number) in self.references {
            if !self.definitions.contains_key(&identifier) {
                report(
                    &mut self.diagnostics,
                    &path,
                    line_number,
                    "unknown-id",
                    &format!("Undefined clause {identifier}"),
                );
            }
        }
        self.diagnostics
    }
}

fn check_cases(
    lines: &[Line],
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    references: &mut Vec<(String, String, usize)>,
) {
    let mut sections = 0;
    for (index, heading) in lines.iter().enumerate() {
        let Kind::Heading { level, title } = &heading.kind else {
            continue;
        };
        if !matches!(
            title.as_str(),
            "Contract cases" | "Acceptance cases" | "Cases"
        ) {
            continue;
        }
        sections += 1;
        let mut table = lines[index + 1..]
            .iter()
            .take_while(
                |line| !matches!(&line.kind, Kind::Heading { level: next, .. } if next <= level),
            )
            .filter(|line| line.text.starts_with('|'));
        let Some(header) = table.next() else {
            report(
                diagnostics,
                path,
                heading.number,
                "missing-case-table",
                "Add a case table with at least one case.",
            );
            continue;
        };
        let header_matches = table_cells(&header.text)
            .is_some_and(|cells| cells == ["Case", "Input", "Expected result", "Clauses"]);
        if !header_matches {
            report(
                diagnostics,
                path,
                header.number,
                "case-table-header",
                "Use | Case | Input | Expected result | Clauses |. Put initial conditions and inputs in Input.",
            );
            continue;
        }
        let divider = table.next();
        let divider_matches = divider
            .filter(|row| row.number == header.number + 1)
            .and_then(|row| table_cells(&row.text))
            .is_some_and(|cells| {
                cells.len() == 4
                    && cells.iter().all(|cell| {
                        let dashes = cell.strip_prefix(':').unwrap_or(cell);
                        let dashes = dashes.strip_suffix(':').unwrap_or(dashes);
                        dashes.len() >= 3 && dashes.bytes().all(|byte| byte == b'-')
                    })
            });
        if !divider_matches {
            report(
                diagnostics,
                path,
                divider.unwrap_or(header).number,
                "case-table-divider",
                "Add | --- | --- | --- | --- | after the case header.",
            );
            continue;
        }
        let mut previous = header.number + 1;
        for row in table {
            if row.number != previous + 1 {
                report(
                    diagnostics,
                    path,
                    row.number,
                    "case-table-layout",
                    "Keep the header, divider, and case rows on consecutive lines.",
                );
                break;
            }
            previous = row.number;
            let Some(cells) = table_cells(&row.text) else {
                report(
                    diagnostics,
                    path,
                    row.number,
                    "case-row",
                    "Give the case four pipe-delimited cells.",
                );
                continue;
            };
            let [case, input, expected, binding] = cells.as_slice() else {
                report(
                    diagnostics,
                    path,
                    row.number,
                    "case-row",
                    "Give the case four cells. Escape pipes inside a value.",
                );
                continue;
            };
            if [case, input, expected, binding]
                .into_iter()
                .any(|cell| cell.is_empty())
            {
                report(
                    diagnostics,
                    path,
                    row.number,
                    "case-row",
                    "Fill in the case name, input, expected result, and clauses.",
                );
                continue;
            }
            for entry in binding.split(',').map(str::trim) {
                let id = entry.strip_prefix('@').unwrap_or(entry);
                if !valid_id(id) {
                    report(
                        diagnostics,
                        path,
                        row.number,
                        "case-clause-id",
                        "List existing clause IDs separated by commas in the Clauses column.",
                    );
                    break;
                }
                if entry == id {
                    references.push((id.to_owned(), path.to_owned(), row.number));
                }
            }
        }
        if previous == header.number + 1 {
            report(
                diagnostics,
                path,
                header.number,
                "missing-case-table",
                "Add at least one case with a clause reference.",
            );
        }
    }
    if sections == 0 {
        report(
            diagnostics,
            path,
            1,
            "missing-case-table",
            "Add a Contract cases section with a case table.",
        );
    }
}

fn report(diagnostics: &mut Vec<Diagnostic>, path: &str, line: usize, code: &str, message: &str) {
    diagnostics.push(Diagnostic {
        path: path.to_owned(),
        line,
        code: code.to_owned(),
        message: message.to_owned(),
    });
}

fn parse_definition(line: &str) -> Option<(&str, &str)> {
    let (left, right) = line.split_once('|')?;
    let identifier = left.trim_end();
    if !valid_id(identifier) {
        return None;
    }
    Some((identifier, right.trim_start()))
}

pub(crate) fn valid_id(identifier: &str) -> bool {
    let mut parts = identifier.split('-').collect::<Vec<_>>();
    if parts.len() < 2 {
        return false;
    }
    let sequence = parts.pop().expect("at least two segments");
    if sequence.len() != 3 || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    parts.iter().all(|part| {
        let mut bytes = part.bytes();
        matches!(bytes.next(), Some(b'A'..=b'Z'))
            && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    })
}

pub(crate) fn looks_like_clause(line: &str) -> bool {
    if let Some((left, _)) = line.split_once('|') {
        let identifier = left.trim_end();
        if !identifier.is_empty()
            && matches!(identifier.as_bytes().first(), Some(b'A'..=b'Z'))
            && identifier
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return true;
        }
    }
    let Some(identifier) = line.split_whitespace().next() else {
        return false;
    };
    valid_id(identifier)
        && line[identifier.len()..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
}

pub(crate) fn has_unnumbered_obligation(line: &str) -> bool {
    !line.starts_with('|') && find_marker(line).is_some()
}

fn find_marker(text: &str) -> Option<(usize, usize)> {
    let mut earliest = None;
    for (start, _) in text.char_indices() {
        for marker in ["must", "may"] {
            if !text[start..].starts_with(marker) {
                continue;
            }
            let end_word = start + marker.len();
            if !is_word_boundary(text, start) || !is_word_boundary(text, end_word) {
                continue;
            }
            let mut end = end_word;
            if marker == "must" {
                let whitespace_length = text[end..]
                    .chars()
                    .take_while(|character| character.is_whitespace())
                    .map(char::len_utf8)
                    .sum::<usize>();
                let not_start = end + whitespace_length;
                if whitespace_length > 0 && text[not_start..].starts_with("not") {
                    let not_end = not_start + 3;
                    if is_word_boundary(text, not_end) {
                        end = not_end;
                    }
                }
            }
            let found = (start, end);
            if earliest.is_none_or(|current: (usize, usize)| start < current.0) {
                earliest = Some(found);
            }
        }
    }
    earliest
}

fn is_word_boundary(text: &str, byte_index: usize) -> bool {
    let before = text[..byte_index]
        .chars()
        .next_back()
        .is_some_and(is_word_character);
    let after = text[byte_index..]
        .chars()
        .next()
        .is_some_and(is_word_character);
    before != after
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

pub(crate) fn find_references(text: &str) -> Vec<String> {
    let mut references = Vec::new();
    for (start, character) in text.char_indices() {
        if character != '@' {
            continue;
        }
        if text[..start]
            .chars()
            .next_back()
            .is_some_and(is_word_character)
        {
            continue;
        }
        let tail = &text[start + 1..];
        let mut candidate_end = 0;
        for (offset, character) in tail.char_indices() {
            if !(character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-') {
                break;
            }
            candidate_end = offset + character.len_utf8();
        }
        let candidate = &tail[..candidate_end];
        let mut best = None;
        for (end, _) in candidate
            .char_indices()
            .chain(std::iter::once((candidate.len(), '\0')))
        {
            if end == 0 || !valid_id(&candidate[..end]) {
                continue;
            }
            let following = &tail[end..];
            if !following.chars().next().is_some_and(is_word_character) {
                best = Some(candidate[..end].to_owned());
            }
        }
        if let Some(identifier) = best {
            references.push(identifier);
        }
    }
    references
}
