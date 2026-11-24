//! Experimental, explicit-data candidate authoring; never an acceptance/controller gate.
mod attachments;
mod compare;
mod hierarchy;
mod module_artifacts;
mod presentation;
mod trace;
pub use trace::verify_trace;
mod revise;
pub use compare::compare_review;
pub use presentation::revise_presentation;
pub use revise::revise;
mod render;
pub mod revisions;
pub use revisions::{RevisionManifest, revisions, verify_revisions};

pub use attachments::AttachmentStatus;
use attachments::{AttachmentRef, FormField, SentencePortion};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, Read, Write},
    path::Path,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    format: String,
    semantic: Semantic,
    presentation: Presentation,
    authority: Authority,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Authority {
    owner_acceptance: String,
    implementation_admission: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Semantic {
    namespace: String,
    entities: Vec<Entity>,
    unresolved: Vec<Decision>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    id: String,
    question: String,
    required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    proposal: Option<DecisionProposal>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionProposal {
    text: String,
    entity_refs: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    size: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    profile: Option<String>,
    nodes: Vec<Node>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Node {
    Module {
        id: String,
        title: String,
        file: String,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        articles: Option<Vec<Node>>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        children: Option<Vec<Node>>,
        decisions: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attachments: Vec<String>,
    },
    Part {
        id: String,
        title: String,
        children: Vec<Node>,
    },
    Chapter {
        id: String,
        title: String,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        articles: Option<Vec<Node>>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        children: Option<Vec<Node>>,
    },
    Section {
        id: String,
        title: String,
        children: Vec<Node>,
    },
    Subsection {
        id: String,
        title: String,
        children: Vec<Node>,
    },
    Article {
        id: String,
        title: String,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        entities: Option<Vec<String>>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        paragraphs: Option<Vec<Paragraph>>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Paragraph {
    id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    entities: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    items: Option<Vec<Item>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    entities: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    subitems: Option<Vec<Subitem>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Subitem {
    id: String,
    entities: Vec<String>,
}
fn present_nonnull<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}
const PARAGRAPH_PROFILE: &str = "caac-paragraph-presentation-v1";
const ITEM_PROFILE: &str = "caac-item-presentation-v1";
const SUBITEM_PROFILE: &str = "caac-subitem-presentation-v1";
impl Presentation {
    fn attachment_ids(&self) -> impl Iterator<Item = &str> {
        self.attachments
            .iter()
            .chain(
                self.nodes
                    .iter()
                    .filter_map(|node| match node {
                        Node::Module { attachments, .. } => Some(attachments),
                        _ => None,
                    })
                    .flatten(),
            )
            .map(String::as_str)
    }
    fn has_extended_hierarchy(&self) -> bool {
        self.size == "L"
            || self.has_optional_m_levels()
            || (self.size == "XL"
                && self.nodes.iter().any(|node| {
                    matches!(
                        node,
                        Node::Module {
                            children: Some(_),
                            ..
                        }
                    )
                }))
    }
    fn has_optional_m_levels(&self) -> bool {
        self.size == "M"
            && self
                .nodes
                .iter()
                .any(|node| !matches!(node, Node::Chapter { children: None, .. }))
    }
    fn has_subitems(&self) -> bool {
        self.nodes.iter().any(|node| {
            node.any_article(&|node| {
                let Node::Article {
                    paragraphs: Some(paragraphs),
                    ..
                } = node
                else {
                    return false;
                };
                paragraphs
                    .iter()
                    .filter_map(|paragraph| paragraph.items.as_ref())
                    .flatten()
                    .any(|item| item.subitems.is_some())
            })
        })
    }
    fn uses_v7(&self) -> bool {
        self.has_optional_m_levels() || self.profile.as_deref() == Some(SUBITEM_PROFILE)
    }
    fn has_items(&self) -> bool {
        self.nodes.iter().any(|node| node.any_article(&|node| {
            matches!(node, Node::Article { paragraphs: Some(groups), .. } if groups.iter().any(|p| p.items.is_some()))
        }))
    }
    fn is_single_flat_article(&self) -> bool {
        matches!(
            self.nodes.as_slice(),
            [Node::Article {
                entities: Some(_),
                paragraphs: None,
                ..
            }]
        )
    }
    fn validate_expression_layout(&self) -> Result<(), String> {
        if !matches!(self.size.as_str(), "XS" | "S")
            || self.profile.is_some()
            || self.has_paragraphs()
        {
            return Err("Expressions require XS or flat S presentation".into());
        }
        if self.size == "XS" && !self.is_single_flat_article() {
            return Err("The XS profile requires one flat article".into());
        }
        Ok(())
    }
    fn has_paragraphs(&self) -> bool {
        self.nodes.iter().any(|node| {
            node.any_article(&|node| {
                matches!(
                    node,
                    Node::Article {
                        paragraphs: Some(_),
                        ..
                    }
                )
            })
        })
    }
}
impl Node {
    fn children(&self) -> &[Node] {
        match self {
            Self::Module {
                articles, children, ..
            }
            | Self::Chapter {
                articles, children, ..
            } => articles.as_deref().or(children.as_deref()).unwrap_or(&[]),
            Self::Part { children, .. }
            | Self::Section { children, .. }
            | Self::Subsection { children, .. } => children,
            Self::Article { .. } => &[],
        }
    }
    fn upper(&self) -> Option<(&'static str, &str, &str)> {
        match self {
            Self::Part { id, title, .. } => Some(("part", id, title)),
            Self::Chapter { id, title, .. } => Some(("chapter", id, title)),
            Self::Section { id, title, .. } => Some(("section", id, title)),
            Self::Subsection { id, title, .. } => Some(("subsection", id, title)),
            Self::Module { .. } | Self::Article { .. } => None,
        }
    }
    fn any_article(&self, predicate: &impl Fn(&Node) -> bool) -> bool {
        match self {
            Self::Article { .. } => predicate(self),
            _ => self
                .children()
                .iter()
                .any(|child| child.any_article(predicate)),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Entity {
    Definition {
        id: String,
        text: String,
    },
    Clause {
        id: String,
        actor: String,
        modality: Modality,
        text: String,
        definition_refs: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        annex_refs: Vec<AttachmentRef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        form_refs: Vec<AttachmentRef>,
    },
    Operation {
        id: String,
        name: String,
        #[serde(deserialize_with = "unique_map")]
        inputs: BTreeMap<String, Type>,
        output: Type,
        clause_refs: Vec<String>,
        exclusions: Vec<String>,
    },
    Exception {
        id: String,
        target: String,
        applicability: Applicability,
        effect: Effect,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_nonnull"
        )]
        #[schemars(with = "String")]
        sentence: Option<String>,
    },
    Case(#[schemars(with = "CaseInput")] Case),
    Annex {
        id: String,
        title: String,
        status: AttachmentStatus,
        columns: Vec<String>,
        rows: Vec<Vec<Literal>>,
    },
    Form {
        id: String,
        title: String,
        status: AttachmentStatus,
        fields: Vec<FormField>,
    },
    SentencePart {
        id: String,
        clause: String,
        portion: SentencePortion,
        text: String,
    },
    Expression {
        id: String,
        text: String,
        #[serde(deserialize_with = "unique_map")]
        inputs: BTreeMap<String, Type>,
        output: Type,
        clause_refs: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum CaseTarget {
    Operation(String),
    Expression(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "CaseInput")]
struct Case {
    id: String,
    #[serde(flatten)]
    target: CaseTarget,
    input: BTreeMap<String, Literal>,
    expected: Literal,
    clause_refs: Vec<String>,
    exception_refs: Vec<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline, extend("oneOf" = [{"required": ["operation"]}, {"required": ["expression"]}]))]
struct CaseInput {
    id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    #[schemars(with = "String")]
    operation: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_nonnull"
    )]
    #[schemars(with = "String")]
    expression: Option<String>,
    #[serde(deserialize_with = "unique_map")]
    input: BTreeMap<String, Literal>,
    expected: Literal,
    clause_refs: Vec<String>,
    exception_refs: Vec<String>,
}

impl TryFrom<CaseInput> for Case {
    type Error = &'static str;

    fn try_from(value: CaseInput) -> Result<Self, Self::Error> {
        let target = match (value.operation, value.expression) {
            (Some(id), None) => CaseTarget::Operation(id),
            (None, Some(id)) => CaseTarget::Expression(id),
            _ => return Err("Case requires exactly one of operation or expression"),
        };
        Ok(Self {
            id: value.id,
            target,
            input: value.input,
            expected: value.expected,
            clause_refs: value.clause_refs,
            exception_refs: value.exception_refs,
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Modality {
    Obligation,
    Prohibition,
    Permission,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Type {
    I32,
    Bool,
    String,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(untagged)]
enum Literal {
    Integer(#[schemars(range(min = i32::MIN, max = i32::MAX))] i32),
    Boolean(bool),
    Text(String),
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Applicability {
    InputEquals { field: String, value: Literal },
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Effect {
    ReplaceResult { value: Literal },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Answers {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    expected_base: String,
    answers: Vec<Answer>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
enum Answer {
    ProposeEntity {
        answer_id: String,
        target: Target,
        value: Entity,
        source: String,
    },
    ProposeDecision {
        answer_id: String,
        decision_id: String,
        value: DecisionProposal,
        source: String,
    },
    Skip {
        answer_id: String,
        decision_id: String,
        source: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    kind: String,
    id: String,
}
/// Export the structural JSON Schema for one complete Entity interview answer.
/// This is not semantic validation: references, duplicate raw keys, identity,
/// authority and revision correctness remain runtime checks.
pub fn entity_schema() -> schemars::Schema {
    schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<Entity>()
}

impl Entity {
    fn id(&self) -> &str {
        match self {
            Self::Definition { id, .. }
            | Self::Clause { id, .. }
            | Self::Operation { id, .. }
            | Self::Exception { id, .. }
            | Self::Case(Case { id, .. })
            | Self::Annex { id, .. }
            | Self::Form { id, .. }
            | Self::SentencePart { id, .. }
            | Self::Expression { id, .. } => id,
        }
    }
    fn kind(&self) -> &str {
        match self {
            Self::Definition { .. } => "definition",
            Self::Clause { .. } => "clause",
            Self::Operation { .. } => "operation",
            Self::Exception { .. } => "exception",
            Self::Case(_) => "case",
            Self::Annex { .. } => "annex",
            Self::Form { .. } => "form",
            Self::SentencePart { .. } => "sentence-part",
            Self::Expression { .. } => "expression",
        }
    }
}
impl Candidate {
    fn has_expressions(&self) -> bool {
        self.semantic
            .entities
            .iter()
            .any(|entity| matches!(entity, Entity::Expression { .. }))
    }

    fn uses_v8(&self) -> bool {
        self.semantic.entities.iter().any(|entity| {
            matches!(
                entity,
                Entity::Annex { .. } | Entity::Form { .. } | Entity::SentencePart { .. }
            )
        })
    }
}
fn nonblank(s: &str) -> Result<(), String> {
    if s.trim().is_empty() || s.chars().any(char::is_control) {
        Err("Expected nonblank text without control characters".into())
    } else {
        Ok(())
    }
}
fn id(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 128
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        Err(format!("Invalid stable identifier: {s:?}"))
    } else {
        Ok(())
    }
}
fn unique<'a>(values: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for value in values {
        id(value)?;
        if !ids.insert(value) {
            return Err(format!("Duplicate identifier: {value}"));
        }
    }
    Ok(())
}
fn matches_type(value: &Literal, typ: &Type) -> bool {
    matches!(
        (value, typ),
        (Literal::Integer(_), Type::I32)
            | (Literal::Boolean(_), Type::Bool)
            | (Literal::Text(_), Type::String)
    )
}
fn reference<'a>(
    entities: &'a BTreeMap<&str, &Entity>,
    target: &str,
    kind: &str,
) -> Result<&'a Entity, String> {
    match entities.get(target) {
        Some(e) if e.kind() == kind => Ok(*e),
        _ => Err(format!("Reference {target} must resolve to {kind}")),
    }
}
fn refs(entities: &BTreeMap<&str, &Entity>, targets: &[String], kind: &str) -> Result<(), String> {
    unique(targets.iter().map(String::as_str))?;
    for target in targets {
        reference(entities, target, kind)?;
    }
    Ok(())
}
/// Check this finite schema profile, not the truth or completeness of business prose.
pub fn validate(candidate: &Candidate) -> Result<(), String> {
    if candidate.format != "caac-candidate-v1" {
        return Err("Unsupported candidate format".into());
    }
    if candidate.authority.owner_acceptance != "pending"
        || candidate.authority.implementation_admission != "pending"
    {
        return Err("Candidate authority must remain pending".into());
    }
    id(&candidate.semantic.namespace)?;
    unique(
        candidate
            .semantic
            .entities
            .iter()
            .map(Entity::id)
            .chain(candidate.semantic.unresolved.iter().map(|d| d.id.as_str())),
    )?;
    for decision in &candidate.semantic.unresolved {
        nonblank(&decision.question)?;
    }
    let es: BTreeMap<_, _> = candidate
        .semantic
        .entities
        .iter()
        .map(|e| (e.id(), e))
        .collect();
    if es.is_empty() {
        return Err("Candidate must contain explicit entities".into());
    }
    let mut operation_names = BTreeSet::new();
    let mut expression_count = 0;
    for entity in es.values() {
        match entity {
            Entity::Definition { text, .. } => nonblank(text)?,
            Entity::Clause {
                actor,
                text,
                definition_refs,
                annex_refs,
                form_refs,
                ..
            } => {
                nonblank(actor)?;
                nonblank(text)?;
                refs(&es, definition_refs, "definition")?;
                attachments::validate_references(&es, annex_refs, form_refs)?;
            }
            Entity::Operation {
                name,
                inputs,
                clause_refs,
                exclusions,
                ..
            } => {
                id(name)?;
                if !operation_names.insert(name) {
                    return Err("Duplicate operation name".into());
                }
                for input in inputs.keys() {
                    id(input)?;
                }
                if clause_refs.is_empty() {
                    return Err("Operation requires at least one clause".into());
                }
                refs(&es, clause_refs, "clause")?;
                for exclusion in exclusions {
                    nonblank(exclusion)?;
                }
            }
            Entity::Expression {
                text,
                inputs,
                clause_refs,
                ..
            } => {
                expression_count += 1;
                nonblank(text)?;
                for input in inputs.keys() {
                    id(input)?;
                }
                if clause_refs.is_empty() {
                    return Err("Expression requires at least one clause".into());
                }
                refs(&es, clause_refs, "clause")?;
            }
            Entity::Exception {
                target,
                sentence,
                applicability: Applicability::InputEquals { field, value },
                effect: Effect::ReplaceResult { value: result },
                ..
            } => {
                reference(&es, target, "clause")?;
                if let Some(sentence) = sentence {
                    attachments::validate_exception_sentence(&es, target, sentence)?;
                }
                id(field)?;
                let mut uses = 0;
                for e in es.values() {
                    if let Entity::Operation {
                        inputs,
                        output,
                        clause_refs,
                        ..
                    } = e
                        && clause_refs.contains(target)
                    {
                        uses += 1;
                        if !inputs.get(field).is_some_and(|t| matches_type(value, t))
                            || !matches_type(result, output)
                        {
                            return Err("Exception input/result type must match every operation using its target clause".into());
                        }
                    }
                }
                if uses == 0 {
                    return Err("Exception target must be used by an operation".into());
                }
            }
            Entity::Case(Case {
                target,
                input,
                expected,
                clause_refs,
                exception_refs,
                ..
            }) => {
                let (target_id, kind) = match target {
                    CaseTarget::Operation(id) => (id, "operation"),
                    CaseTarget::Expression(id) => (id, "expression"),
                };
                let (inputs, output, target_refs) = match reference(&es, target_id, kind)? {
                    Entity::Operation {
                        inputs,
                        output,
                        clause_refs,
                        ..
                    }
                    | Entity::Expression {
                        inputs,
                        output,
                        clause_refs,
                        ..
                    } => (inputs, output, clause_refs),
                    _ => unreachable!(),
                };
                if input.len() != inputs.len()
                    || inputs
                        .iter()
                        .any(|(k, t)| !input.get(k).is_some_and(|v| matches_type(v, t)))
                    || !matches_type(expected, output)
                {
                    return Err(format!("Case input/result types must match its {kind}"));
                }
                if clause_refs.is_empty() || clause_refs.iter().any(|r| !target_refs.contains(r)) {
                    return Err(format!("Case clauses must belong to its {kind}"));
                }
                refs(&es, clause_refs, "clause")?;
                if matches!(target, CaseTarget::Expression(_)) && !exception_refs.is_empty() {
                    return Err("Expression cases require empty exception_refs".into());
                }
                refs(&es, exception_refs, "exception")?;
                for r in exception_refs {
                    let Entity::Exception { target, .. } = reference(&es, r, "exception")? else {
                        unreachable!()
                    };
                    if !clause_refs.contains(target) {
                        return Err(
                            "Case exception target must be among its clause references".into()
                        );
                    }
                }
            }
            Entity::Annex {
                title,
                columns,
                rows,
                ..
            } => attachments::validate_annex(title, columns, rows)?,
            Entity::Form { title, fields, .. } => attachments::validate_form(title, fields)?,
            Entity::SentencePart { .. } => {}
        }
    }
    attachments::validate_sentence_parts(&es)?;
    let has_expressions = expression_count > 0;
    if has_expressions {
        candidate.presentation.validate_expression_layout()?;
    }
    let placed = presentation_ids(&candidate.presentation)?;
    if candidate.presentation.size == "XS" && operation_names.len() + expression_count != 1 {
        return Err(if has_expressions {
            "The XS profile requires exactly one declared operation or expression"
        } else {
            "The XS function profile requires exactly one declared operation"
        }
        .into());
    }
    if placed.len() != es.len() || placed.iter().any(|id| !es.contains_key(id.as_str())) {
        return Err("Presentation must place every semantic entity exactly once".into());
    }
    attachments::validate_placements(&candidate.presentation, es.values().copied())?;
    validate_module_decisions(candidate)?;
    for decision in &candidate.semantic.unresolved {
        if let Some(proposal) = &decision.proposal {
            nonblank(&proposal.text)?;
            if proposal.entity_refs.is_empty() {
                return Err("Decision proposal needs entity references".into());
            }
            unique(proposal.entity_refs.iter().map(String::as_str))?;
            if proposal
                .entity_refs
                .iter()
                .any(|id| !es.contains_key(id.as_str()))
            {
                return Err("Decision proposal references missing entity".into());
            }
        }
    }
    Ok(())
}
fn validate_module_decisions(candidate: &Candidate) -> Result<(), String> {
    if candidate.presentation.size != "XL" {
        return Ok(());
    }
    let mut assigned = Vec::new();
    for node in &candidate.presentation.nodes {
        let Node::Module { decisions, .. } = node else {
            return Err("XL presentation requires modules".into());
        };
        assigned.extend(decisions.iter().map(String::as_str));
    }
    unique(assigned.iter().copied())?;
    let unresolved: BTreeSet<_> = candidate
        .semantic
        .unresolved
        .iter()
        .map(|d| d.id.as_str())
        .collect();
    if assigned.len() != unresolved.len() || assigned.iter().any(|id| !unresolved.contains(id)) {
        return Err("XL modules must place every unresolved decision exactly once".into());
    }
    Ok(())
}
fn module_file(file: &str) -> bool {
    let Some(stem) = file.strip_suffix(".md") else {
        return false;
    };
    if file.len() > 255
        || file == "review.md"
        || !stem
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !stem
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return false;
    }
    !matches!(stem, "con" | "prn" | "aux" | "nul")
        && !(stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}
fn presentation_ids(presentation: &Presentation) -> Result<Vec<String>, String> {
    let expected = if presentation.has_subitems() {
        Some(SUBITEM_PROFILE)
    } else if presentation.has_items() {
        Some(ITEM_PROFILE)
    } else {
        presentation.has_paragraphs().then_some(PARAGRAPH_PROFILE)
    };
    if presentation.profile.as_deref() != expected {
        return Err("Presentation requires its exact subitem/item/paragraph profile; flat presentation must omit profile".into());
    }
    let mut placed = Vec::new();
    let mut nodes = BTreeSet::new();
    let mut files = BTreeSet::new();
    fn item(
        item: &Item,
        placed: &mut Vec<String>,
        nodes: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        id(&item.id)?;
        if !nodes.insert(item.id.clone()) {
            return Err("Duplicate item presentation ID".into());
        }
        match (&item.entities, &item.subitems) {
            (Some(refs), None) if !refs.is_empty() => placed.extend(refs.iter().cloned()),
            (None, Some(subitems)) if !subitems.is_empty() => {
                for subitem in subitems {
                    id(&subitem.id)?;
                    if !nodes.insert(subitem.id.clone()) {
                        return Err("Duplicate subitem presentation ID".into());
                    }
                    if subitem.entities.is_empty() {
                        return Err("Subitem requires nonempty entities".into());
                    }
                    placed.extend(subitem.entities.iter().cloned());
                }
            }
            _ => return Err("Item requires exactly one nonempty entities or subitems list".into()),
        }
        Ok(())
    }
    fn article(
        node: &Node,
        placed: &mut Vec<String>,
        nodes: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        let Node::Article {
            id: node_id,
            title,
            entities,
            paragraphs,
        } = node
        else {
            return Err("Chapter children must be articles".into());
        };
        id(node_id)?;
        nonblank(title)?;
        if !nodes.insert(node_id.clone()) {
            return Err("Duplicate node or empty article".into());
        }
        match (entities, paragraphs) {
            (Some(refs), None) if !refs.is_empty() => placed.extend(refs.iter().cloned()),
            (None, Some(groups)) if !groups.is_empty() => {
                for group in groups {
                    id(&group.id)?;
                    if !nodes.insert(group.id.clone()) {
                        return Err("Duplicate paragraph presentation ID".into());
                    }
                    match (&group.entities, &group.items) {
                        (Some(refs), None) if !refs.is_empty() => {
                            placed.extend(refs.iter().cloned())
                        }
                        (None, Some(items)) if !items.is_empty() => {
                            for group_item in items {
                                item(group_item, placed, nodes)?;
                            }
                        }
                        _ => {
                            return Err(
                                "Paragraph requires exactly one nonempty entities or items list"
                                    .into(),
                            );
                        }
                    }
                }
            }
            _ => {
                return Err(
                    "Article requires exactly one nonempty entities or paragraphs list".into(),
                );
            }
        }
        Ok(())
    }
    fn choice<'a>(
        articles: &'a Option<Vec<Node>>,
        children: &'a Option<Vec<Node>>,
        kind: &str,
    ) -> Result<(&'a [Node], bool), String> {
        match (articles, children) {
            (Some(nodes), None) if !nodes.is_empty() => Ok((nodes, true)),
            (None, Some(nodes)) if !nodes.is_empty() => Ok((nodes, false)),
            _ => Err(format!(
                "{kind} requires exactly one nonempty articles or children list"
            )),
        }
    }
    fn hierarchy_node(
        node: &Node,
        parent_rank: usize,
        placed: &mut Vec<String>,
        nodes: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if matches!(node, Node::Article { .. }) {
            return article(node, placed, nodes);
        }
        let Some((kind, node_id, title)) = node.upper() else {
            return Err("Modules may only appear at the top level of XL".into());
        };
        let rank = hierarchy::rank(kind);
        if rank <= parent_rank {
            return Err("Upper levels must follow part, chapter, section, subsection order".into());
        }
        id(node_id)?;
        nonblank(title)?;
        if !nodes.insert(node_id.into()) {
            return Err("Duplicate presentation ID".into());
        }
        let (children, articles_only) = match node {
            Node::Chapter {
                articles, children, ..
            } => choice(articles, children, "Chapter")?,
            _ if !node.children().is_empty() => (node.children(), false),
            _ => return Err(format!("{kind} requires nonempty children")),
        };
        for child in children {
            if articles_only {
                article(child, placed, nodes)?;
            } else {
                hierarchy_node(child, rank, placed, nodes)?;
            }
        }
        Ok(())
    }
    if presentation.nodes.is_empty() {
        return Err("Presentation requires nodes".into());
    }
    if presentation.size == "XL" && !presentation.attachments.is_empty() {
        return Err("XL attachment placements belong to modules".into());
    }
    if presentation.size == "XL" && !(2..=64).contains(&presentation.nodes.len()) {
        return Err("XL presentation requires 2 to 64 modules".into());
    }
    if presentation.size == "XS" && !presentation.is_single_flat_article() {
        return Err("The XS function profile requires one flat article".into());
    }
    for node in &presentation.nodes {
        match (presentation.size.as_str(), node) {
            ("XS" | "S", Node::Article { .. }) => article(node, &mut placed, &mut nodes)?,
            ("M", Node::Part { .. }) => return Err("M presentation does not allow parts".into()),
            ("M" | "L", _) => hierarchy_node(node, 0, &mut placed, &mut nodes)?,
            (
                "XL",
                Node::Module {
                    id: node_id,
                    title,
                    file,
                    articles,
                    children,
                    ..
                },
            ) => {
                id(node_id)?;
                nonblank(title)?;
                if !nodes.insert(node_id.clone()) {
                    return Err("Duplicate presentation ID".into());
                }
                if !module_file(file) || !files.insert(file) {
                    return Err(
                        "Module files must be distinct safe lowercase Markdown basenames".into(),
                    );
                }
                let (children, articles_only) = choice(articles, children, "Module")?;
                for child in children {
                    if articles_only {
                        article(child, &mut placed, &mut nodes)?;
                    } else {
                        hierarchy_node(child, 0, &mut placed, &mut nodes)?;
                    }
                }
            }
            _ => {
                return Err(
                    "Expected an XS function article, S articles, M or L hierarchy, or XL modules"
                        .into(),
                );
            }
        }
    }
    placed.extend(presentation.attachment_ids().map(str::to_owned));
    unique(placed.iter().map(String::as_str))?;
    Ok(placed)
}

pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value).map_err(|e| e.to_string())
}
/// All normative content is projected from the same validated candidate, never interpreted.
pub fn render(candidate: &Candidate) -> Result<String, String> {
    render::render(candidate)
}

/// Shared pure answer evaluator used by apply and explicit review handoffs.
pub(crate) fn evaluate_answers(
    original: &[u8],
    answer_bytes: &[u8],
) -> Result<(Candidate, Vec<serde_json::Value>), String> {
    let request: Answers = serde_json::from_slice(answer_bytes).map_err(|e| e.to_string())?;
    if request
        .format
        .as_deref()
        .is_some_and(|format| format != "caac-interview-answers-v1")
    {
        return Err("Unsupported answer format".into());
    }
    if request.expected_base != digest(original) {
        return Err("Stale expected base; no output published".into());
    }
    let mut candidate: Candidate = serde_json::from_slice(original).map_err(|e| e.to_string())?;
    // Initial drafts may have unfilled entity references; final validation is mandatory.
    let mut answer_ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut records = Vec::new();
    for answer in request.answers {
        let (answer_id, source) = match &answer {
            Answer::ProposeEntity {
                answer_id, source, ..
            }
            | Answer::Skip {
                answer_id, source, ..
            }
            | Answer::ProposeDecision {
                answer_id, source, ..
            } => (answer_id, source),
        };
        id(answer_id)?;
        nonblank(source)?;
        if !answer_ids.insert(answer_id.clone()) {
            return Err("Duplicate answer ID".into());
        }
        match answer {
            Answer::ProposeEntity {
                answer_id,
                target,
                value,
                source,
            } => {
                if target.kind != value.kind() || target.id != value.id() {
                    return Err("Answer target does not match entity".into());
                }
                if !targets.insert(target.id.clone())
                    || candidate
                        .semantic
                        .entities
                        .iter()
                        .any(|e| e.id() == target.id)
                {
                    return Err(
                        "Answer would duplicate or replace an entity; replacement is unsupported"
                            .into(),
                    );
                }
                let fields = serde_json::to_value(&value).map_err(|e| e.to_string())?;
                for (field, value) in fields.as_object().ok_or("Entity must be an object")? {
                    records.push(serde_json::json!({"answer_id":answer_id,"entity":target.id,"field":field,"value_sha256":digest(&serde_json::to_vec(value).map_err(|e|e.to_string())?),"source":source,"authority":"unverified-proposal"}));
                }
                candidate.semantic.entities.push(value);
            }
            Answer::ProposeDecision {
                answer_id,
                decision_id,
                value,
                source,
            } => {
                id(&decision_id)?;
                if !targets.insert(decision_id.clone()) {
                    return Err("Decision may be answered once".into());
                }
                let decision = candidate
                    .semantic
                    .unresolved
                    .iter_mut()
                    .find(|d| d.id == decision_id)
                    .ok_or("Unknown decision")?;
                if decision.proposal.is_some() {
                    return Err("Replacing a decision proposal is unsupported".into());
                }
                records.push(serde_json::json!({"answer_id":answer_id,"decision_id":decision_id,"field":"proposal","value_sha256":digest(&json(&value)?),"source":source,"authority":"unverified-proposal"}));
                decision.proposal = Some(value);
            }
            Answer::Skip {
                answer_id,
                decision_id,
                source,
            } => {
                id(&decision_id)?;
                if !targets.insert(decision_id.clone())
                    || !candidate
                        .semantic
                        .unresolved
                        .iter()
                        .any(|d| d.id == decision_id)
                {
                    return Err("Skipped decision must exist and may be answered once".into());
                }
                records.push(serde_json::json!({"answer_id":answer_id,"decision_id":decision_id,"result":"unresolved","source":source,"authority":"unverified-proposal"}));
            }
        }
    }
    validate(&candidate)?;
    Ok((candidate, records))
}

/// Inputs are immutable; a fresh output directory is never overwritten.
/// The final provenance file marks publication completion. I/O failure may leave partial output.
pub fn apply(base_path: &Path, answers_path: &Path, destination: &Path) -> Result<(), String> {
    apply_bound(base_path, answers_path, destination, None)
}
fn apply_bound(
    base_path: &Path,
    answers_path: &Path,
    destination: &Path,
    expected: Option<(&[u8], &[u8])>,
) -> Result<(), String> {
    apply_guarded(base_path, answers_path, destination, expected, None)
}
pub(crate) fn apply_guarded(
    base_path: &Path,
    answers_path: &Path,
    destination: &Path,
    expected: Option<(&[u8], &[u8])>,
    guard: Option<&dyn Fn() -> Result<(), String>>,
) -> Result<(), String> {
    const MAX_INPUT: usize = 1024 * 1024;
    let original = crate::source_intake::read_regular(base_path, MAX_INPUT)?;
    let answer_bytes = crate::source_intake::read_regular(answers_path, MAX_INPUT)?;
    if expected.is_some_and(|(base, answers)| base != original || answers != answer_bytes) {
        return Err("Bound input snapshot changed before apply; not published".into());
    }
    let (candidate, records) = evaluate_answers(&original, &answer_bytes)?;
    let bytes = json(&candidate)?;
    let view = render(&candidate)?;
    let modules = module_artifacts::ModuleArtifacts::render(&candidate, view.len())?;
    let revision_manifest = revisions(&candidate)?;
    let manifest = serde_json::json!({"format":"caac-candidate-provenance-v3","renderer_profile":render::profile(&candidate),"base_sha256":digest(&original),"answers_sha256":digest(&answer_bytes),"candidate_sha256":digest(&bytes),"semantic_sha256":digest(&json(&candidate.semantic)?),"review_sha256":digest(view.as_bytes()),"revision_manifest":revision_manifest,"field_changes":records,"required_unresolved":candidate.semantic.unresolved.iter().filter(|d|d.required).count(),"contract_complete":false,"owner_acceptance":"pending","implementation_admission":"pending","behavior_evidence":"not-run","exception_review":"pending"});
    let manifest: Provenance = serde_json::from_value(manifest).map_err(|e| e.to_string())?;
    let manifest_bytes = json(&manifest)?;
    if bytes.len() > MAX_INPUT || view.len() > MAX_ARTIFACT || manifest_bytes.len() > MAX_ARTIFACT {
        return Err("Generated candidate or review artifacts exceed supported size limits".into());
    }
    // Recheck before output; this is conflict detection, not a lock against concurrent writers.
    if crate::source_intake::read_regular(base_path, MAX_INPUT)? != original
        || crate::source_intake::read_regular(answers_path, MAX_INPUT)? != answer_bytes
    {
        return Err("Input changed during authoring; no output published".into());
    }
    if let Some(guard) = guard {
        guard()?;
        if crate::source_intake::read_regular(base_path, MAX_INPUT)? != original
            || crate::source_intake::read_regular(answers_path, MAX_INPUT)? != answer_bytes
        {
            return Err("Input changed after evidence revalidation; no output published".into());
        }
    }
    crate::output::require_absent(destination)?;
    fs::create_dir(destination).map_err(|e| e.to_string())?;
    crate::output::write_new(&destination.join("candidate.json"), &bytes)?;
    crate::output::write_new(&destination.join("review.md"), view.as_bytes())?;
    modules.publish(destination)?;
    crate::output::write_new(&destination.join("provenance.json"), &manifest_bytes)?;
    Ok(())
}

fn unique_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an object with unique keys")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut result = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if result.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!("duplicate key: {key}")));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Visitor(std::marker::PhantomData))
}

const INTERVIEW_FORMAT: &str = "caac-interview-answers-v1";
const MAX_INPUT: usize = 1024 * 1024;
const MAX_LINE: usize = 64 * 1024;
#[derive(Serialize)]
#[serde(tag = "question")]
enum Question {
    Entity { id: String },
    Decision { decision: Decision },
}
fn interview_plan(candidate: &Candidate) -> Result<Vec<Question>, String> {
    if candidate.format != "caac-candidate-v1"
        || candidate.authority.owner_acceptance != "pending"
        || candidate.authority.implementation_admission != "pending"
    {
        return Err("Unsupported draft format or non-pending authority".into());
    }
    id(&candidate.semantic.namespace)?;
    unique(
        candidate
            .semantic
            .entities
            .iter()
            .map(Entity::id)
            .chain(candidate.semantic.unresolved.iter().map(|d| d.id.as_str())),
    )?;
    if candidate.has_expressions() {
        candidate.presentation.validate_expression_layout()?;
    }
    let placed = presentation_ids(&candidate.presentation)?;
    validate_module_decisions(candidate)?;
    attachments::validate_placements(&candidate.presentation, candidate.semantic.entities.iter())?;
    if candidate
        .semantic
        .entities
        .iter()
        .any(|e| !placed.iter().any(|id| id == e.id()))
    {
        return Err("Existing entity is not placed in the draft".into());
    }
    let mut plan: Vec<_> = placed
        .into_iter()
        .filter(|id| !candidate.semantic.entities.iter().any(|e| e.id() == id))
        .map(|id| Question::Entity { id })
        .collect();
    for decision in &candidate.semantic.unresolved {
        nonblank(&decision.question)?;
        if decision.proposal.is_none() {
            plan.push(Question::Decision {
                decision: decision.clone(),
            });
        }
    }
    Ok(plan)
}
fn read_answer(reader: &mut impl BufRead) -> Result<String, String> {
    let mut bytes = Vec::new();
    let read = reader
        .take(MAX_LINE as u64 + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|e| e.to_string())?;
    if read == 0 {
        return Err("Interview interrupted by EOF; collected answers retained".into());
    }
    if bytes.len() > MAX_LINE {
        return Err("Interview answer exceeds 64 KiB; collected answers retained".into());
    }
    let line = String::from_utf8(bytes).map_err(|_| "Interview answers must use UTF-8")?;
    let line = line.trim_end_matches(['\r', '\n']).to_owned();
    if line == "cancel" {
        return Err("Interview cancelled; collected answers retained".into());
    }
    Ok(line)
}
fn check_prefix(request: &Answers, plan: &[Question]) -> Result<(), String> {
    if request.answers.len() > plan.len() {
        return Err("Journal has more answers than the frozen plan".into());
    }
    for (index, (answer, question)) in request.answers.iter().zip(plan).enumerate() {
        let (answer_id, source) = match answer {
            Answer::ProposeEntity {
                answer_id, source, ..
            }
            | Answer::ProposeDecision {
                answer_id, source, ..
            }
            | Answer::Skip {
                answer_id, source, ..
            } => (answer_id, source),
        };
        if answer_id != &format!("interview-{}", index + 1) {
            return Err("Journal answer order differs from frozen plan".into());
        }
        nonblank(source)?;
        match (answer, question) {
            (Answer::ProposeEntity { target, value, .. }, Question::Entity { id })
                if &target.id == id && value.id() == id && target.kind == value.kind() => {}
            (
                Answer::ProposeDecision { decision_id, .. } | Answer::Skip { decision_id, .. },
                Question::Decision { decision },
            ) if decision_id == &decision.id => {}
            _ => return Err("Journal answer does not match the frozen question".into()),
        }
    }
    Ok(())
}
fn ensure_base(path: &Path, original: &[u8]) -> Result<(), String> {
    if crate::source_intake::read_regular(path, MAX_INPUT)? != original {
        return Err(
            "Draft changed during interview; collected answers retained for the old base".into(),
        );
    }
    Ok(())
}
fn ensure_journal(path: &Path, saved: &[u8]) -> Result<(), String> {
    if crate::source_intake::read_regular(path, MAX_INPUT)? != saved {
        return Err("Journal changed outside this interview; not applied".into());
    }
    Ok(())
}
/// Bounded typed stdin adapter. The journal is untrusted proposal data, never acceptance.
pub fn interview(
    base_path: &Path,
    journal_path: &Path,
    destination: &Path,
    resume: bool,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<(), String> {
    let base_key = crate::output::destination_key(base_path)?;
    let journal_key = crate::output::destination_key(journal_path)?;
    let output_key = crate::output::destination_key(destination)?;
    if base_key.starts_with(&journal_key)
        || journal_key.starts_with(&base_key)
        || base_key.starts_with(&output_key)
        || output_key.starts_with(&base_key)
        || journal_key.starts_with(&output_key)
        || output_key.starts_with(&journal_key)
    {
        return Err("Base, journal and output paths must not overlap".into());
    }
    crate::output::require_absent(destination)?;
    let original = crate::source_intake::read_regular(base_path, MAX_INPUT)?;
    let candidate: Candidate = serde_json::from_slice(&original).map_err(|e| e.to_string())?;
    let plan = interview_plan(&candidate)?;
    let (mut request, mut saved) = if resume {
        let bytes = crate::source_intake::read_regular(journal_path, MAX_INPUT)?;
        let request: Answers = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if request.format.as_deref() != Some(INTERVIEW_FORMAT)
            || request.expected_base != digest(&original)
        {
            return Err("Journal format or exact base does not match; not resumed".into());
        }
        check_prefix(&request, &plan)?;
        writeln!(
            writer,
            "Saved answers are untrusted proposals, not authenticated or accepted: {}",
            crate::suggest::safe_json(&request)
        )
        .map_err(|e| e.to_string())?;
        writeln!(
            writer,
            "Type reuse to reuse these answers, or cancel. This is not owner acceptance."
        )
        .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        if read_answer(reader)? != "reuse" {
            return Err("Saved answers were not explicitly selected for reuse".into());
        }
        (request, bytes)
    } else {
        crate::output::require_absent(journal_path)?;
        let request = Answers {
            format: Some(INTERVIEW_FORMAT.into()),
            expected_base: digest(&original),
            answers: Vec::new(),
        };
        let bytes = json(&request)?;
        ensure_base(base_path, &original)?;
        crate::output::write_new(journal_path, &bytes)?;
        (request, bytes)
    };
    ensure_journal(journal_path, &saved)?;
    for (index, question) in plan.iter().enumerate().skip(request.answers.len()) {
        ensure_base(base_path, &original)?;
        ensure_journal(journal_path, &saved)?;
        writeln!(
            writer,
            "Question {}: {}",
            index + 1,
            crate::suggest::safe_json(question)
        )
        .map_err(|e| e.to_string())?;
        writeln!(writer,"Entity structural input schema: caac candidate schema. Runtime validation still applies.\nEnter one complete Entity JSON line, or for a decision {{\"text\":\"proposal\",\"entity_refs\":[\"id\"]}}. Use skip to leave incomplete, or cancel.").map_err(|e|e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        let line = read_answer(reader)?;
        let answer_id = format!("interview-{}", index + 1);
        let source = "interactive typed proposal; unverified".to_owned();
        let answer = match question {
            Question::Entity { id } => {
                if line == "skip" {
                    return Err("Entity remains missing; collected answers retained; resume to answer this ID".into());
                }
                let value: Entity = serde_json::from_str(&line).map_err(|e| e.to_string())?;
                if value.id() != id {
                    return Err("Entity answer does not match the prompted ID".into());
                }
                Answer::ProposeEntity {
                    answer_id,
                    target: Target {
                        kind: value.kind().into(),
                        id: id.clone(),
                    },
                    value,
                    source,
                }
            }
            Question::Decision { decision } => {
                if line == "skip" {
                    Answer::Skip {
                        answer_id,
                        decision_id: decision.id.clone(),
                        source,
                    }
                } else {
                    Answer::ProposeDecision {
                        answer_id,
                        decision_id: decision.id.clone(),
                        value: serde_json::from_str(&line).map_err(|e| e.to_string())?,
                        source,
                    }
                }
            }
        };
        request.answers.push(answer);
        let bytes = json(&request)?;
        if bytes.len() > MAX_INPUT {
            return Err("Journal exceeds 1 MiB; previous collected answers retained".into());
        }
        ensure_base(base_path, &original)?;
        crate::output::replace_owned(journal_path, &saved, &bytes)?;
        saved = bytes;
    }
    ensure_base(base_path, &original)?;
    ensure_journal(journal_path, &saved)?;
    // The same apply path performs all final references, types and authority checks.
    apply_bound(
        base_path,
        journal_path,
        destination,
        Some((&original, &saved)),
    )
}

const MAX_ARTIFACT: usize = 32 * 1024 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    format: String,
    renderer_profile: String,
    base_sha256: String,
    answers_sha256: String,
    candidate_sha256: String,
    semantic_sha256: String,
    review_sha256: String,
    revision_manifest: RevisionManifest,
    field_changes: Vec<FieldChange>,
    required_unresolved: usize,
    contract_complete: bool,
    owner_acceptance: String,
    implementation_admission: String,
    behavior_evidence: String,
    exception_review: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum FieldChange {
    Entity {
        answer_id: String,
        entity: String,
        field: String,
        value_sha256: String,
        source: String,
        authority: String,
    },
    Decision {
        answer_id: String,
        decision_id: String,
        field: String,
        value_sha256: String,
        source: String,
        authority: String,
    },
    Skip {
        answer_id: String,
        decision_id: String,
        result: String,
        source: String,
        authority: String,
    },
}
/// Verify unsigned artifact consistency, not authorship, inputs, or owner acceptance.
pub fn verify(
    candidate_path: &Path,
    provenance_path: &Path,
    review_path: &Path,
) -> Result<(), String> {
    let candidate_bytes = crate::source_intake::read_regular(candidate_path, MAX_INPUT)?;
    let provenance_bytes = crate::source_intake::read_regular(provenance_path, MAX_ARTIFACT)?;
    let review_bytes = crate::source_intake::read_regular(review_path, MAX_ARTIFACT)?;
    verify_bytes(&candidate_bytes, &provenance_bytes, &review_bytes)?;
    let candidate: Candidate =
        serde_json::from_slice(&candidate_bytes).map_err(|e| e.to_string())?;
    let modules = module_artifacts::ModuleArtifacts::render(&candidate, review_bytes.len())?;
    modules.verify(review_path)?;
    if crate::source_intake::read_regular(candidate_path, MAX_INPUT)? != candidate_bytes
        || crate::source_intake::read_regular(provenance_path, MAX_ARTIFACT)? != provenance_bytes
        || crate::source_intake::read_regular(review_path, MAX_ARTIFACT)? != review_bytes
    {
        return Err("Input changed during artifact verification".into());
    }
    modules.verify(review_path)?;
    Ok(())
}

fn verify_bytes(
    candidate_bytes: &[u8],
    provenance_bytes: &[u8],
    review_bytes: &[u8],
) -> Result<(), String> {
    let envelope =
        compare::strict(std::str::from_utf8(provenance_bytes).map_err(|e| e.to_string())?)?;
    if envelope["format"] == "caac-candidate-provenance-v5" {
        return presentation::verify_v5(candidate_bytes, provenance_bytes, review_bytes);
    }
    if envelope["format"] == "caac-candidate-provenance-v4" {
        return revise::verify_v4(candidate_bytes, provenance_bytes, review_bytes);
    }
    let candidate: Candidate =
        serde_json::from_slice(candidate_bytes).map_err(|e| e.to_string())?;
    let provenance: Provenance =
        serde_json::from_slice(provenance_bytes).map_err(|e| e.to_string())?;
    if provenance.format != "caac-candidate-provenance-v3" {
        return Err("Unsupported provenance version; supported v3, v4 or v5 is required".into());
    }
    if provenance.renderer_profile != render::profile(&candidate) {
        return Err("Unsupported renderer profile".into());
    }
    verify_revisions(&candidate, &provenance.revision_manifest)?;
    if provenance.contract_complete
        || provenance.owner_acceptance != "pending"
        || provenance.implementation_admission != "pending"
        || provenance.behavior_evidence != "not-run"
        || provenance.exception_review != "pending"
        || provenance.required_unresolved
            != candidate
                .semantic
                .unresolved
                .iter()
                .filter(|d| d.required)
                .count()
    {
        return Err("Unsupported completion, evidence or authority claim".into());
    }
    for change in &provenance.field_changes {
        let authority = match change {
            FieldChange::Entity { authority, .. }
            | FieldChange::Decision { authority, .. }
            | FieldChange::Skip { authority, .. } => authority,
        };
        if authority != "unverified-proposal" {
            return Err("Field provenance cannot claim approval".into());
        }
    }
    if provenance.candidate_sha256 != digest(candidate_bytes)
        || provenance.review_sha256 != digest(review_bytes)
        || provenance.semantic_sha256 != digest(&json(&candidate.semantic)?)
    {
        return Err("Artifact checksum mismatch".into());
    }
    if render(&candidate)?.as_bytes() != review_bytes {
        return Err("Review is not the exact projection of the supplied candidate".into());
    }
    Ok(())
}

#[cfg(test)]
mod evidence_publication_guard_tests {
    use super::*;
    #[test]
    fn final_guard_cannot_switch_the_evaluated_answers_before_publication() {
        let directory = tempfile::tempdir().unwrap();
        let base = directory.path().join("base");
        let answers = directory.path().join("answers");
        let output = directory.path().join("out");
        let original = include_bytes!("../examples/candidate/draft.json");
        let answer_bytes = include_bytes!("../examples/candidate/answers.json");
        fs::write(&base, original).unwrap();
        fs::write(&answers, answer_bytes).unwrap();
        let guard = || {
            let mut changed = answer_bytes.to_vec();
            changed.push(b' ');
            fs::write(&answers, changed).map_err(|e| e.to_string())
        };
        let error = apply_guarded(
            &base,
            &answers,
            &output,
            Some((original, answer_bytes)),
            Some(&guard),
        )
        .unwrap_err();
        assert!(error.contains("after evidence revalidation"));
        assert!(!output.exists());
    }
}
