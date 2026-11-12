use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub line: usize,
    pub code: String,
    pub message: String,
}

pub fn check(paths: &[PathBuf]) -> Result<Vec<Diagnostic>, String> {
    let mut diagnostics = Vec::new();
    let mut definitions = HashMap::new();
    let mut references = Vec::new();

    for path in paths {
        let path_text = path.display().to_string();
        let contents = fs::read_to_string(path).map_err(|error| format!("{path_text}: {error}"))?;
        let mut clause_count = 0;
        let mut fence: Option<(char, usize, usize)> = None;
        let mut comment_start = None;
        let mut excluded = None;

        for (line_index, raw) in split_lines(&contents).into_iter().enumerate() {
            let line_number = line_index + 1;
            if let Some((marker, length, _)) = fence {
                if closes_fence(raw.trim(), marker, length) {
                    fence = None;
                }
                continue;
            }

            let visible = remove_comments(raw, line_number, &mut comment_start);
            let line = visible.trim();

            if let Some((marker, length)) = opening_fence(line) {
                fence = Some((marker, length, line_number));
                continue;
            }

            if let Some((level, title)) = heading(line) {
                if excluded.is_some_and(|current| level <= current) {
                    excluded = None;
                }
                if excluded.is_none() && title.to_lowercase().contains("(non-normative)") {
                    excluded = Some(level);
                }
            }

            if excluded.is_some() || line.is_empty() {
                continue;
            }

            if let Some(decision) = line.strip_prefix("OPEN:") {
                let message = decision.trim();
                report(
                    &mut diagnostics,
                    &path_text,
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
                        &mut diagnostics,
                        &path_text,
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
                        &mut diagnostics,
                        &path_text,
                        line_number,
                        "clause-shape",
                        "Expected actor, normative marker, and obligation",
                    );
                }
                if body.split_whitespace().count() > 20 {
                    report(
                        &mut diagnostics,
                        &path_text,
                        line_number,
                        "clause-length",
                        "Clause exceeds 20 whitespace-separated words",
                    );
                }
                if body.contains(';') {
                    report(
                        &mut diagnostics,
                        &path_text,
                        line_number,
                        "semicolon",
                        "Split the clause without a semicolon",
                    );
                }
            } else if looks_like_clause(line) {
                report(
                    &mut diagnostics,
                    &path_text,
                    line_number,
                    "clause-shape",
                    "Expected ID | sentence",
                );
            } else if !line.starts_with('|') && find_marker(&without_quoted_text(line)).is_some() {
                report(
                    &mut diagnostics,
                    &path_text,
                    line_number,
                    "unnumbered-obligation",
                    "Normative line needs a clause ID",
                );
            }

            references.extend(
                find_references(body)
                    .into_iter()
                    .map(|identifier| (identifier, path_text.clone(), line_number)),
            );
        }

        if let Some((_, _, opening_line)) = fence {
            report(
                &mut diagnostics,
                &path_text,
                opening_line,
                "unclosed-fence",
                "Code fence has no closing marker",
            );
        }
        if let Some(opening_line) = comment_start {
            report(
                &mut diagnostics,
                &path_text,
                opening_line,
                "unclosed-comment",
                "HTML comment has no closing marker",
            );
        }
        if clause_count == 0 {
            report(
                &mut diagnostics,
                &path_text,
                1,
                "no-clauses",
                "File contains no clause definitions",
            );
        }
    }

    for (identifier, path, line_number) in references {
        if !definitions.contains_key(&identifier) {
            report(
                &mut diagnostics,
                &path,
                line_number,
                "unknown-id",
                &format!("Undefined clause {identifier}"),
            );
        }
    }
    Ok(diagnostics)
}

fn report(diagnostics: &mut Vec<Diagnostic>, path: &str, line: usize, code: &str, message: &str) {
    diagnostics.push(Diagnostic {
        path: path.to_owned(),
        line,
        code: code.to_owned(),
        message: message.to_owned(),
    });
}

fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < text.len() {
        let character = text[index..]
            .chars()
            .next()
            .expect("valid character boundary");
        let length = character.len_utf8();
        if is_line_break(character) {
            lines.push(&text[start..index]);
            index += length;
            if character == '\r' && text[index..].starts_with('\n') {
                index += 1;
            }
            start = index;
        } else {
            index += length;
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

fn is_line_break(character: char) -> bool {
    matches!(
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
    )
}

fn remove_comments(raw: &str, line_number: usize, comment_start: &mut Option<usize>) -> String {
    let mut visible = String::new();
    let mut remaining = raw;
    loop {
        if comment_start.is_some() {
            if let Some(end) = remaining.find("-->") {
                remaining = &remaining[end + 3..];
                *comment_start = None;
            } else {
                break;
            }
        } else if let Some(start) = remaining.find("<!--") {
            visible.push_str(&remaining[..start]);
            remaining = &remaining[start + 4..];
            *comment_start = Some(line_number);
        } else {
            visible.push_str(remaining);
            break;
        }
    }
    visible
}

fn opening_fence(line: &str) -> Option<(char, usize)> {
    let marker = line.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let length = line
        .chars()
        .take_while(|character| *character == marker)
        .count();
    (length >= 3).then_some((marker, length))
}

fn closes_fence(line: &str, marker: char, minimum: usize) -> bool {
    let length = line
        .chars()
        .take_while(|character| *character == marker)
        .count();
    length >= minimum && line[length..].trim().is_empty()
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let remainder = &line[level..];
    let first = remainder.chars().next()?;
    if !first.is_whitespace() {
        return None;
    }
    Some((level, remainder.trim_start()))
}

fn parse_definition(line: &str) -> Option<(&str, &str)> {
    let (left, right) = line.split_once('|')?;
    let identifier = left.trim_end();
    if !valid_id(identifier) {
        return None;
    }
    Some((identifier, right.trim_start()))
}

fn valid_id(identifier: &str) -> bool {
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

fn looks_like_clause(line: &str) -> bool {
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

fn without_quoted_text(line: &str) -> String {
    let mut visible = String::new();
    let mut inside = false;
    let mut cursor = 0;
    while cursor < line.len() {
        let character = line[cursor..]
            .chars()
            .next()
            .expect("valid character boundary");
        if character == '`' {
            inside = !inside;
        } else if !inside {
            visible.push(character);
        }
        cursor += character.len_utf8();
    }
    visible
}

fn find_references(text: &str) -> Vec<String> {
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
