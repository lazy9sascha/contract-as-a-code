//! Derived identities and exact bindings for the finite candidate profile.
use super::{AttachmentStatus, Candidate, Case, CaseTarget, Entity, Node, digest, json, validate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const FORMAT: &str = "caac-candidate-revisions-v1";
const ENCODING: &str = "caac-candidate-v1/serde-json-pretty-v1";
const MAX_MANIFEST: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectKind {
    Definition,
    Clause,
    Operation,
    Expression,
    Exception,
    Case,
    Decision,
    Annex,
    Form,
    SentencePart,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObjectRevision {
    pub kind: ObjectKind,
    pub id: String,
    /// Direct body content, not transitive effective meaning.
    pub revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExactReference {
    pub namespace: String,
    pub package_revision: String,
    pub kind: ObjectKind,
    pub id: String,
    pub entity_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceField {
    DefinitionRefs,
    ClauseRefs,
    Target,
    Operation,
    Expression,
    ExceptionRefs,
    ProposalEntityRefs,
    AnnexRefs,
    FormRefs,
    SentenceClause,
    Sentence,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReferenceBinding {
    pub source: ExactReference,
    pub field: ReferenceField,
    /// Null for scalar fields, zero-based for ordered reference lists.
    pub index: Option<usize>,
    pub target: ExactReference,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::present_nonnull"
    )]
    pub attachment_usage: Option<AttachmentStatus>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ReviewLocation {
    UnnumberedArticle {
        node_id: String,
    },
    Article {
        node_id: String,
        chapter_id: Option<String>,
        number: usize,
    },
    Paragraph {
        node_id: String,
        article_id: String,
        chapter_id: Option<String>,
        article_number: usize,
        number: usize,
    },
    Item {
        node_id: String,
        paragraph_id: String,
        article_id: String,
        chapter_id: Option<String>,
        article_number: usize,
        paragraph_number: usize,
        number: usize,
    },
    Subitem {
        node_id: String,
        item_id: String,
        paragraph_id: String,
        article_id: String,
        chapter_id: Option<String>,
        article_number: usize,
        paragraph_number: usize,
        item_number: usize,
        number: usize,
    },
    Attachment {
        node_id: String,
        number: usize,
    },
    UnresolvedDecision {
        index: usize,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresentationAncestor {
    pub kind: String,
    pub node_id: String,
    pub number: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresentationBinding {
    pub target: ExactReference,
    pub file: String,
    pub location: ReviewLocation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ancestors: Vec<PresentationAncestor>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RevisionManifest {
    pub format: String,
    pub encoding: String,
    pub namespace: String,
    pub package_revision: String,
    pub semantic_revision: String,
    pub presentation_revision: String,
    pub objects: Vec<ObjectRevision>,
    pub references: Vec<ReferenceBinding>,
    pub presentation: Vec<PresentationBinding>,
}
fn revision(domain: &str, value: &impl Serialize) -> Result<String, String> {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend(json(value)?);
    Ok(format!("sha256:{}", digest(&bytes)))
}
fn object_kind(e: &Entity) -> ObjectKind {
    match e {
        Entity::Definition { .. } => ObjectKind::Definition,
        Entity::Clause { .. } => ObjectKind::Clause,
        Entity::Operation { .. } => ObjectKind::Operation,
        Entity::Exception { .. } => ObjectKind::Exception,
        Entity::Case(_) => ObjectKind::Case,
        Entity::Annex { .. } => ObjectKind::Annex,
        Entity::Form { .. } => ObjectKind::Form,
        Entity::SentencePart { .. } => ObjectKind::SentencePart,
        Entity::Expression { .. } => ObjectKind::Expression,
    }
}
/// Recompute a derived manifest. Never accepts supplied hashes or acceptance claims.
pub fn revisions(candidate: &Candidate) -> Result<RevisionManifest, String> {
    validate(candidate)?;
    let namespace = &candidate.semantic.namespace;
    let package_revision = revision("caac.package.v1", candidate)?;
    let semantic_revision = revision("caac.semantic.v1", &candidate.semantic)?;
    let presentation_revision = revision("caac.presentation.v1", &candidate.presentation)?;
    let mut objects = Vec::new();
    for entity in &candidate.semantic.entities {
        objects.push(ObjectRevision {
            kind: object_kind(entity),
            id: entity.id().into(),
            revision: revision(
                "caac.entity.v1",
                &serde_json::json!({"namespace":namespace,"body":entity}),
            )?,
        });
    }
    for decision in &candidate.semantic.unresolved {
        objects.push(ObjectRevision {
            kind: ObjectKind::Decision,
            id: decision.id.clone(),
            revision: revision(
                "caac.decision.v1",
                &serde_json::json!({"namespace":namespace,"body":decision}),
            )?,
        });
    }
    let addresses: BTreeMap<_, _> = objects
        .iter()
        .map(|object| {
            (
                object.id.as_str(),
                ExactReference {
                    namespace: namespace.clone(),
                    package_revision: package_revision.clone(),
                    kind: object.kind.clone(),
                    id: object.id.clone(),
                    entity_revision: object.revision.clone(),
                },
            )
        })
        .collect();
    let address = |id: &str| {
        addresses
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Revision target is missing: {id}"))
    };
    let mut references = Vec::new();
    let mut edge = |source: &str,
                    field: ReferenceField,
                    index: Option<usize>,
                    target: &str,
                    attachment_usage: Option<AttachmentStatus>|
     -> Result<(), String> {
        references.push(ReferenceBinding {
            source: address(source)?,
            field,
            index,
            target: address(target)?,
            attachment_usage,
        });
        Ok(())
    };
    for e in &candidate.semantic.entities {
        match e {
            Entity::Definition { .. } => {}
            Entity::Clause {
                id,
                definition_refs,
                annex_refs,
                form_refs,
                ..
            } => {
                for (index, target) in definition_refs.iter().enumerate() {
                    edge(
                        id,
                        ReferenceField::DefinitionRefs,
                        Some(index),
                        target,
                        None,
                    )?;
                }
                for (field, refs) in [
                    (ReferenceField::AnnexRefs, annex_refs),
                    (ReferenceField::FormRefs, form_refs),
                ] {
                    for (index, target) in refs.iter().enumerate() {
                        edge(
                            id,
                            field.clone(),
                            Some(index),
                            &target.id,
                            Some(target.usage),
                        )?;
                    }
                }
            }
            Entity::Operation {
                id, clause_refs, ..
            }
            | Entity::Expression {
                id, clause_refs, ..
            } => {
                for (index, target) in clause_refs.iter().enumerate() {
                    edge(id, ReferenceField::ClauseRefs, Some(index), target, None)?;
                }
            }
            Entity::Exception {
                id,
                target,
                sentence,
                ..
            } => {
                edge(id, ReferenceField::Target, None, target, None)?;
                if let Some(sentence) = sentence {
                    edge(id, ReferenceField::Sentence, None, sentence, None)?;
                }
            }
            Entity::Case(Case {
                id,
                target,
                clause_refs,
                exception_refs,
                ..
            }) => {
                let (field, target) = match target {
                    CaseTarget::Operation(id) => (ReferenceField::Operation, id),
                    CaseTarget::Expression(id) => (ReferenceField::Expression, id),
                };
                edge(id, field, None, target, None)?;
                for (index, target) in clause_refs.iter().enumerate() {
                    edge(id, ReferenceField::ClauseRefs, Some(index), target, None)?;
                }
                for (index, target) in exception_refs.iter().enumerate() {
                    edge(id, ReferenceField::ExceptionRefs, Some(index), target, None)?;
                }
            }
            Entity::Annex { .. } | Entity::Form { .. } => {}
            Entity::SentencePart { id, clause, .. } => {
                edge(id, ReferenceField::SentenceClause, None, clause, None)?;
            }
        }
    }
    for d in &candidate.semantic.unresolved {
        if let Some(proposal) = &d.proposal {
            for (index, target) in proposal.entity_refs.iter().enumerate() {
                edge(
                    &d.id,
                    ReferenceField::ProposalEntityRefs,
                    Some(index),
                    target,
                    None,
                )?;
            }
        }
    }
    let mut presentation = Vec::new();
    let mut number = 0;
    let includes_ancestors = candidate.presentation.has_extended_hierarchy()
        || candidate.presentation.uses_v7()
        || candidate.uses_v8();
    let mut article = |node: &Node,
                       number: usize,
                       path: &[super::hierarchy::Ancestor<'_>],
                       file: &str|
     -> Result<(), String> {
        if node.upper().is_some() {
            return Ok(());
        }
        let Node::Article {
            id,
            entities,
            paragraphs,
            ..
        } = node
        else {
            return Err("Expected validated article".into());
        };
        let chapter_id = path
            .iter()
            .find(|ancestor| ancestor.kind == "chapter")
            .map(|ancestor| ancestor.id.to_owned());
        let ancestors: Vec<_> = if includes_ancestors {
            path.iter()
                .map(|ancestor| PresentationAncestor {
                    kind: ancestor.kind.into(),
                    node_id: ancestor.id.into(),
                    number: ancestor.number,
                })
                .collect()
        } else {
            Vec::new()
        };
        for entity in entities.iter().flatten() {
            presentation.push(PresentationBinding {
                target: address(entity)?,
                file: file.into(),
                location: if candidate.presentation.size == "XS" {
                    ReviewLocation::UnnumberedArticle {
                        node_id: id.clone(),
                    }
                } else {
                    ReviewLocation::Article {
                        node_id: id.clone(),
                        chapter_id: chapter_id.clone(),
                        number,
                    }
                },
                ancestors: ancestors.clone(),
            });
        }
        if let Some(paragraphs) = paragraphs {
            for (index, paragraph) in paragraphs.iter().enumerate() {
                for entity in paragraph.entities.iter().flatten() {
                    presentation.push(PresentationBinding {
                        target: address(entity)?,
                        file: file.into(),
                        location: ReviewLocation::Paragraph {
                            node_id: paragraph.id.clone(),
                            article_id: id.clone(),
                            chapter_id: chapter_id.clone(),
                            article_number: number,
                            number: index + 1,
                        },
                        ancestors: ancestors.clone(),
                    });
                }
                if let Some(items) = &paragraph.items {
                    for (item_index, item) in items.iter().enumerate() {
                        for entity in item.entities.iter().flatten() {
                            presentation.push(PresentationBinding {
                                target: address(entity)?,
                                file: file.into(),
                                location: ReviewLocation::Item {
                                    node_id: item.id.clone(),
                                    paragraph_id: paragraph.id.clone(),
                                    article_id: id.clone(),
                                    chapter_id: chapter_id.clone(),
                                    article_number: number,
                                    paragraph_number: index + 1,
                                    number: item_index + 1,
                                },
                                ancestors: ancestors.clone(),
                            });
                        }
                        if let Some(subitems) = &item.subitems {
                            for (subitem_index, subitem) in subitems.iter().enumerate() {
                                for entity in &subitem.entities {
                                    presentation.push(PresentationBinding {
                                        target: address(entity)?,
                                        file: file.into(),
                                        location: ReviewLocation::Subitem {
                                            node_id: subitem.id.clone(),
                                            item_id: item.id.clone(),
                                            paragraph_id: paragraph.id.clone(),
                                            article_id: id.clone(),
                                            chapter_id: chapter_id.clone(),
                                            article_number: number,
                                            paragraph_number: index + 1,
                                            item_number: item_index + 1,
                                            number: subitem_index + 1,
                                        },
                                        ancestors: ancestors.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    };
    if candidate.presentation.size == "XL" {
        for node in &candidate.presentation.nodes {
            let Node::Module { file, .. } = node else {
                return Err("Expected validated module".into());
            };
            super::hierarchy::walk(node.children(), &mut number, &mut |node, number, path| {
                article(node, number, path, file)
            })?;
        }
    } else {
        super::hierarchy::walk(
            &candidate.presentation.nodes,
            &mut number,
            &mut |node, number, path| article(node, number, path, "review.md"),
        )?;
    }
    let mut bind_attachments = |ids: &[String], file: &str| -> Result<(), String> {
        for (index, attachment) in ids.iter().enumerate() {
            presentation.push(PresentationBinding {
                target: address(attachment)?,
                file: file.into(),
                location: ReviewLocation::Attachment {
                    node_id: attachment.clone(),
                    number: index + 1,
                },
                ancestors: Vec::new(),
            });
        }
        Ok(())
    };
    if candidate.presentation.size == "XL" {
        for node in &candidate.presentation.nodes {
            let Node::Module {
                file, attachments, ..
            } = node
            else {
                return Err("Expected validated module".into());
            };
            bind_attachments(attachments, file)?;
        }
    } else {
        bind_attachments(&candidate.presentation.attachments, "review.md")?;
    }
    if candidate.presentation.size == "XL" {
        for node in &candidate.presentation.nodes {
            let Node::Module {
                file, decisions, ..
            } = node
            else {
                return Err("Expected validated module".into());
            };
            for (index, decision) in decisions.iter().enumerate() {
                presentation.push(PresentationBinding {
                    target: address(decision)?,
                    file: file.clone(),
                    location: ReviewLocation::UnresolvedDecision { index },
                    ancestors: Vec::new(),
                });
            }
        }
    } else {
        for (index, d) in candidate.semantic.unresolved.iter().enumerate() {
            presentation.push(PresentationBinding {
                target: address(&d.id)?,
                file: "review.md".into(),
                location: ReviewLocation::UnresolvedDecision { index },
                ancestors: Vec::new(),
            });
        }
    }
    let manifest = RevisionManifest {
        format: manifest_format(candidate).into(),
        encoding: ENCODING.into(),
        namespace: namespace.clone(),
        package_revision,
        semantic_revision,
        presentation_revision,
        objects,
        references,
        presentation,
    };
    if json(&manifest)?.len() > MAX_MANIFEST {
        return Err("Derived revision manifest exceeds 16 MiB limit".into());
    }
    Ok(manifest)
}
/// Compare the supplied precondition, without repairing or replacing stale bindings.
pub fn verify_revisions(candidate: &Candidate, expected: &RevisionManifest) -> Result<(), String> {
    if expected.format != manifest_format(candidate) || expected.encoding != ENCODING {
        return Err("Unsupported revision manifest version or encoding".into());
    }
    if &revisions(candidate)? != expected {
        return Err("Revision manifest does not match the exact typed candidate snapshot".into());
    }
    Ok(())
}

fn manifest_format(candidate: &Candidate) -> &'static str {
    if candidate.has_expressions() {
        "caac-candidate-revisions-v9"
    } else if candidate.uses_v8() {
        "caac-candidate-revisions-v8"
    } else if candidate.presentation.uses_v7() {
        "caac-candidate-revisions-v7"
    } else if candidate.presentation.has_extended_hierarchy() {
        "caac-candidate-revisions-v6"
    } else if candidate.presentation.size == "XL" {
        "caac-candidate-revisions-v5"
    } else if candidate.presentation.size == "XS" {
        "caac-candidate-revisions-v4"
    } else if candidate.presentation.has_items() {
        "caac-candidate-revisions-v3"
    } else if candidate.presentation.has_paragraphs() {
        "caac-candidate-revisions-v2"
    } else {
        FORMAT
    }
}
