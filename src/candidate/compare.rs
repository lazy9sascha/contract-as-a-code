//! Conservative edited-review comparison. Proposals never apply themselves.
use super::*;
use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue, Sourcepos},
    parse_document,
};
use serde_json::{Value, json};

const MAX_REVIEW: usize = 4 * 1024 * 1024;
const MAX_NODES: usize = 100_000;
#[derive(Clone)]
enum Slot {
    Field(String, String),
    Entity(String),
}

// Preserve duplicate-key rejection at every nesting level before converting to Value.
struct Strict(Value);
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Strict;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("JSON without duplicate keys")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Strict, E> {
                Ok(Strict(json!(v)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Strict, A::Error> {
                let mut v = vec![];
                while let Some(Strict(x)) = a.next_element()? {
                    v.push(x)
                }
                Ok(Strict(Value::Array(v)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut a: A) -> Result<Strict, A::Error> {
                let mut v = serde_json::Map::new();
                while let Some((k, Strict(x))) = a.next_entry::<String, Strict>()? {
                    if v.insert(k, x).is_some() {
                        return Err(serde::de::Error::custom("Duplicate JSON key"));
                    }
                }
                Ok(Strict(Value::Object(v)))
            }
        }
        d.deserialize_any(Visitor)
    }
}
pub(super) fn strict(text: &str) -> Result<Value, String> {
    serde_json::from_str::<Strict>(text)
        .map(|s| s.0)
        .map_err(|e| e.to_string())
}
fn range(text: &str, p: Sourcepos) -> Result<(usize, usize), String> {
    let lines: Vec<_> = text.split_inclusive('\n').collect();
    if p.start.line == 0 || p.end.line > lines.len() {
        return Err("Unsupported source position".into());
    }
    let start = lines[..p.start.line - 1]
        .iter()
        .map(|s| s.len())
        .sum::<usize>()
        + p.start.column.saturating_sub(1);
    let end = lines[..p.end.line - 1]
        .iter()
        .map(|s| s.len())
        .sum::<usize>()
        + p.end.column;
    if start > end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err("Invalid source byte boundary".into());
    }
    Ok((start, end))
}
fn skeleton(text: &str, mut ranges: Vec<(usize, usize)>) -> Result<String, String> {
    ranges.sort_unstable();
    let mut out = String::new();
    let mut from = 0;
    for (start, end) in ranges {
        if start < from {
            return Err("Overlapping source slots".into());
        }
        out.push_str(&text[from..start]);
        out.push_str("<CAAC-SLOT>");
        from = end;
    }
    out.push_str(&text[from..]);
    Ok(out)
}
fn heading_id<'a>(node: &'a AstNode<'a>, entities: &BTreeMap<String, Value>) -> Option<String> {
    if !matches!(node.data.borrow().value, NodeValue::Heading(_)) {
        return None;
    }
    let id = node.children().find_map(|n| {
        if let NodeValue::Code(c) = &n.data.borrow().value {
            serde_json::from_str::<String>(&c.literal).ok()
        } else {
            None
        }
    })?;
    let e = entities.get(&id)?;
    let title = node.children().find_map(|n| {
        if let NodeValue::Text(t) = &n.data.borrow().value {
            Some(t.to_string())
        } else {
            None
        }
    })?;
    let kind = e["kind"].as_str()?;
    (title.to_lowercase() == format!("{kind} · ")).then_some(id)
}
fn topology<'a>(nodes: &[&'a AstNode<'a>]) -> Vec<Option<usize>> {
    let mut indexes = BTreeMap::new();
    nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let parent = n
                .parent()
                .and_then(|p| indexes.get(&(p as *const _ as usize)).copied());
            indexes.insert(*n as *const _ as usize, i);
            parent
        })
        .collect()
}
fn slots<'a>(
    nodes: &[&'a AstNode<'a>],
    entities: &BTreeMap<String, Value>,
) -> BTreeMap<usize, Slot> {
    let mut out = BTreeMap::new();
    let mut current = None;
    for (i, node) in nodes.iter().enumerate() {
        if matches!(node.data.borrow().value, NodeValue::Heading(_)) {
            current = heading_id(node, entities);
        }
        let Some(id) = &current else { continue };
        match &node.data.borrow().value {
            NodeValue::Code(c) => {
                let label = node
                    .parent()
                    .and_then(|p| {
                        p.children()
                            .find(|n| matches!(n.data.borrow().value, NodeValue::Strong))
                    })
                    .and_then(|s| s.first_child())
                    .and_then(|n| {
                        if let NodeValue::Text(t) = &n.data.borrow().value {
                            Some(t.to_string())
                        } else {
                            None
                        }
                    });
                if let Some(label) = label {
                    for (key, value) in entities[id].as_object().unwrap() {
                        if label == format!("{}:", super::render::label(key))
                            && strict(&c.literal).ok().as_ref() == Some(value)
                        {
                            out.insert(i, Slot::Field(id.clone(), key.clone()));
                        }
                    }
                }
            }
            NodeValue::CodeBlock(c)
                if c.info == "json" && strict(&c.literal).ok().as_ref() == Some(&entities[id]) =>
            {
                out.insert(i, Slot::Entity(id.clone()));
            }
            _ => {}
        }
    }
    out
}
fn conflict(message: &str, position: Option<Sourcepos>) -> Value {
    json!({"reason":message,"edited_location":position.map(|p|json!({"start_line":p.start.line,"start_column":p.start.column,"end_line":p.end.line,"end_column":p.end.column})),"authority":"unverified"})
}

/// Verify a source triple, compare a bounded edit, and publish only pending proposals/conflicts.
pub fn compare_review(
    candidate_path: &Path,
    provenance_path: &Path,
    review_path: &Path,
    edited_path: &Path,
    destination: &Path,
) -> Result<String, String> {
    crate::output::require_absent(destination)?;
    let paths = [candidate_path, provenance_path, review_path, edited_path];
    let limits = [MAX_INPUT, MAX_ARTIFACT, MAX_REVIEW, MAX_REVIEW];
    let out_key = crate::output::destination_key(destination)?;
    let mut bytes = vec![];
    for (path, limit) in paths.iter().zip(limits) {
        let key = crate::output::destination_key(path)?;
        if key.starts_with(&out_key) || out_key.starts_with(&key) {
            return Err("Output overlaps input".into());
        }
        bytes.push(crate::source_intake::read_regular(path, limit)?);
    }
    let proposal = proposal_from_bytes(&bytes)?;
    let status = proposal["status"]
        .as_str()
        .ok_or("Missing comparison status")?;
    let encoded = json(&proposal)?;
    if encoded.len() > MAX_ARTIFACT {
        return Err("Proposal exceeds artifact budget".into());
    }
    for ((path, limit), expected) in paths.iter().zip(limits).zip(&bytes) {
        if crate::source_intake::read_regular(path, limit)? != *expected {
            return Err("Input changed before proposal publication".into());
        }
    }
    fs::create_dir(destination).map_err(|e| e.to_string())?;
    crate::output::write_new(&destination.join("proposal.json"), &encoded)?;
    Ok(status.into())
}

pub(super) fn proposal_from_bytes(bytes: &[Vec<u8>]) -> Result<Value, String> {
    if bytes.len() != 4
        || bytes[0].len() > MAX_INPUT
        || bytes[1].len() > MAX_ARTIFACT
        || bytes[2].len() > MAX_REVIEW
        || bytes[3].len() > MAX_REVIEW
    {
        return Err("Comparison snapshot exceeds limits".into());
    }
    verify_bytes(&bytes[0], &bytes[1], &bytes[2])?;
    let candidate: Candidate = serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
    if candidate.presentation.size == "XL" {
        return Err("XL module review edits require an explicit candidate change".into());
    }
    let original = std::str::from_utf8(&bytes[2]).map_err(|e| e.to_string())?;
    let edited = std::str::from_utf8(&bytes[3]).map_err(|e| e.to_string())?;
    let entities: BTreeMap<String, Value> = candidate
        .semantic
        .entities
        .iter()
        .map(|e| {
            Ok((
                e.id().to_string(),
                serde_json::to_value(e).map_err(|e| e.to_string())?,
            ))
        })
        .collect::<Result<_, String>>()?;
    let a = Arena::new();
    let b = Arena::new();
    let opts = Options::default();
    let old = parse_document(&a, original, &opts);
    let new = parse_document(&b, edited, &opts);
    let old: Vec<_> = old.descendants().take(MAX_NODES + 1).collect();
    let new: Vec<_> = new.descendants().take(MAX_NODES + 1).collect();
    if old.len() > MAX_NODES || new.len() > MAX_NODES {
        return Err("Review AST exceeds comparison node budget".into());
    }
    let slots = slots(&old, &entities);
    let mut conflicts = vec![];
    let mut replacements = vec![];
    let same_topology = topology(&old) == topology(&new);
    let ast_equal = same_topology
        && old.len() == new.len()
        && old
            .iter()
            .zip(&new)
            .all(|(a, b)| a.data.borrow().value == b.data.borrow().value);
    let mut status = if original == edited {
        "unchanged"
    } else if ast_equal {
        "presentation-only"
    } else {
        "proposed"
    };
    let whitespace_only = original
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .eq(edited.chars().filter(|c| !c.is_ascii_whitespace()));
    if original != edited && !(ast_equal && whitespace_only) {
        let mut fields = entities.clone();
        let mut exacts = entities.clone();
        let mut ranges_old = vec![];
        let mut ranges_new = vec![];
        let mut seen_fields = BTreeSet::new();
        let mut seen_exact = BTreeSet::new();
        if old.len() != new.len() || !same_topology {
            conflicts.push(conflict(
                "Changed document structure; no supported alignment",
                None,
            ));
        } else {
            for (i, (left, right)) in old.iter().zip(&new).enumerate() {
                let l = left.data.borrow();
                let r = right.data.borrow();
                if let Some(slot) = slots.get(&i) {
                    let text = match (&l.value, &r.value) {
                        (NodeValue::Code(_), NodeValue::Code(c)) => Some(c.literal.as_str()),
                        (NodeValue::CodeBlock(_), NodeValue::CodeBlock(c)) if c.info == "json" => {
                            Some(c.literal.as_str())
                        }
                        _ => None,
                    };
                    let parsed = text
                        .ok_or("Slot kind changed")
                        .and_then(|s| strict(s).map_err(|_| "Invalid or duplicate-key JSON"));
                    match parsed {
                        Ok(value) => {
                            match slot {
                                Slot::Field(id, key) => {
                                    fields.get_mut(id).unwrap()[key] = value;
                                    seen_fields.insert((id.clone(), key.clone()));
                                }
                                Slot::Entity(id) => {
                                    exacts.insert(id.clone(), value);
                                    seen_exact.insert(id.clone());
                                }
                            }
                            ranges_old.push(range(original, l.sourcepos)?);
                            ranges_new.push(range(edited, r.sourcepos)?);
                        }
                        Err(e) => conflicts.push(conflict(e, Some(r.sourcepos))),
                    }
                } else if l.value != r.value {
                    conflicts.push(conflict(
                        "Unsupported edit outside existing entity fields",
                        Some(r.sourcepos),
                    ));
                }
            }
        }
        if conflicts.is_empty() && skeleton(original, ranges_old)? != skeleton(edited, ranges_new)?
        {
            conflicts.push(conflict("Raw source changed outside approved field slots (including parser-ignored content)",None));
        }
        let mut hypothetical: Candidate =
            serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
        for e in &mut hypothetical.semantic.entities {
            let id = e.id().to_string();
            if !seen_exact.contains(&id)
                || entities[&id]
                    .as_object()
                    .unwrap()
                    .keys()
                    .any(|k| !seen_fields.contains(&(id.clone(), k.clone())))
            {
                conflicts.push(conflict("Missing one-to-one entity field projection", None));
                continue;
            }
            if fields[&id] != exacts[&id] {
                conflicts.push(conflict(
                    "Labeled and exact entity representations disagree",
                    None,
                ));
                continue;
            }
            match serde_json::from_value::<Entity>(exacts[&id].clone()) {
                Ok(value) if value.id() == e.id() && value.kind() == e.kind() => {
                    if exacts[&id] != entities[&id] {
                        replacements.push(json!({"id":id,"kind":e.kind(),"before":entities[&id],"after":exacts[&id],"authority":"pending"}));
                    }
                    *e = value;
                }
                _ => conflicts.push(conflict(
                    "Unsupported entity type, ID, field or value",
                    None,
                )),
            }
        }
        if conflicts.is_empty()
            && let Err(e) = validate(&hypothetical)
        {
            conflicts.push(conflict(
                &format!("Hypothetical candidate validation failed: {e}"),
                None,
            ));
        }
        if !conflicts.is_empty() {
            status = "conflict";
            replacements.clear();
        } else if replacements.is_empty() {
            status = "presentation-only";
        }
    }
    let revisions = revisions(&candidate)?;
    for r in &mut replacements {
        r["expected_revision"] = json!(
            revisions
                .objects
                .iter()
                .find(|o| o.id == r["id"].as_str().unwrap())
                .ok_or("Missing revision")?
                .revision
        );
    }
    let proposal = json!({"format":"caac-review-proposal-v1","renderer_profile":super::render::profile(&candidate),"status":status,"source_candidate_sha256":digest(&bytes[0]),"source_provenance_sha256":digest(&bytes[1]),"source_review_sha256":digest(&bytes[2]),"edited_review_sha256":digest(&bytes[3]),"source_revisions":revisions,"replacements":replacements,"conflicts":conflicts,"owner_acceptance":"pending","implementation_admission":"pending","applied":false,"semantic_equivalence":"not-established","presentation_only_meaning":"No supported typed change; raw edits remain uninterpreted, not approved or discarded","raw_review_change":if original==edited{Value::Null}else{json!({"before":original,"after":edited})}});
    Ok(proposal)
}
