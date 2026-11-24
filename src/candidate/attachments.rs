use super::{Entity, Literal, Presentation, Type, nonblank, reference, unique};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AttachmentStatus {
    Binding,
    Illustrative,
}

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AttachmentRef {
    pub(super) id: String,
    pub(super) usage: AttachmentStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct FormField {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) value_type: Type,
    pub(super) required: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum SentencePortion {
    First,
    Second,
}

impl Entity {
    fn attachment_status(&self) -> Option<AttachmentStatus> {
        match self {
            Self::Annex { status, .. } | Self::Form { status, .. } => Some(*status),
            _ => None,
        }
    }
}

pub(super) fn validate_references(
    entities: &BTreeMap<&str, &Entity>,
    annexes: &[AttachmentRef],
    forms: &[AttachmentRef],
) -> Result<(), String> {
    for (kind, refs) in [("annex", annexes), ("form", forms)] {
        unique(refs.iter().map(|link| link.id.as_str()))?;
        for link in refs {
            let target = reference(entities, &link.id, kind)?;
            if target.attachment_status() != Some(link.usage) {
                return Err(format!(
                    "Attachment reference {} must match its status",
                    link.id
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_annex(
    title: &str,
    columns: &[String],
    rows: &[Vec<Literal>],
) -> Result<(), String> {
    nonblank(title)?;
    if columns.is_empty() || rows.is_empty() || rows.iter().any(|row| row.len() != columns.len()) {
        return Err("Annex requires columns and nonempty rows matching the column count".into());
    }
    let mut names = BTreeSet::new();
    for name in columns {
        nonblank(name)?;
        if !names.insert(name) {
            return Err("Annex column names must be distinct".into());
        }
    }
    Ok(())
}

pub(super) fn validate_form(title: &str, fields: &[FormField]) -> Result<(), String> {
    nonblank(title)?;
    if fields.is_empty() {
        return Err("Form requires at least one field".into());
    }
    unique(fields.iter().map(|field| field.id.as_str()))?;
    for field in fields {
        nonblank(&field.label)?;
    }
    Ok(())
}

pub(super) fn validate_sentence_parts(entities: &BTreeMap<&str, &Entity>) -> Result<(), String> {
    let mut lengths = BTreeMap::<&str, [usize; 2]>::new();
    for entity in entities.values() {
        let Entity::SentencePart {
            clause,
            portion,
            text,
            ..
        } = entity
        else {
            continue;
        };
        nonblank(text)?;
        let Entity::Clause { text: parent, .. } = reference(entities, clause, "clause")? else {
            unreachable!()
        };
        let (index, matches) = match portion {
            SentencePortion::First => (0, parent.starts_with(text)),
            SentencePortion::Second => (1, parent.ends_with(text)),
        };
        if !matches || text.len() >= parent.len() {
            return Err("Sentence part must match a proper prefix or suffix of its clause".into());
        }
        let slots = lengths.entry(clause.as_str()).or_default();
        if slots[index] != 0 {
            return Err("Clause has more than one sentence part in the same position".into());
        }
        slots[index] = text.len();
        if slots[0] + slots[1] > parent.len() {
            return Err("Sentence parts of the same clause overlap".into());
        }
    }
    Ok(())
}

pub(super) fn validate_exception_sentence(
    entities: &BTreeMap<&str, &Entity>,
    target: &str,
    sentence: &str,
) -> Result<(), String> {
    let Entity::SentencePart { clause, .. } = reference(entities, sentence, "sentence-part")?
    else {
        unreachable!()
    };
    if clause != target {
        return Err("Exception sentence must belong to its target clause".into());
    }
    Ok(())
}

pub(super) fn validate_placements<'a>(
    presentation: &Presentation,
    entities: impl IntoIterator<Item = &'a Entity>,
) -> Result<(), String> {
    let attachments: BTreeSet<_> = presentation.attachment_ids().collect();
    for entity in entities {
        if entity.attachment_status().is_some() != attachments.contains(entity.id()) {
            return Err("Annexes and forms require a separate attachment placement".into());
        }
    }
    Ok(())
}
