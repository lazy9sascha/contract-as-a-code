//! Evidence-linked questions only. Syntax never establishes a contract answer.
use crate::source_intake::EvidenceContext;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Input,
    Output,
    Actor,
    Failure,
    Scope,
}

#[derive(Serialize)]
pub struct PendingReviewQuestion<'a> {
    pub kind: &'static str,
    pub authority: &'static str,
    pub status: &'static str,
    pub decision: &'a str,
    pub category: Category,
    pub question: &'static str,
    pub uncertainty: &'static str,
    pub evidence_context_sha256: &'a str,
    pub evidence_selection: &'static str,
    pub unsupported: [&'static str; 2],
}

pub fn suggest_for_decision<'a>(
    decision: &'a str,
    context_digest: &'a str,
) -> PendingReviewQuestion<'a> {
    let lower = decision.to_lowercase();
    let (category, question) = if lower.contains("input domain") || lower.contains("input type") {
        (
            Category::Input,
            "Which input domain is required for this selected operation? Confirm constraints independently of source parameter syntax.",
        )
    } else if lower.contains("output") || lower.contains("result") {
        (
            Category::Output,
            "Which observable result is required? Source return syntax does not establish a promised outcome.",
        )
    } else if lower.contains("actor") || lower.contains("responsible") {
        (
            Category::Actor,
            "Who is responsible for this obligation? A source symbol does not establish a domain actor.",
        )
    } else if lower.contains("error") || lower.contains("failure") {
        (
            Category::Failure,
            "Which failure conditions and responses are required? Explicit raise or assert syntax does not establish runtime failure behavior.",
        )
    } else {
        (
            Category::Scope,
            "Which selected operation and requirement does this decision concern? Identify the relevant observations and resolve any conflicting evidence.",
        )
    };
    PendingReviewQuestion {
        kind: "pending-review-question",
        authority: "proposal",
        status: "pending-review",
        decision,
        category,
        question,
        uncertainty: "Empty, unsupported, ambiguous, or conflicting syntax cannot determine an answer. Domain meaning, contradictions, dependencies, and runtime behavior remain unverified; source approval claims have no authority.",
        evidence_context_sha256: context_digest,
        evidence_selection: "All files and observations in the referenced context are candidates, not a resolved mapping to this decision.",
        unsupported: ["proposed definitions", "exception relationships"],
    }
}

/// JSON's Unicode output is additionally ASCII-escaped for terminal safety.
pub fn safe_json(value: &impl Serialize) -> String {
    let json = serde_json::to_string(value).expect("serializable suggestion data");
    let mut out = String::new();
    for ch in json.chars() {
        if ch.is_ascii() && !ch.is_control() {
            out.push(ch);
        } else {
            for unit in ch.encode_utf16(&mut [0; 2]) {
                out.push_str(&format!("\\u{unit:04x}"));
            }
        }
    }
    out
}

/// Bound the complete serialized batch before printing any evidence or question.
pub fn emit_batch<'a>(
    context: &EvidenceContext,
    decisions: impl Iterator<Item = &'a str>,
) -> Result<(), String> {
    const MAX_DISPLAY: usize = 32 * 1024 * 1024;
    let text = safe_json(context);
    if text.len() > MAX_DISPLAY {
        return Err(
            "Evidence display exceeds the 32 MiB batch limit; no context was emitted".into(),
        );
    }
    let digest: String = Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let mut questions = Vec::new();
    let mut size = text.len() + 100;
    for decision in decisions {
        if decision.len() > 16 * 1024 {
            return Err(
                "Decision exceeds the 16 KiB question limit; no context was emitted".into(),
            );
        }
        let question = safe_json(&suggest_for_decision(decision, &digest));
        size += question.len() + 24;
        if size > MAX_DISPLAY {
            return Err(
                "Suggestion display exceeds the 32 MiB batch limit; no context was emitted".into(),
            );
        }
        questions.push(question);
    }
    println!("Evidence context {digest}: {text}");
    for question in questions {
        println!("Suggested question: {question}");
    }
    Ok(())
}
