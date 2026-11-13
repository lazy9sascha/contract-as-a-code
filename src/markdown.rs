//! Shared CommonMark structure with a bounded CAAC admission policy.
//! Source text, not decoded inline text, supplies normative domain fields.
use comrak::{
    Arena, Options,
    nodes::{NodeValue, Sourcepos},
    parse_document,
};

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Text,
    Heading { level: usize, title: String },
}

#[derive(Clone, Debug)]
pub(crate) struct Line {
    pub number: usize,
    pub text: String,
    pub marker_text: String,
    pub kind: Kind,
}

#[derive(Debug)]
pub(crate) struct Issue {
    pub line: usize,
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Default)]
pub(crate) struct Projection {
    pub lines: Vec<Line>,
    pub issues: Vec<Issue>,
}

pub(crate) fn project(content: &str) -> Projection {
    let normalized = content.replace("\r\n", "\n").replace(
        [
            '\r', '\u{000b}', '\u{000c}', '\u{001c}', '\u{001d}', '\u{001e}', '\u{0085}',
            '\u{2028}', '\u{2029}',
        ],
        "\n",
    );
    let source: Vec<&str> = normalized.split('\n').collect();
    let text: Vec<Vec<u8>> = source.iter().map(|line| line.as_bytes().to_vec()).collect();
    let mut hidden = vec![false; source.len()];
    let mut kinds = vec![Kind::Text; source.len()];
    let mut result = Projection::default();
    let mut local_issues: Vec<Vec<Issue>> = (0..source.len()).map(|_| Vec::new()).collect();
    let mut omissions: Vec<Vec<(usize, usize)>> = vec![Vec::new(); source.len()];
    let mut supported_heading = vec![false; source.len()];
    let mut code_spans: Vec<Vec<(usize, usize)>> = vec![Vec::new(); source.len()];
    let arena = Arena::new();
    let mut options = Options::default();
    // CAAC case rows retain their bounded domain decoder. GFM table parsing
    // rewrites escaped pipes before inline parsing, invalidating raw-source spans.
    // Do not enable it in a source-preserving projection.
    options.extension.front_matter_delimiter = Some("---".into());
    let offset = if source[0].starts_with("<!-- caac-schema-") {
        match crate::codegen::embedded_schema(source[0], "<!-- caac-schema-v1:", " -->") {
            Ok(Some(_)) => {
                hidden[0] = true;
                1
            }
            _ => {
                result.issues.push(Issue {
                    line: 1,
                    code: "invalid-metadata",
                    message: "Invalid CAAC metadata prefix",
                });
                0
            }
        }
    } else {
        0
    };
    let body = if offset == 1 {
        normalized.split_once('\n').map_or("", |(_, body)| body)
    } else {
        &normalized
    };
    let root = parse_document(&arena, body, &options);
    let mut frontmatter = false;
    for node in root.descendants() {
        let data = node.data();
        let pos = data.sourcepos;
        let Some(start) = pos
            .start
            .line
            .checked_sub(1)
            .and_then(|line| line.checked_add(offset))
        else {
            continue;
        };
        let Some(end) = pos.end.line.checked_add(offset) else {
            result.issues.push(span_issue(start.saturating_add(1)));
            continue;
        };
        if start >= source.len() || end <= start || end > source.len() {
            result.issues.push(span_issue(start.saturating_add(1)));
            continue;
        }
        match &data.value {
            NodeValue::FrontMatter(_) => {
                frontmatter = true;
                hidden[start..end].fill(true);
            }
            NodeValue::CodeBlock(code) => {
                if code.fenced && !code.closed {
                    result.issues.push(Issue {
                        line: start + 1,
                        code: "unclosed-fence",
                        message: "Code fence has no closing marker",
                    });
                }
                let top_level = node
                    .parent()
                    .is_some_and(|p| matches!(p.data().value, NodeValue::Document));
                if code.fenced && top_level {
                    hidden[start..end].fill(true);
                } else {
                    local_issues[start].push(Issue {
                        line: start + 1,
                        code: "unsupported-markdown",
                        message: "Indented or container code is unsupported in normative content",
                    });
                }
            }
            NodeValue::Heading(heading) => {
                let top_level = node
                    .parent()
                    .is_some_and(|p| matches!(p.data().value, NodeValue::Document));
                let plain = node
                    .children()
                    .all(|child| matches!(child.data().value, NodeValue::Text(_)));
                // Preserve source spelling: decoded entities and escapes are not canonical names.
                let title = match (node.first_child(), node.last_child()) {
                    (Some(first), Some(last)) if !heading.setext => {
                        let from = first.data().sourcepos.start.column.saturating_sub(1);
                        let to = last.data().sourcepos.end.column;
                        match source[start].get(from..to) {
                            Some(title) => title.to_owned(),
                            None => {
                                result.issues.push(span_issue(start + 1));
                                String::new()
                            }
                        }
                    }
                    _ => String::new(),
                };
                let nonempty = !title.trim().is_empty();
                if top_level {
                    kinds[start] = Kind::Heading {
                        level: usize::from(heading.level),
                        title,
                    };
                }
                supported_heading[start] =
                    plain && nonempty && !heading.setext && !heading.closed && top_level;
                if !supported_heading[start] {
                    local_issues[start].push(Issue { line: start + 1, code: "unsupported-markdown", message: "Use plain ATX headings without closing hashes in normative content" });
                }
            }
            NodeValue::HtmlBlock(html) => {
                let literal = html.literal.trim();
                if literal.starts_with("<!--") && !literal.contains("-->") {
                    result.issues.push(Issue {
                        line: start + 1,
                        code: "unclosed-comment",
                        message: "HTML comment has no closing marker",
                    });
                    hidden[start..end].fill(true);
                } else if literal
                    .strip_prefix("<!--")
                    .and_then(|s| s.strip_suffix("-->"))
                    .is_some_and(|s| !s.contains("-->"))
                {
                    hidden[start..end].fill(true);
                } else {
                    local_issues[start].push(Issue { line: start + 1, code: "unsupported-markdown", message: "Raw HTML or mixed comment blocks are unsupported in normative content" });
                }
            }
            NodeValue::Code(code) => {
                if pos.start.line != pos.end.line {
                    local_issues[start].push(Issue {
                        line: start + 1,
                        code: "unsupported-markdown",
                        message: "Multiline inline code is unsupported in normative content",
                    });
                }
                match source_spans(&source, pos, offset) {
                    Some(ranges) => {
                        let raw = span_text(&source, &ranges);
                        let delimiters = "`".repeat(code.num_backticks);
                        if code.num_backticks == 0
                            || raw.len() < 2 * code.num_backticks
                            || !raw.starts_with(&delimiters)
                            || !raw.ends_with(&delimiters)
                        {
                            result.issues.push(span_issue(start + 1));
                        } else {
                            for (line, from, to) in ranges {
                                code_spans[line].push((from, to));
                            }
                        }
                    }
                    None => result.issues.push(span_issue(start + 1)),
                }
            }
            NodeValue::HtmlInline(html) => {
                if html.starts_with("<!--") && html.ends_with("-->") {
                    match source_spans(&source, pos, offset) {
                        Some(ranges) if span_text(&source, &ranges) == *html => {
                            for (line, from, to) in ranges {
                                omissions[line].push((from, to));
                            }
                        }
                        _ => result.issues.push(span_issue(start + 1)),
                    }
                } else {
                    local_issues[start].push(Issue {
                        line: start + 1,
                        code: "unsupported-markdown",
                        message: "Raw inline HTML is unsupported in normative content",
                    });
                }
            }
            NodeValue::BlockQuote => local_issues[start].push(Issue {
                line: start + 1,
                code: "unsupported-markdown",
                message: "Block quotes are unsupported in normative content",
            }),
            _ => {}
        }
    }
    if source.get(offset).is_some_and(|line| line.trim() == "---") && !frontmatter {
        result.issues.push(Issue {
            line: offset + 1,
            code: "unclosed-frontmatter",
            message: "Frontmatter has no closing marker",
        });
        hidden.fill(true);
    }
    let mut excluded: Option<usize> = None;
    for (index, bytes) in text.into_iter().enumerate() {
        if hidden[index] {
            continue;
        }
        if let Kind::Heading { level, title } = &kinds[index] {
            if excluded.is_some_and(|current| *level <= current) {
                excluded = None;
            }
            if supported_heading[index]
                && excluded.is_none()
                && title.to_lowercase().contains("(non-normative)")
            {
                excluded = Some(*level);
            }
        }
        let mut keep = vec![true; bytes.len()];
        for (from, to) in &omissions[index] {
            keep[*from..*to].fill(false);
        }
        let mut marker_bytes = bytes.clone();
        for (from, to) in &code_spans[index] {
            marker_bytes[*from..*to].fill(b' ');
        }
        let marker_text = String::from_utf8(
            marker_bytes
                .into_iter()
                .zip(&keep)
                .filter_map(|(byte, keep)| keep.then_some(byte))
                .collect(),
        );
        let Ok(marker_text) = marker_text else {
            result.issues.push(span_issue(index + 1));
            continue;
        };
        let visible = String::from_utf8(
            bytes
                .into_iter()
                .zip(keep)
                .filter_map(|(byte, keep)| keep.then_some(byte))
                .collect(),
        );
        let Ok(visible) = visible else {
            result.issues.push(span_issue(index + 1));
            continue;
        };
        let visible = visible.trim();
        if marker_text.contains("<!--") {
            result.issues.push(Issue {
                line: index + 1,
                code: "unclosed-comment",
                message: "HTML comment is incomplete or unsupported",
            });
        }
        if excluded.is_some() {
            continue;
        }
        result.issues.append(&mut local_issues[index]);
        if !visible.is_empty() {
            result.lines.push(Line {
                number: index + 1,
                text: visible.to_owned(),
                marker_text: marker_text.trim().to_owned(),
                kind: kinds[index].clone(),
            });
        }
    }
    result
}

// Validate provenance before any byte is omitted or masked. AST coordinates
// must describe real UTF-8 source boundaries, not a transformed parser buffer.
fn source_spans(
    source: &[&str],
    pos: Sourcepos,
    offset: usize,
) -> Option<Vec<(usize, usize, usize)>> {
    let first = pos.start.line.checked_sub(1)?.checked_add(offset)?;
    let last = pos.end.line.checked_sub(1)?.checked_add(offset)?;
    if first > last || last >= source.len() {
        return None;
    }
    let mut ranges = Vec::new();
    for (line, text) in source.iter().enumerate().take(last + 1).skip(first) {
        let from = if line == first {
            pos.start.column.checked_sub(1)?
        } else {
            0
        };
        let to = if line == last {
            pos.end.column
        } else {
            text.len()
        };
        text.get(from..to)?;
        ranges.push((line, from, to));
    }
    Some(ranges)
}

fn span_text(source: &[&str], ranges: &[(usize, usize, usize)]) -> String {
    ranges
        .iter()
        .map(|(line, from, to)| &source[*line][*from..*to])
        .collect::<Vec<_>>()
        .join("\n")
}

fn span_issue(line: usize) -> Issue {
    Issue {
        line,
        code: "unsupported-markdown",
        message: "Markdown source spans cannot preserve this input",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::nodes::LineColumn;

    #[test]
    fn source_spans_reject_invalid_boundaries_without_panicking() {
        let source = ["aé𐀀z"];
        for (start, end) in [(0, 1), (3, 7), (2, 4), (1, 99), (6, 2)] {
            let pos = Sourcepos {
                start: LineColumn {
                    line: 1,
                    column: start,
                },
                end: LineColumn {
                    line: 1,
                    column: end,
                },
            };
            assert!(source_spans(&source, pos, 0).is_none(), "{start}..{end}");
        }
        let pos = Sourcepos {
            start: LineColumn { line: 1, column: 2 },
            end: LineColumn { line: 1, column: 7 },
        };
        let ranges = source_spans(&source, pos, 0).unwrap();
        assert_eq!(span_text(&source, &ranges), "é𐀀");
        assert!(source_spans(&source, pos, 1).is_none());
    }

    #[test]
    fn multiline_comment_omission_preserves_physical_source_lines() {
        let projected = project("prefix <!-- é\n𐀀 --> suffix\n");
        assert!(projected.issues.is_empty(), "{:?}", projected.issues);
        assert_eq!(projected.lines.len(), 2);
        assert_eq!(projected.lines[0].number, 1);
        assert_eq!(projected.lines[0].text, "prefix");
        assert_eq!(projected.lines[1].number, 2);
        assert_eq!(projected.lines[1].text, "suffix");
    }
}
