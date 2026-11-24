//! Exact collection-bound evidence indexing, without semantic comparison.
use crate::{
    candidate::digest,
    contract_collection::{self, Request},
    output, source_intake,
};
use serde::Serialize;
use std::{fs, path::Path};
const MAX_ARTIFACT: usize = 32 * 1024 * 1024;
#[derive(Serialize)]
pub struct EvidenceRef {
    pub id: String,
    pub kind: &'static str,
    pub collection_pointer: String,
    pub file_sha256: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub observation_index: Option<usize>,
    pub authority: &'static str,
}
#[derive(Serialize)]
pub struct Question {
    pub topic: &'static str,
    pub status: &'static str,
    pub question: &'static str,
}
#[derive(Serialize)]
pub struct Packet {
    pub format: &'static str,
    pub collection_sha256: String,
    pub collection_byte_length: usize,
    pub collection_profile: &'static str,
    pub root: contract_collection::RootStatus,
    pub evidence: Vec<EvidenceRef>,
    pub questions: Vec<Question>,
    pub comparison: &'static str,
    pub owner_acceptance: &'static str,
    pub limits: Vec<&'static str>,
}
/// Recreate the exact canonical collection from explicit selections, then publish
/// an index. Never use paths from untrusted collection bytes to select inputs.
pub fn prepare_review(
    collection_path: &Path,
    request: &Request,
    destination: &Path,
) -> Result<Packet, String> {
    let (packet, encoded) = prepare(collection_path, request, destination)?;
    output::write_new(destination, &encoded)?;
    Ok(packet)
}
pub(crate) fn prepare(
    collection_path: &Path,
    request: &Request,
    destination: &Path,
) -> Result<(Packet, Vec<u8>), String> {
    output::require_absent(destination)?;
    let bytes = source_intake::read_regular(collection_path, MAX_ARTIFACT)?;
    let identity = fs::canonicalize(collection_path).map_err(|e| e.to_string())?;
    let output_key = output::destination_key(destination)?;
    if output_key == identity || output_key.starts_with(&identity) {
        return Err("Output overlaps collection input".into());
    }
    let (collection, canonical) = contract_collection::prepare(request, destination)?;
    if bytes != canonical {
        return Err("Collection must exactly match current collect-produced canonical bytes and explicit input selections".into());
    }
    let hash = digest(&bytes);
    let mut evidence = Vec::new();
    for (index, document) in collection.documents.iter().enumerate() {
        evidence.push(EvidenceRef {
            id: format!("{hash}:document:{index}"),
            kind: "document",
            collection_pointer: format!("/documents/{index}"),
            file_sha256: document.sha256.clone(),
            start_byte: 0,
            end_byte: document.byte_length,
            observation_index: None,
            authority: "unverified-selected-content",
        });
    }
    if let Some(baseline) = &collection.baseline {
        for (index, file) in baseline.evidence.files.iter().enumerate() {
            evidence.push(EvidenceRef {
                id: format!("{hash}:source:{index}"),
                kind: "source-file",
                collection_pointer: format!("/baseline/evidence/files/{index}"),
                file_sha256: file.sha256.clone(),
                start_byte: 0,
                end_byte: file.byte_length,
                observation_index: None,
                authority: "syntax-observation-only",
            });
            for (observation_index, observation) in file.analysis.observations.iter().enumerate() {
                if observation.start_byte > observation.end_byte
                    || observation.end_byte > file.byte_length
                {
                    return Err("Source observation range is outside its file".into());
                }
                evidence.push(EvidenceRef {
                    id: format!("{hash}:source:{index}:observation:{observation_index}"),
                    kind: "source-observation",
                    collection_pointer: format!(
                        "/baseline/evidence/files/{index}/analysis/observations/{observation_index}"
                    ),
                    file_sha256: file.sha256.clone(),
                    start_byte: observation.start_byte,
                    end_byte: observation.end_byte,
                    observation_index: Some(observation_index),
                    authority: "syntax-observation-only",
                });
            }
        }
    }
    if let Some(analyzer) = &collection.analyzer_evidence {
        for (index, artifact) in analyzer.artifacts.iter().enumerate() {
            evidence.push(EvidenceRef {
                id: format!("{hash}:analyzer:{index}"),
                kind: "analyzer-output",
                collection_pointer: format!("/analyzer_evidence/artifacts/{index}"),
                file_sha256: artifact.sha256.clone(),
                start_byte: 0,
                end_byte: artifact.byte_length,
                observation_index: None,
                authority: "unverified-analyzer-or-caller-claim",
            });
        }
    }
    let packet = Packet {
        format: "caac-review-packet-v1",
        collection_sha256: hash,
        collection_byte_length: bytes.len(),
        collection_profile: if collection.root.assessment_input_index.is_some() {
            "exact-canonical-collect-v4"
        } else if collection.analyzer_evidence.is_some() {
            "exact-canonical-collect-v3"
        } else if collection.run_inputs.is_empty() {
            "exact-canonical-collect-v1"
        } else {
            "exact-canonical-collect-v2"
        },
        root: collection.root,
        evidence,
        questions: vec![
            Question {
                topic: "root",
                status: "unresolved",
                question: "What establishes the project-level contract and its applicability within this selected scope?",
            },
            Question {
                topic: "coverage",
                status: "unresolved",
                question: "Which selected requirements have implementation support, and which relationships remain unverified?",
            },
            Question {
                topic: "mismatch",
                status: "unresolved",
                question: "Are there suspected requirement/implementation discrepancies? Separate cited observations from inference and unresolved intent.",
            },
            Question {
                topic: "execution",
                status: "unresolved",
                question: "What actual execution evidence exists? Syntax observations do not establish runtime behavior.",
            },
        ],
        comparison: "not-performed",
        owner_acceptance: "pending",
        limits: vec![
            "Evidence IDs are scoped to the exact collection hash; byte ranges are half-open original-file byte offsets. Document references cover whole documents, not inferred obligations.",
            "This packet does not duplicate document text or source observations; resolve collection_pointer against the exact supplied collection.",
            "Only current canonical collect-produced bytes are admitted. Whitespace changes, reordered selections, duplicate keys and any other edits are rejected, not normalized.",
            "Freshly recreated inputs are caller-selected, never taken from collection instructions or paths. Rechecks are snapshots, not locks or authenticated provenance.",
            "Questions are prompts, not discovered findings. Review notes must distinguish cited observations, inference and unresolved questions; citations do not prove the truth of a claim.",
            "Source observations may contain only signatures or partial syntax, not function bodies or behavioral coverage. Source-file references identify exact files; separately authorized raw-span inspection is additional work. No semantic comparison, execution evidence, root acceptance, candidate conversion or implementation authority is granted.",
        ],
    };
    let encoded = serde_json::to_vec_pretty(&packet).map_err(|e| e.to_string())?;
    if encoded.len() > MAX_ARTIFACT {
        return Err("Review packet exceeds 32 MiB artifact limit".into());
    }
    let (_, rechecked) = contract_collection::prepare(request, destination)?;
    if rechecked != bytes
        || source_intake::read_regular(collection_path, MAX_ARTIFACT)? != bytes
        || fs::canonicalize(collection_path).map_err(|e| e.to_string())? != identity
    {
        return Err("Collection or selected inputs changed before packet publication".into());
    }
    Ok((packet, encoded))
}
