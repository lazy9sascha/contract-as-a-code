//! Authored claims with checked citations, not certified semantic conclusions.
use crate::{
    candidate,
    contract_collection::{self, Request},
    output, review_packet, source_intake,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
const INPUT: usize = 1024 * 1024;
const ARTIFACT: usize = 32 * INPUT;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Findings {
    format: String,
    expected_collection_sha256: String,
    expected_packet_sha256: String,
    expected_base_sha256: String,
    findings: Vec<Finding>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Finding {
    id: String,
    claim_kind: ClaimKind,
    claim: String,
    inference_limits: String,
    citations: Vec<Citation>,
    handoff: Handoff,
}
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ClaimKind {
    Aligned,
    SuspectedMismatch,
    Unverified,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Citation {
    evidence_id: String,
    start_byte: usize,
    end_byte: usize,
    excerpt: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Handoff {
    decision_id: String,
    proposal_text: String,
    entity_refs: Vec<String>,
}
fn text(s: &str) -> Result<(), String> {
    if s.trim().is_empty() || s.len() > 16384 || s.chars().any(char::is_control) {
        Err("Claim text must be nonblank, bounded and without control characters".into())
    } else {
        Ok(())
    }
}
fn id(s: &str) -> Result<(), String> {
    if s.is_empty()
        || s.len() > 128
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        Err("Invalid finding ID".into())
    } else {
        Ok(())
    }
}
/// Validate authored claims/citations and publish answers plus a final receipt.
/// The candidate is not changed; apply remains a separate explicit invocation.
pub fn handoff(
    collection_path: &Path,
    packet_path: &Path,
    findings_path: &Path,
    base_path: &Path,
    request: &Request,
    destination: &Path,
) -> Result<(), String> {
    let prepared = prepare(
        collection_path,
        packet_path,
        findings_path,
        base_path,
        request,
        destination,
    )?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::create_dir(destination).map_err(|e| e.to_string())?;
    output::write_new(&destination.join("answers.json"), &prepared.answer_bytes)?;
    output::write_new(&destination.join("receipt.json"), &prepared.receipt_bytes)?;
    Ok(())
}
struct Prepared {
    base_bytes: Vec<u8>,
    answer_bytes: Vec<u8>,
    receipt_bytes: Vec<u8>,
}
fn prepare(
    collection_path: &Path,
    packet_path: &Path,
    findings_path: &Path,
    base_path: &Path,
    request: &Request,
    destination: &Path,
) -> Result<Prepared, String> {
    output::require_absent(destination)?;
    let paths = [collection_path, packet_path, findings_path, base_path];
    let limits = [ARTIFACT, ARTIFACT, INPUT, INPUT];
    let mut snapshots = Vec::new();
    let key = output::destination_key(destination)?;
    for (path, limit) in paths.iter().zip(limits) {
        let bytes = source_intake::read_regular(path, limit)?;
        let resolved = fs::canonicalize(path).map_err(|e| e.to_string())?;
        if key == resolved || key.starts_with(&resolved) || resolved.starts_with(&key) {
            return Err("Output overlaps a handoff input".into());
        }
        snapshots.push((resolved, bytes));
    }
    let f: Findings = serde_json::from_slice(&snapshots[2].1).map_err(|e| e.to_string())?;
    if f.format != "caac-review-findings-v1" || f.findings.is_empty() || f.findings.len() > 128 {
        return Err("Unsupported findings profile or count".into());
    }
    if f.expected_collection_sha256 != candidate::digest(&snapshots[0].1)
        || f.expected_packet_sha256 != candidate::digest(&snapshots[1].1)
        || f.expected_base_sha256 != candidate::digest(&snapshots[3].1)
    {
        return Err("Stale exact findings input hashes".into());
    }
    let (packet, packet_bytes) = review_packet::prepare(collection_path, request, destination)?;
    if packet_bytes != snapshots[1].1
        || packet.collection_sha256 != candidate::digest(&snapshots[0].1)
    {
        return Err("Packet is not the exact current collection index".into());
    }
    let (collection, collection_bytes) = contract_collection::prepare(request, destination)?;
    if collection_bytes != snapshots[0].1 {
        return Err("Collection changed during handoff".into());
    }
    let base: candidate::Candidate =
        serde_json::from_slice(&snapshots[3].1).map_err(|e| e.to_string())?;
    candidate::validate(&base)?;
    let mut contents = BTreeMap::new();
    for (i, d) in collection.documents.iter().enumerate() {
        contents.insert(format!("/documents/{i}"), d.text.as_bytes().to_vec());
    }
    if let Some(b) = &collection.baseline {
        for (i, file) in b.evidence.files.iter().enumerate() {
            let bytes = source_intake::read_regular(Path::new(&file.resolved_path), INPUT)?;
            if candidate::digest(&bytes) != file.sha256 || bytes.len() != file.byte_length {
                return Err("Cited source snapshot changed".into());
            }
            contents.insert(format!("/baseline/evidence/files/{i}"), bytes);
        }
    }
    if let Some(analyzer) = &collection.analyzer_evidence {
        for (i, artifact) in analyzer.artifacts.iter().enumerate() {
            contents.insert(
                format!("/analyzer_evidence/artifacts/{i}"),
                artifact.text.as_bytes().to_vec(),
            );
        }
    }
    let evidence: BTreeMap<_, _> = packet.evidence.iter().map(|e| (e.id.as_str(), e)).collect();
    let mut ids = BTreeSet::new();
    let mut answers = Vec::new();
    for finding in &f.findings {
        id(&finding.id)?;
        text(&finding.claim)?;
        text(&finding.inference_limits)?;
        text(&finding.handoff.proposal_text)?;
        if !ids.insert(&finding.id) || finding.citations.is_empty() || finding.citations.len() > 16
        {
            return Err("Duplicate finding or invalid citation count".into());
        }
        let mut doc = false;
        let mut source = false;
        let mut citation_keys = BTreeSet::new();
        for citation in &finding.citations {
            let e = evidence
                .get(citation.evidence_id.as_str())
                .ok_or("Unknown exact evidence ID")?;
            if !citation_keys.insert((
                &citation.evidence_id,
                citation.start_byte,
                citation.end_byte,
            )) {
                return Err("Duplicate citation".into());
            }
            if citation.excerpt.is_empty()
                || citation.excerpt.len() > 16384
                || citation.start_byte >= citation.end_byte
                || citation.start_byte < e.start_byte
                || citation.end_byte > e.end_byte
            {
                return Err("Citation exceeds evidence range or excerpt budget".into());
            }
            let pointer = if e.kind == "source-observation" {
                e.collection_pointer.split("/analysis/").next().unwrap()
            } else {
                &e.collection_pointer
            };
            let bytes = contents
                .get(pointer)
                .ok_or("Evidence content unavailable")?;
            if candidate::digest(bytes) != e.file_sha256 {
                return Err("Evidence file hash mismatch".into());
            }
            let full = std::str::from_utf8(bytes).map_err(|_| "Cited file must be UTF-8")?;
            let quote = full
                .get(citation.start_byte..citation.end_byte)
                .ok_or("Citation is not a valid UTF-8 byte span")?;
            if quote != citation.excerpt {
                return Err("Citation excerpt differs from exact source text".into());
            }
            if e.kind == "document" {
                doc = true;
            } else if matches!(e.kind, "source-file" | "source-observation") {
                source = true;
            }
        }
        if finding.claim_kind != ClaimKind::Unverified && !(doc && source) {
            return Err("Aligned/mismatch claims require both document and source citations; neither proves truth".into());
        }
        answers.push(json!({"action":"propose-decision","answer_id":finding.id,"decision_id":finding.handoff.decision_id,"value":{"text":finding.handoff.proposal_text,"entity_refs":finding.handoff.entity_refs},"source":format!("Unverified authored finding {} in findings SHA256 {}; checked citation identity only",finding.id,candidate::digest(&snapshots[2].1))}));
    }
    let answers = json!({"expected_base":f.expected_base_sha256,"answers":answers});
    let answer_bytes = serde_json::to_vec_pretty(&answers).map_err(|e| e.to_string())?;
    if answer_bytes.len() > INPUT {
        return Err("Generated answers exceed 1 MiB".into());
    }
    let (result, _) = candidate::evaluate_answers(&snapshots[3].1, &answer_bytes)?;
    if serde_json::to_vec_pretty(&result)
        .map_err(|e| e.to_string())?
        .len()
        > INPUT
    {
        return Err("Resulting candidate exceeds 1 MiB".into());
    }
    let receipt: Value = json!({"format":"caac-review-findings-receipt-v1","collection_sha256":f.expected_collection_sha256,"packet_sha256":f.expected_packet_sha256,"base_sha256":f.expected_base_sha256,"findings_sha256":candidate::digest(&snapshots[2].1),"answers_sha256":candidate::digest(&answer_bytes),"findings":f.findings,"citation_validation":"exact-identity-range-and-quote-checked","claim_truth":"not-assessed","candidate_application":"not-performed","owner_acceptance":"pending","completion":"handoff-only","limits":["Authored alignment and mismatch claims are not certified by citation or candidate validation.","Answers only propose responses to existing unresolved decisions; questions and requiredness remain unchanged. Explicit candidate apply is separate.","Unsigned receipt is not authenticated authority. Existing candidate apply does not verify this receipt or certify semantic truth.","receipt.json is written last; I/O failure may leave a partial directory. Inspect uncertain outcomes before retrying; never overwrite."]});
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?;
    if receipt_bytes.len() > ARTIFACT {
        return Err("Receipt exceeds 32 MiB".into());
    }
    let (_, rechecked) = review_packet::prepare(collection_path, request, destination)?;
    if rechecked != snapshots[1].1 {
        return Err("Packet inputs changed before publication".into());
    }
    for ((path, limit), (resolved, bytes)) in paths.iter().zip(limits).zip(&snapshots) {
        if source_intake::read_regular(path, limit)? != *bytes
            || fs::canonicalize(path).map_err(|e| e.to_string())? != *resolved
        {
            return Err("Handoff input changed before publication".into());
        }
    }
    Ok(Prepared {
        base_bytes: snapshots[3].1.clone(),
        answer_bytes,
        receipt_bytes,
    })
}

/// Explicit evidence inputs for invocation-time citation revalidation.
pub struct ApplyEvidence<'a> {
    pub collection: &'a Path,
    pub packet: &'a Path,
    pub findings: &'a Path,
    pub receipt: &'a Path,
    pub selections: &'a Request,
}
/// Revalidate exact handoff artifacts and apply the same checked base/answer bytes.
/// The v3 output does not certify that future standalone verification repeats this check.
pub fn apply_bound(
    base: &Path,
    answers: &Path,
    destination: &Path,
    evidence: &ApplyEvidence<'_>,
) -> Result<(), String> {
    output::require_absent(destination)?;
    let receipt_bytes = source_intake::read_regular(evidence.receipt, ARTIFACT)?;
    let receipt_identity = fs::canonicalize(evidence.receipt).map_err(|e| e.to_string())?;
    let answer_bytes = source_intake::read_regular(answers, INPUT)?;
    let answer_identity = fs::canonicalize(answers).map_err(|e| e.to_string())?;
    let key = output::destination_key(destination)?;
    for input in [&receipt_identity, &answer_identity] {
        if key == *input || key.starts_with(input) || input.starts_with(&key) {
            return Err("Output overlaps receipt or answers".into());
        }
    }
    let prepared = prepare(
        evidence.collection,
        evidence.packet,
        evidence.findings,
        base,
        evidence.selections,
        destination,
    )?;
    if receipt_bytes != prepared.receipt_bytes || answer_bytes != prepared.answer_bytes {
        return Err("Answers or receipt differ from the exact regenerated handoff".into());
    }
    let guard = || {
        let current = prepare(
            evidence.collection,
            evidence.packet,
            evidence.findings,
            base,
            evidence.selections,
            destination,
        )?;
        if current.base_bytes != prepared.base_bytes
            || current.answer_bytes != answer_bytes
            || current.receipt_bytes != receipt_bytes
            || source_intake::read_regular(evidence.receipt, ARTIFACT)? != receipt_bytes
            || fs::canonicalize(evidence.receipt).map_err(|e| e.to_string())? != receipt_identity
            || source_intake::read_regular(answers, INPUT)? != answer_bytes
            || fs::canonicalize(answers).map_err(|e| e.to_string())? != answer_identity
        {
            return Err("Evidence handoff changed before candidate publication".into());
        }
        Ok(())
    };
    candidate::apply_guarded(
        base,
        answers,
        destination,
        Some((&prepared.base_bytes, &answer_bytes)),
        Some(&guard),
    )
}
