//! Explicit presentation-only transitions. No semantic or approval edits.
use super::revise::{RevisionProvenance, publish, sha, verify_current};
use super::revisions::{ObjectRevision, PresentationBinding};
use super::*;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    format: String,
    expected_candidate_sha256: String,
    expected_package_revision: String,
    presentation: Presentation,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Transition {
    operation: String,
    parent_candidate_json: String,
    parent_candidate_sha256: String,
    parent_package_revision: String,
    parent_provenance_sha256: String,
    parent_review_sha256: String,
    input_presentation_sha256: String,
    before_presentation_revision: String,
    after_presentation_revision: String,
    preserved_semantic_revision: String,
    preserved_objects: Vec<ObjectRevision>,
    before_mapping: Vec<PresentationBinding>,
    after_mapping: Vec<PresentationBinding>,
    evidence_policy: String,
}
fn transition(
    parent_json: &str,
    target: &Candidate,
    parent_provenance_sha256: &str,
    input_presentation_sha256: &str,
) -> Result<Transition, String> {
    if parent_json.len() > MAX_INPUT
        || !sha(parent_provenance_sha256)
        || !sha(input_presentation_sha256)
    {
        return Err("Invalid presentation transition bounds/checksum shape".into());
    }
    let parent: Candidate = serde_json::from_str(parent_json).map_err(|e| e.to_string())?;
    let before = revisions(&parent)?;
    let after = revisions(target)?;
    let mut skeleton = parent.clone();
    skeleton.presentation = target.presentation.clone();
    if json(&skeleton)? != json(target)?
        || before.semantic_revision != after.semantic_revision
        || before.objects != after.objects
    {
        return Err(
            "Presentation revision must preserve entire semantic object, authority and format"
                .into(),
        );
    }
    if before.presentation_revision == after.presentation_revision {
        return Err("No-op presentation revision is not admitted".into());
    }
    Ok(Transition {
        operation: "replace-presentation".into(),
        parent_candidate_json: parent_json.into(),
        parent_candidate_sha256: digest(parent_json.as_bytes()),
        parent_package_revision: before.package_revision,
        parent_provenance_sha256: parent_provenance_sha256.into(),
        parent_review_sha256: digest(render(&parent)?.as_bytes()),
        input_presentation_sha256: input_presentation_sha256.into(),
        before_presentation_revision: before.presentation_revision,
        after_presentation_revision: after.presentation_revision,
        preserved_semantic_revision: before.semantic_revision,
        preserved_objects: before.objects,
        before_mapping: before.presentation,
        after_mapping: after.presentation,
        evidence_policy: "fresh-review-required-for-whole-package".into(),
    })
}
pub(super) fn verify_v5(
    candidate_bytes: &[u8],
    provenance_bytes: &[u8],
    review_bytes: &[u8],
) -> Result<(), String> {
    let candidate: Candidate =
        serde_json::from_slice(candidate_bytes).map_err(|e| e.to_string())?;
    let p: RevisionProvenance<Transition> =
        serde_json::from_slice(provenance_bytes).map_err(|e| e.to_string())?;
    if p.format != "caac-candidate-provenance-v5"
        || p.renderer_profile != render::profile(&candidate)
    {
        return Err("Unsupported presentation provenance/profile".into());
    }
    verify_current(&candidate, candidate_bytes, review_bytes, &p)?;
    let expected = transition(
        &p.revision_transition.parent_candidate_json,
        &candidate,
        &p.revision_transition.parent_provenance_sha256,
        &p.revision_transition.input_presentation_sha256,
    )?;
    if expected != p.revision_transition {
        return Err("Presentation receipt does not match parent and current package".into());
    }
    Ok(())
}
/// Explicit request replaces only presentation, producing a fresh pending package.
pub fn revise_presentation(
    candidate_path: &Path,
    provenance_path: &Path,
    review_path: &Path,
    request_path: &Path,
    destination: &Path,
) -> Result<(), String> {
    crate::output::require_absent(destination)?;
    let paths = [candidate_path, provenance_path, review_path, request_path];
    let limits = [MAX_INPUT, MAX_ARTIFACT, MAX_ARTIFACT, MAX_INPUT];
    let key = crate::output::destination_key(destination)?;
    let mut bytes = vec![];
    for (p, limit) in paths.iter().zip(limits) {
        let input = crate::output::destination_key(p)?;
        if key.starts_with(&input) || input.starts_with(&key) {
            return Err("Output overlaps presentation input".into());
        }
        bytes.push(crate::source_intake::read_regular(p, limit)?);
    }
    verify_bytes(&bytes[0], &bytes[1], &bytes[2])?;
    let request: Request = serde_json::from_slice(&bytes[3]).map_err(|e| e.to_string())?;
    let mut target: Candidate = serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
    if request.format != "caac-presentation-request-v1"
        || request.expected_candidate_sha256 != digest(&bytes[0])
        || request.expected_package_revision != revisions(&target)?.package_revision
    {
        return Err("Unsupported request or stale exact candidate/package precondition".into());
    }
    target.presentation = request.presentation;
    validate(&target)?;
    let receipt = transition(
        std::str::from_utf8(&bytes[0]).map_err(|e| e.to_string())?,
        &target,
        &digest(&bytes[1]),
        &digest(&bytes[3]),
    )?;
    let candidate = json(&target)?;
    let review = render(&target)?;
    let provenance = RevisionProvenance {
        format: "caac-candidate-provenance-v5".into(),
        renderer_profile: render::profile(&target).into(),
        candidate_sha256: digest(&candidate),
        review_sha256: digest(review.as_bytes()),
        semantic_sha256: digest(&json(&target.semantic)?),
        revision_manifest: revisions(&target)?,
        revision_transition: receipt,
        required_unresolved: target
            .semantic
            .unresolved
            .iter()
            .filter(|d| d.required)
            .count(),
        contract_complete: false,
        owner_acceptance: "pending".into(),
        implementation_admission: "pending".into(),
        behavior_evidence: "not-run".into(),
        exception_review: "pending".into(),
    };
    publish(
        &paths,
        &limits,
        &bytes,
        destination,
        &candidate,
        &json(&provenance)?,
        &review,
    )
}
