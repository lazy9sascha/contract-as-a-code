//! Bounded labeled legal-style view. No inference or document-to-schema parser.
use super::*;

pub const PROFILE: &str = "caac-candidate-legal-review-v1";
const SUBITEM_DESCRIPTION: &str = "Subitem profile: `caac-subitem-presentation-v1`. Paragraph numbers restart in each article. Item numbers restart in each paragraph. Subitem numbers restart in each item. All three levels start at 1. Grouping records presentation only.\n\n";
pub(super) fn profile(candidate: &Candidate) -> &'static str {
    if candidate.has_expressions() {
        "caac-candidate-legal-review-v9"
    } else if candidate.uses_v8() {
        "caac-candidate-legal-review-v8"
    } else if candidate.presentation.uses_v7() {
        "caac-candidate-legal-review-v7"
    } else if candidate.presentation.has_extended_hierarchy() {
        "caac-candidate-legal-review-v6"
    } else if candidate.presentation.size == "XL" {
        "caac-candidate-legal-review-v5"
    } else if candidate.presentation.size == "XS" {
        "caac-candidate-legal-review-v4"
    } else if candidate.presentation.has_items() {
        "caac-candidate-legal-review-v3"
    } else if candidate.presentation.has_paragraphs() {
        "caac-candidate-legal-review-v2"
    } else {
        PROFILE
    }
}

// JSON quoting retains literal types and line boundaries. Escape invisible directional
// controls as well; dynamic data is never interpreted as Markdown or HTML.
fn display_json(value: &impl Serialize, pretty: bool) -> Result<String, String> {
    let text = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .map_err(|e| e.to_string())?;
    Ok(text.chars().fold(String::new(), |mut out, ch| {
        if (ch.is_control() && ch != '\n') || matches!(ch, '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}') {
            out.push_str(&format!("\\u{:04x}", ch as u32));
        } else {
            out.push(ch);
        }
        out
    }))
}
fn fence(text: &str, minimum: usize) -> String {
    "`".repeat(
        text.split(|c| c != '`')
            .map(str::len)
            .max()
            .unwrap_or(0)
            .max(minimum - 1)
            + 1,
    )
}
fn literal(value: &impl Serialize) -> Result<String, String> {
    let text = display_json(value, false)?;
    let mark = fence(&text, 1);
    Ok(format!("{mark} {text} {mark}"))
}
fn exact(out: &mut String, value: &impl Serialize) -> Result<(), String> {
    let text = display_json(value, true)?;
    let mark = fence(&text, 3);
    out.push_str(&format!("{mark}json\n{text}\n{mark}\n\n"));
    Ok(())
}
pub(super) fn label(key: &str) -> &str {
    match key {
        "id" => "Stable ID",
        "kind" => "Kind",
        "text" => "Authored text",
        "actor" => "Actor",
        "modality" => "Modality",
        "definition_refs" => "Definition references",
        "name" => "Operation name",
        "inputs" => "Declared input types",
        "output" => "Declared result type",
        "clause_refs" => "Clause references",
        "exclusions" => "Exclusions",
        "target" => "Target clause",
        "applicability" => "Applicability (explicit condition)",
        "effect" => "Effect (explicit replacement)",
        "operation" => "Operation reference",
        "expression" => "Expression reference",
        "input" => "Given input values",
        "expected" => "Expected result (not executed)",
        "exception_refs" => "Exception references",
        "annex_refs" => "Annex references",
        "form_refs" => "Form references",
        "sentence" => "Sentence reference",
        "clause" => "Parent clause",
        "portion" => "Sentence portion",
        "title" => "Title",
        "status" => "Attachment status",
        "columns" => "Table columns",
        "rows" => "Table rows",
        "fields" => "Form fields",
        "question" => "Open question",
        "required" => "Required decision",
        "proposal" => "Pending proposal (not resolved)",
        _ => key,
    }
}
fn fields(out: &mut String, value: &impl Serialize) -> Result<(), String> {
    let object = serde_json::to_value(value).map_err(|e| e.to_string())?;
    for (key, value) in object.as_object().ok_or("Expected typed object")? {
        out.push_str(&format!("- **{}:** {}\n", label(key), literal(value)?));
    }
    out.push('\n');
    Ok(())
}
fn heading_level(depth: usize) -> String {
    "#".repeat(depth.min(6))
}
fn entity(out: &mut String, value: &Entity, depth: usize) -> Result<(), String> {
    let kind = match value {
        Entity::Definition { .. } => "Definition",
        Entity::Clause { .. } => "Clause",
        Entity::Operation { .. } => "Operation",
        Entity::Exception { .. } => "Exception",
        Entity::Case(_) => "Case",
        Entity::Annex { .. } => "Annex",
        Entity::Form { .. } => "Form",
        Entity::SentencePart { .. } => "Sentence-Part",
        Entity::Expression { .. } => "Expression",
    };
    out.push_str(&format!(
        "{} {kind} · {}\n\n",
        heading_level(depth),
        literal(&value.id())?
    ));
    fields(out, value)?;
    out.push_str("Exact typed entity (same source, not additional obligations):\n\n");
    exact(out, value)?;
    match value {
        Entity::Annex { columns, rows, .. } => {
            out.push_str("**Table** (same source as the exact entity):\n\n");
            out.push('|');
            for column in columns {
                out.push_str(&format!(" {} |", literal(column)?.replace('|', "\\|")));
            }
            out.push_str("\n|");
            for _ in columns {
                out.push_str(" --- |");
            }
            out.push('\n');
            for row in rows {
                out.push('|');
                for cell in row {
                    out.push_str(&format!(" {} |", literal(cell)?.replace('|', "\\|")));
                }
                out.push('\n');
            }
            out.push('\n');
        }
        Entity::Form { fields, .. } => {
            out.push_str("**Form fields** (same source as the exact entity):\n\n");
            for field in fields {
                out.push_str(&format!(
                    "- Field {}: {} (type: {}, required: {})\n",
                    literal(&field.id)?,
                    literal(&field.label)?,
                    literal(&field.value_type)?,
                    literal(&field.required)?
                ));
            }
            out.push('\n');
        }
        _ => {}
    }
    Ok(())
}

fn attachments(
    out: &mut String,
    ids: &[String],
    entities: &BTreeMap<&str, &Entity>,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    out.push_str("## Attachments\n\n");
    for (index, id) in ids.iter().enumerate() {
        out.push_str(&format!("### Attachment {}\n\n", index + 1));
        entity(out, entities[id.as_str()], 4)?;
    }
    Ok(())
}
fn article(
    out: &mut String,
    node: &Node,
    entities: &BTreeMap<&str, &Entity>,
    number: Option<usize>,
    depth: usize,
) -> Result<(), String> {
    let Node::Article {
        id,
        title,
        entities: refs,
        paragraphs,
    } = node
    else {
        return Err("Expected article".into());
    };
    let level = heading_level(depth);
    let prefix = number
        .map(|n| format!("Article {n} — "))
        .unwrap_or_default();
    out.push_str(&format!(
        "{level} {prefix}{}\n\nPresentation ID: {}\n\n",
        literal(title)?,
        literal(id)?
    ));
    for id in refs.iter().flatten() {
        entity(out, entities[id.as_str()], depth + 1)?;
    }
    if let Some(paragraphs) = paragraphs {
        for (index, paragraph) in paragraphs.iter().enumerate() {
            let level = heading_level(depth + 1);
            out.push_str(&format!(
                "{level} Paragraph {}\n\nPresentation ID: {}\n\n",
                index + 1,
                literal(&paragraph.id)?
            ));
            for id in paragraph.entities.iter().flatten() {
                entity(out, entities[id.as_str()], depth + 2)?;
            }
            if let Some(items) = &paragraph.items {
                for (item_index, item) in items.iter().enumerate() {
                    let level = heading_level(depth + 2);
                    out.push_str(&format!(
                        "{level} Item {}\n\nPresentation ID: {}\n\n",
                        item_index + 1,
                        literal(&item.id)?
                    ));
                    for id in item.entities.iter().flatten() {
                        entity(out, entities[id.as_str()], depth + 3)?;
                    }
                    if let Some(subitems) = &item.subitems {
                        for (subitem_index, subitem) in subitems.iter().enumerate() {
                            out.push_str(&format!(
                                "{} Subitem {}\n\nPresentation ID: {}\n\n",
                                heading_level(depth + 3),
                                subitem_index + 1,
                                literal(&subitem.id)?
                            ));
                            for id in &subitem.entities {
                                entity(out, entities[id.as_str()], depth + 4)?;
                            }
                        }
                    }
                }
            }
        }
    }
    out.push_str("Exact article presentation:\n\n");
    exact(out, node)
}

fn exact_group(out: &mut String, node: &Node) -> Result<(), String> {
    let (kind, _, _) = node.upper().ok_or("Expected upper-level node")?;
    out.push_str(&format!("Exact {kind} presentation:\n\n"));
    exact(out, node)
}

fn nodes(
    out: &mut String,
    nodes: &[Node],
    entities: &BTreeMap<&str, &Entity>,
    number: &mut usize,
    unnumbered: bool,
) -> Result<(), String> {
    // Emit a group's exact projection after its children, as in earlier profiles.
    let mut pending: Vec<&Node> = Vec::new();
    super::hierarchy::walk(nodes, number, &mut |node, ordinal, path| {
        while pending.len() > path.len() {
            exact_group(out, pending.pop().unwrap())?;
        }
        if let Some((kind, id, title)) = node.upper() {
            let label = match kind {
                "part" => "Part",
                "chapter" => "Chapter",
                "section" => "Section",
                "subsection" => "Subsection",
                _ => return Err("Expected validated upper-level node".into()),
            };
            out.push_str(&format!(
                "{} {label} {ordinal} — {}\n\nPresentation ID: {}\n\n",
                heading_level(2 + path.len()),
                literal(&title)?,
                literal(&id)?
            ));
            pending.push(node);
        } else {
            article(
                out,
                node,
                entities,
                (!unnumbered).then_some(ordinal),
                2 + path.len(),
            )?;
        }
        Ok(())
    })?;
    while let Some(node) = pending.pop() {
        exact_group(out, node)?;
    }
    Ok(())
}

pub(super) fn render(candidate: &Candidate) -> Result<String, String> {
    let manifest = revisions(candidate)?;
    let profile = profile(candidate);
    if candidate.presentation.size == "XL" {
        let mut out = format!(
            "# Candidate legal-style review index\n\nRenderer profile: `{profile}`. Experimental proposal, not an accepted contract.\nOwner acceptance and implementation admission: pending.\nStructural validation is not semantic approval, executed evidence, or contract completion.\n\n## Modules\n\n"
        );
        for node in &candidate.presentation.nodes {
            let Node::Module {
                id, title, file, ..
            } = node
            else {
                return Err("Expected validated module".into());
            };
            out.push_str(&format!(
                "- [`{file}`]({file}) — {} (presentation ID: {})\n",
                literal(title)?,
                literal(id)?
            ));
        }
        return Ok(out);
    }
    let description = if candidate.has_expressions() {
        "This view shows authored expression text, declared operand and result types, and exact revision mappings. XS uses one unnumbered article. Flat S numbers its articles in order."
    } else if candidate.uses_v8() {
        "This view shows typed fields and their revision mappings. Numbering follows the declared order. Heading depth is capped at six. The revision mapping records the full path. Attachments appear separately from articles. Their displayed tables and form fields repeat the exact entity source."
    } else if candidate.presentation.uses_v7() {
        "This view shows typed fields and their revision mappings. Upper-level groups follow the selected size. Numbering follows the declared order. Heading depth is capped at six. The revision mapping records the full path."
    } else if candidate.presentation.has_extended_hierarchy() {
        "This labeled view preserves typed fields and their revision mappings. Upper-level groups are optional and ordered as part, chapter, section and subsection. Each kind is numbered among its siblings; articles are numbered across the package. Heading depth is capped at six. The revision mapping records the full path. Grouping is presentation only and does not establish semantic equivalence."
    } else if candidate.presentation.size == "XS" {
        "This view shows one declared operation in an unnumbered article. The article ID links its heading to the revision mapping. All semantic fields and open decisions remain visible."
    } else {
        // Earlier renderer profiles bind these exact bytes.
        "This labeled view preserves all typed fields, explicit S/M hierarchy and exact revision mappings. Authored text is quoted without adding modal words or inferred meaning. JSON literals preserve types and whitespace; escaped directional characters are data. Exact JSON projections repeat the same source, not new requirements. No annexes, sentence anchors, XS/XL profiles, edited-document parser or semantic-equivalence claim are supported."
    };
    let mut out = format!(
        "# Candidate legal-style review\n\nRenderer profile: `{profile}`. Experimental proposal, not an accepted contract.\nOwner acceptance and implementation admission: pending.\nStructural validation is not semantic approval, executed evidence, or contract completion.\n\n{description}\n\n## Package\n\n"
    );
    if candidate.presentation.profile.as_deref() == Some(SUBITEM_PROFILE) {
        out.push_str(SUBITEM_DESCRIPTION);
    } else if candidate.presentation.has_items() {
        if candidate.presentation.has_extended_hierarchy() {
            out.push_str("Item profile: `caac-item-presentation-v1`. Ordered paragraph/item grouping is presentation only, not normative scope, applicability, precedence or inheritance. Paragraph numbers restart at 1 in each article; item numbers restart at 1 in each paragraph. Items require paragraphs. Nested items and subitems are not supported.\n\n");
        } else {
            out.push_str("Item profile: `caac-item-presentation-v1`. Ordered paragraph/item grouping is presentation only, not normative scope, applicability, precedence or inheritance. Paragraph numbers restart at 1 in each article; item numbers restart at 1 in each paragraph. No nested item, subitem or omitted-parent support.\n\n");
        }
    } else if candidate.presentation.has_paragraphs() {
        if candidate.presentation.has_extended_hierarchy() {
            out.push_str("Paragraph profile: `caac-paragraph-presentation-v1`. Ordered paragraph grouping is presentation only, not normative scope, applicability or inheritance. Paragraph numbers restart at 1 in each article. Paragraphs require articles. Nested paragraphs are not supported.\n\n");
        } else {
            out.push_str("Paragraph profile: `caac-paragraph-presentation-v1`. Ordered paragraph grouping is presentation only, not normative scope, applicability or inheritance. Paragraph numbers restart at 1 in each article. No item, nested paragraph or omitted-parent support.\n\n");
        }
    }
    let header = serde_json::json!({"namespace":candidate.semantic.namespace,"format":candidate.format,"size":candidate.presentation.size,"authority":candidate.authority});
    fields(&mut out, &header)?;
    exact(&mut out, &header)?;
    let entities: BTreeMap<_, _> = candidate
        .semantic
        .entities
        .iter()
        .map(|e| (e.id(), e))
        .collect();
    let mut number = 0;
    nodes(
        &mut out,
        &candidate.presentation.nodes,
        &entities,
        &mut number,
        candidate.presentation.size == "XS",
    )?;
    attachments(&mut out, &candidate.presentation.attachments, &entities)?;
    out.push_str("## Unresolved decisions\n\nAll listed decisions remain unresolved, including those with proposals.\n\n");
    for decision in &candidate.semantic.unresolved {
        out.push_str(&format!("### Decision {}\n\n", literal(&decision.id)?));
        fields(&mut out, decision)?;
    }
    exact(&mut out, &candidate.semantic.unresolved)?;
    out.push_str("## Exact source revisions and review mapping\n\nDirect object-content hashes do not prove unchanged effective meaning. Every reference is bound to this full package revision. Presentation IDs are not Markdown anchors or byte spans.\n\n");
    exact(&mut out, &manifest)?;
    out.push_str("## Required evidence\n\nBehavior cases: not run. Exception review: pending. No evidence or acceptance is inherited from another revision.\n");
    Ok(out)
}

pub(super) fn module_reviews(candidate: &Candidate) -> Result<Vec<(String, String)>, String> {
    if candidate.presentation.size != "XL" {
        return Ok(Vec::new());
    }
    let manifest = revisions(candidate)?;
    let entities: BTreeMap<_, _> = candidate
        .semantic
        .entities
        .iter()
        .map(|e| (e.id(), e))
        .collect();
    let decisions: BTreeMap<_, _> = candidate
        .semantic
        .unresolved
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect();
    let mut number = 0;
    let mut reviews = Vec::new();
    let mut total = 0usize;
    for node in &candidate.presentation.nodes {
        let Node::Module {
            id,
            title,
            file,
            decisions: assigned,
            attachments: assigned_attachments,
            ..
        } = node
        else {
            return Err("Expected validated module".into());
        };
        let mut out = format!(
            "# Candidate module review — {}\n\nRenderer profile: `{}`. Experimental proposal, not an accepted contract.\nOwner acceptance and implementation admission: pending.\nStructural validation is not semantic approval, executed evidence, or contract completion.\n\nPresentation ID: {}\n\n[Review index](review.md)\n\n",
            literal(title)?,
            profile(candidate),
            literal(id)?
        );
        if candidate.presentation.has_extended_hierarchy() {
            out.push_str("Upper-level groups are numbered by kind among siblings. Article numbers continue across modules. Heading depth is capped at six. The revision mapping records the full path.\n\n");
        }
        if candidate.presentation.profile.as_deref() == Some(SUBITEM_PROFILE) {
            out.push_str(SUBITEM_DESCRIPTION);
        }
        nodes(&mut out, node.children(), &entities, &mut number, false)?;
        attachments(&mut out, assigned_attachments, &entities)?;
        out.push_str("## Unresolved decisions\n\nAll listed decisions remain unresolved, including those with proposals.\n\n");
        let mut module_decisions = Vec::new();
        for id in assigned {
            let decision = decisions[id.as_str()];
            out.push_str(&format!("### Decision {}\n\n", literal(&decision.id)?));
            fields(&mut out, decision)?;
            module_decisions.push(decision);
        }
        exact(&mut out, &module_decisions)?;
        out.push_str("## Exact source revisions and review mapping\n\nDirect object-content hashes do not prove unchanged effective meaning. Every reference is bound to this full package revision. Presentation IDs are not Markdown anchors or byte spans.\n\n");
        let mapping = serde_json::json!({
            "package_revision": manifest.package_revision,
            "presentation": manifest.presentation.iter().filter(|p| &p.file == file).collect::<Vec<_>>()
        });
        exact(&mut out, &mapping)?;
        out.push_str("## Required evidence\n\nBehavior cases: not run. Exception review: pending. No evidence or acceptance is inherited from another revision.\n");
        total = total.saturating_add(out.len());
        if total > MAX_ARTIFACT {
            return Err("Module reviews exceed the artifact size limit".into());
        }
        reviews.push((file.clone(), out));
    }
    Ok(reviews)
}
