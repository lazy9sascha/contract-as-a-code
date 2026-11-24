//! Explicit selected-document intake, not canonical requirements or acceptance.
use crate::{candidate::digest, output, source_intake};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const FILE_LIMIT: usize = 1024 * 1024;
const TOTAL_LIMIT: usize = 8 * FILE_LIMIT;
const ARTIFACT_LIMIT: usize = 32 * FILE_LIMIT;

#[derive(Default)]
pub struct Request {
    pub contracts: Vec<PathBuf>,
    pub run_inputs: Vec<PathBuf>,
    pub crawk: Option<crate::crawk_evidence::Selection>,
    pub root: Option<PathBuf>,
    pub root_assessment: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
}
#[derive(Serialize)]
pub struct Document {
    pub selected_paths: Vec<String>,
    pub resolved_path: String,
    pub sha256: String,
    pub byte_length: usize,
    pub text: String,
    pub role: &'static str,
    pub authority: &'static str,
}
#[derive(Serialize)]
pub struct RootStatus {
    pub finding: &'static str,
    pub reason: &'static str,
    pub document_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment_input_index: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RootAssessment {
    status: String,
    scope: String,
    reported_by: String,
}

impl RootAssessment {
    fn reason(text: &str) -> Result<&'static str, String> {
        if !text.trim_start().starts_with('{') {
            return Err("Root assessment must be a JSON object".into());
        }
        let report: Self =
            serde_json::from_str(text).map_err(|e| format!("Invalid root assessment: {e}"))?;
        if report.scope.trim().is_empty() || report.reported_by.trim().is_empty() {
            return Err("Root assessment requires a nonblank scope and reporter".into());
        }
        match report.status.as_str() {
            "not-found-in-search-scope" => Ok("not-found-in-search-scope"),
            "owner-confirmed-absent" => Ok("owner-confirmed-absent"),
            _ => Err("Unsupported root assessment status".into()),
        }
    }
}

#[derive(Serialize)]
pub struct BaselineLink {
    pub selected_path: String,
    pub resolved_path: String,
    pub sha256: String,
    pub byte_length: usize,
    pub evidence: source_intake::EvidenceContext,
}
#[derive(Serialize)]
pub struct Collection {
    pub format: &'static str,
    pub documents: Vec<Document>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub run_inputs: Vec<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_evidence: Option<crate::crawk_evidence::Evidence>,
    pub root: RootStatus,
    pub baseline: Option<BaselineLink>,
    pub owner_acceptance: &'static str,
    pub comparison: &'static str,
    pub coverage: &'static str,
    pub mismatch: &'static str,
    pub limits: Vec<&'static str>,
}
fn identity(path: &Path) -> Result<(PathBuf, String), String> {
    let selected = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let name = selected
        .to_str()
        .ok_or("Selected path must be UTF-8")?
        .to_string();
    Ok((selected, name))
}
pub(crate) fn capture(path: &Path, limit: usize) -> Result<(PathBuf, String, Vec<u8>), String> {
    let (selected, name) = identity(path)?;
    // Read the selected path itself, so final symlinks and special files are refused.
    let bytes = source_intake::read_regular(&selected, limit)?;
    let resolved = fs::canonicalize(&selected).map_err(|e| e.to_string())?;
    if resolved.to_str().is_none() {
        return Err("Resolved path must be UTF-8".into());
    }
    Ok((resolved, name, bytes))
}
fn baseline(path: &Path) -> Result<BaselineLink, String> {
    let (resolved, selected, bytes) = capture(path, 16 * FILE_LIMIT)?;
    let evidence = source_intake::load_evidence(&[], Some(path))?;
    let hash = digest(&bytes);
    if evidence.baseline_receipt.as_ref().is_none_or(|r| {
        r.baseline_sha256 != hash || r.baseline_resolved_path != resolved.to_string_lossy()
    }) {
        return Err("Baseline changed between capture and verification".into());
    }
    Ok(BaselineLink {
        selected_path: selected,
        resolved_path: resolved.to_string_lossy().into_owned(),
        sha256: hash,
        byte_length: bytes.len(),
        evidence,
    })
}
/// Capture explicitly selected inputs and publish one new collection artifact.
/// Rereads are best-effort snapshot fences, not locks or semantic validation.
pub fn collect(request: &Request, destination: &Path) -> Result<Collection, String> {
    let (collection, encoded) = prepare(request, destination)?;
    output::write_new(destination, &encoded)?;
    Ok(collection)
}
/// Recreate and recheck a collection from explicit caller-selected inputs only.
pub(crate) fn prepare(
    request: &Request,
    destination: &Path,
) -> Result<(Collection, Vec<u8>), String> {
    output::require_absent(destination)?;
    let output_key = output::destination_key(destination)?;
    if request.root.is_some() && request.root_assessment.is_some() {
        return Err("A root document and a missing-root assessment are mutually exclusive".into());
    }
    if request.contracts.len()
        + request.run_inputs.len()
        + usize::from(request.root.is_some())
        + usize::from(request.root_assessment.is_some())
        + if request.crawk.is_some() { 3 } else { 0 }
        > 128
    {
        return Err("At most 128 document and run-input selections are supported".into());
    }
    if request.contracts.is_empty() && request.root.is_none() && request.baseline.is_none() {
        return Err("Select documents, a root, or a source baseline".into());
    }
    let mut documents: Vec<Document> = Vec::new();
    let mut run_inputs: Vec<Document> = Vec::new();
    let mut total = 0;
    let mut root_index = None;
    for (path, is_root, is_context) in request
        .contracts
        .iter()
        .map(|p| (p, false, false))
        .chain(request.root.iter().map(|p| (p, true, false)))
        .chain(
            request
                .root_assessment
                .iter()
                .chain(&request.run_inputs)
                .map(|p| (p, false, true)),
        )
    {
        let (resolved, selected, bytes) = capture(path, FILE_LIMIT)?;
        if output_key == resolved || output_key.starts_with(&resolved) {
            return Err("Output overlaps a selected document".into());
        }
        let documents = if is_context {
            &mut run_inputs
        } else {
            &mut documents
        };
        let resolved_name = resolved.to_string_lossy().into_owned();
        if let Some(index) = documents
            .iter()
            .position(|d| d.resolved_path == resolved_name)
        {
            let document = &mut documents[index];
            if document.text.as_bytes() != bytes {
                return Err("Aliased document changed during capture".into());
            }
            if !document.selected_paths.contains(&selected) {
                document.selected_paths.push(selected);
            }
            if is_root {
                document.role = "caller-designated-root-unverified";
                root_index = Some(index);
            }
            continue;
        }
        total += bytes.len();
        if total > TOTAL_LIMIT {
            return Err("Documents and run inputs exceed 8 MiB total input limit".into());
        }
        let hash = digest(&bytes);
        let text = String::from_utf8(bytes).map_err(|_| "Selected contract must be UTF-8")?;
        if is_root {
            root_index = Some(documents.len());
        }
        documents.push(Document {
            selected_paths: vec![selected],
            resolved_path: resolved_name,
            sha256: hash,
            byte_length: text.len(),
            text,
            role: if is_context {
                "caller-supplied-run-input"
            } else if is_root {
                "caller-designated-root-unverified"
            } else {
                "selected-contract-unverified"
            },
            authority: if is_context {
                "non-normative-caller-context"
            } else {
                "unverified-selected-content"
            },
        });
    }
    let baseline = request.baseline.as_deref().map(baseline).transpose()?;
    if let Some(link) = &baseline {
        for input in std::iter::once(link.resolved_path.as_str())
            .chain(link.evidence.files.iter().map(|f| f.resolved_path.as_str()))
        {
            if output_key == Path::new(input) || output_key.starts_with(input) {
                return Err("Output overlaps baseline inputs".into());
            }
        }
    }
    let analyzer_evidence = request
        .crawk
        .as_ref()
        .map(|selection| {
            let b = baseline
                .as_ref()
                .ok_or("Analyzer evidence requires a baseline")?;
            crate::crawk_evidence::prepare(selection, b, destination)
        })
        .transpose()?;
    if let Some(e) = &analyzer_evidence {
        total += e.artifacts.iter().map(|a| a.byte_length).sum::<usize>();
        if total > TOTAL_LIMIT {
            return Err("Selected document, context and analyzer text exceeds 8 MiB".into());
        }
    }
    // The explicit assessment is the first retained run input, even when an alias is also selected.
    let reported_reason = request
        .root_assessment
        .as_ref()
        .map(|_| RootAssessment::reason(&run_inputs[0].text))
        .transpose()?;
    let mut collection = Collection {
        format: if reported_reason.is_some() {
            "caac-contract-collection-v4"
        } else if analyzer_evidence.is_some() {
            "caac-contract-collection-v3"
        } else if run_inputs.is_empty() {
            "caac-contract-collection-v1"
        } else {
            "caac-contract-collection-v2"
        },
        documents,
        run_inputs,
        analyzer_evidence,
        root: RootStatus {
            finding: if root_index.is_some() {
                "root-contract-unverified"
            } else {
                "missing-root-contract"
            },
            reason: if root_index.is_some() {
                "caller-designated-not-established-or-accepted"
            } else {
                reported_reason.unwrap_or("not-selected-in-declared-input-scope")
            },
            document_index: root_index,
            assessment_input_index: reported_reason.map(|_| 0),
        },
        baseline,
        owner_acceptance: "pending",
        comparison: "not-performed",
        coverage: "unverified",
        mismatch: "unverified",
        limits: vec![
            "Only explicitly selected files are captured; no recursive discovery or link following.",
            if reported_reason.is_some() {
                "The missing-root reason is a caller-supplied report for its stated scope. Reporter identity, coverage and project-wide absence are unverified."
            } else {
                "Root designation is unverified and does not establish a canonical root or acceptance. Missing root means not selected, not project-wide absence."
            },
            "Document text is retained exactly as UTF-8; no contract grammar, obligations, headings, or semantic equivalence are inferred.",
            "Source syntax observations are not requirements, execution evidence, or a source-to-candidate mapping.",
            "Unsigned local intake artifact; paths and selected content may contain private data. No publication or sharing is implied.",
            "Snapshot rereads are not locks; inputs may change after checking.",
        ],
    };
    if !collection.run_inputs.is_empty() {
        collection.limits.push("Run inputs are caller-supplied retained context, not requirement citations or filesystem authority. Actual model consumption, completeness, model identity, and missing system context are not observed.");
    }
    let encoded = serde_json::to_vec_pretty(&collection).map_err(|e| e.to_string())?;
    if encoded.len() > ARTIFACT_LIMIT {
        return Err("Collection exceeds 32 MiB artifact limit".into());
    }
    for document in collection.documents.iter().chain(&collection.run_inputs) {
        for selected in &document.selected_paths {
            let (resolved, _, bytes) = capture(Path::new(selected), FILE_LIMIT)?;
            if resolved != Path::new(&document.resolved_path) || bytes != document.text.as_bytes() {
                return Err("Document changed before publication".into());
            }
        }
    }
    if let Some(link) = &collection.baseline {
        let rechecked = self::baseline(Path::new(&link.selected_path))?;
        if serde_json::to_vec(&rechecked).map_err(|e| e.to_string())?
            != serde_json::to_vec(link).map_err(|e| e.to_string())?
        {
            return Err("Baseline or source observations changed before publication".into());
        }
    }
    if let (Some(selection), Some(expected)) = (&request.crawk, &collection.analyzer_evidence) {
        let current = crate::crawk_evidence::prepare(
            selection,
            collection.baseline.as_ref().unwrap(),
            destination,
        )?;
        if serde_json::to_vec(&current).map_err(|e| e.to_string())?
            != serde_json::to_vec(expected).map_err(|e| e.to_string())?
        {
            return Err("Analyzer inputs or source root changed before publication".into());
        }
    }
    Ok((collection, encoded))
}
