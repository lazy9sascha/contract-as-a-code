//! Explicit pending revision transitions; never an automatic proposal consumer.
use super::revisions::{ObjectKind, ReferenceBinding};
use super::*;
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Change {
    id: String,
    kind: ObjectKind,
    before_revision: String,
    after_revision: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Transition {
    parent_candidate_json: String,
    parent_candidate_sha256: String,
    parent_package_revision: String,
    parent_provenance_sha256: String,
    parent_review_sha256: String,
    input_proposal_sha256: String,
    changed_objects: Vec<Change>,
    reference_preconditions: Vec<ReferenceBinding>,
    evidence_policy: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RevisionProvenance<T> {
    pub(super) format: String,
    pub(super) renderer_profile: String,
    pub(super) candidate_sha256: String,
    pub(super) review_sha256: String,
    pub(super) semantic_sha256: String,
    pub(super) revision_manifest: RevisionManifest,
    pub(super) revision_transition: T,
    pub(super) required_unresolved: usize,
    pub(super) contract_complete: bool,
    pub(super) owner_acceptance: String,
    pub(super) implementation_admission: String,
    pub(super) behavior_evidence: String,
    pub(super) exception_review: String,
}
pub(super) fn sha(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn transition(
    parent_json: &str,
    target: &Candidate,
    parent_provenance_sha256: &str,
    input_proposal_sha256: &str,
) -> Result<Transition, String> {
    if parent_json.len() > MAX_INPUT
        || !sha(parent_provenance_sha256)
        || !sha(input_proposal_sha256)
    {
        return Err("Invalid transition input bounds or checksum shape".into());
    }
    let parent: Candidate = serde_json::from_str(parent_json).map_err(|e| e.to_string())?;
    let before = revisions(&parent)?;
    let after = revisions(target)?;
    if parent.semantic.entities.len() != target.semantic.entities.len() {
        return Err("Entity insertion/deletion not admitted".into());
    }
    let mut skeleton = parent.clone();
    skeleton.semantic.entities = target.semantic.entities.clone();
    if json(&skeleton)? != json(target)? {
        return Err("Only existing entity bodies may change".into());
    }
    let mut changes = vec![];
    for (a, b) in parent
        .semantic
        .entities
        .iter()
        .zip(&target.semantic.entities)
    {
        if a.id() != b.id() || a.kind() != b.kind() {
            return Err("Stable ID, kind and entity order must remain unchanged".into());
        }
        let old = before
            .objects
            .iter()
            .find(|o| o.id == a.id())
            .ok_or("Missing parent object")?;
        let new = after
            .objects
            .iter()
            .find(|o| o.id == b.id())
            .ok_or("Missing revised object")?;
        if old.revision != new.revision {
            changes.push(Change {
                id: a.id().into(),
                kind: old.kind.clone(),
                before_revision: old.revision.clone(),
                after_revision: new.revision.clone(),
            });
        }
    }
    if changes.is_empty() {
        return Err("No-op revision is not admitted".into());
    }
    let mut preconditions = after.references.clone();
    for edge in &mut preconditions {
        for reference in [&mut edge.source, &mut edge.target] {
            let old = before
                .objects
                .iter()
                .find(|o| o.id == reference.id && o.kind == reference.kind)
                .ok_or("Target missing from exact parent package")?;
            reference.namespace = before.namespace.clone();
            reference.package_revision = before.package_revision.clone();
            reference.entity_revision = old.revision.clone();
        }
    }
    Ok(Transition {
        parent_candidate_json: parent_json.into(),
        parent_candidate_sha256: digest(parent_json.as_bytes()),
        parent_package_revision: before.package_revision,
        parent_provenance_sha256: parent_provenance_sha256.into(),
        parent_review_sha256: digest(render(&parent)?.as_bytes()),
        input_proposal_sha256: input_proposal_sha256.into(),
        changed_objects: changes,
        reference_preconditions: preconditions,
        evidence_policy: "fresh-review-required-for-whole-package".into(),
    })
}
pub(super) fn verify_v4(
    candidate_bytes: &[u8],
    provenance_bytes: &[u8],
    review_bytes: &[u8],
) -> Result<(), String> {
    let candidate: Candidate =
        serde_json::from_slice(candidate_bytes).map_err(|e| e.to_string())?;
    let p: RevisionProvenance<Transition> =
        serde_json::from_slice(provenance_bytes).map_err(|e| e.to_string())?;
    if p.format != "caac-candidate-provenance-v4"
        || p.renderer_profile != render::profile(&candidate)
    {
        return Err("Unsupported revision provenance/profile".into());
    }
    verify_current(&candidate, candidate_bytes, review_bytes, &p)?;
    let expected = transition(
        &p.revision_transition.parent_candidate_json,
        &candidate,
        &p.revision_transition.parent_provenance_sha256,
        &p.revision_transition.input_proposal_sha256,
    )?;
    if expected != p.revision_transition {
        return Err(
            "Transition receipt does not match parent snapshot and revised candidate".into(),
        );
    }
    Ok(())
}
/// Explicit invocation creates a new pending revision; source files are immutable.
pub fn revise(
    candidate_path: &Path,
    provenance_path: &Path,
    review_path: &Path,
    proposal_path: &Path,
    destination: &Path,
) -> Result<(), String> {
    crate::output::require_absent(destination)?;
    let paths = [candidate_path, provenance_path, review_path, proposal_path];
    let limits = [MAX_INPUT, MAX_ARTIFACT, 4 * 1024 * 1024, MAX_ARTIFACT];
    let key = crate::output::destination_key(destination)?;
    let mut bytes = vec![];
    for (p, limit) in paths.iter().zip(limits) {
        let input = crate::output::destination_key(p)?;
        if key.starts_with(&input) || input.starts_with(&key) {
            return Err("Output overlaps revision input".into());
        }
        bytes.push(crate::source_intake::read_regular(p, limit)?);
    }
    verify_bytes(&bytes[0], &bytes[1], &bytes[2])?;
    let proposal =
        super::compare::strict(std::str::from_utf8(&bytes[3]).map_err(|e| e.to_string())?)?;
    let edited = proposal["raw_review_change"]["after"]
        .as_str()
        .ok_or("Proposal must retain edited review")?;
    let snapshots = vec![
        bytes[0].clone(),
        bytes[1].clone(),
        bytes[2].clone(),
        edited.as_bytes().to_vec(),
    ];
    let expected = super::compare::proposal_from_bytes(&snapshots)?;
    if proposal != expected {
        return Err(
            "Proposal does not match exact verified source and recomputed edit comparison".into(),
        );
    }
    if expected["status"] != "proposed"
        || expected["replacements"]
            .as_array()
            .is_none_or(|r| r.is_empty())
    {
        return Err("Only nonempty conflict-free proposed entity changes may be revised".into());
    }
    let mut target: Candidate = serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
    let parent_manifest = revisions(&target)?;
    for replacement in expected["replacements"].as_array().unwrap() {
        let id = replacement["id"].as_str().ok_or("Missing target ID")?;
        let e = target
            .semantic
            .entities
            .iter_mut()
            .find(|e| e.id() == id)
            .ok_or("Missing revision target")?;
        let revision = parent_manifest
            .objects
            .iter()
            .find(|r| r.id == id)
            .ok_or("Missing expected target revision")?;
        if replacement["expected_revision"] != revision.revision
            || replacement["before"] != serde_json::to_value(&e).map_err(|e| e.to_string())?
        {
            return Err("Stale entity revision precondition".into());
        }
        *e = serde_json::from_value::<Entity>(replacement["after"].clone())
            .map_err(|e| e.to_string())?;
    }
    validate(&target)?;
    let parent_json = std::str::from_utf8(&bytes[0]).map_err(|e| e.to_string())?;
    let receipt = transition(parent_json, &target, &digest(&bytes[1]), &digest(&bytes[3]))?;
    let candidate = json(&target)?;
    let review = render(&target)?;
    let provenance = RevisionProvenance {
        format: "caac-candidate-provenance-v4".into(),
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
    let provenance = json(&provenance)?;
    publish(
        &paths,
        &limits,
        &bytes,
        destination,
        &candidate,
        &provenance,
        &review,
    )?;
    Ok(())
}

pub(super) fn verify_current<T>(
    candidate: &Candidate,
    candidate_bytes: &[u8],
    review_bytes: &[u8],
    p: &RevisionProvenance<T>,
) -> Result<(), String> {
    verify_revisions(candidate, &p.revision_manifest)?;
    if p.candidate_sha256 != digest(candidate_bytes)
        || p.review_sha256 != digest(review_bytes)
        || p.semantic_sha256 != digest(&json(&candidate.semantic)?)
        || render(candidate)?.as_bytes() != review_bytes
    {
        return Err("Revised artifact checksum or projection mismatch".into());
    }
    if p.contract_complete
        || p.owner_acceptance != "pending"
        || p.implementation_admission != "pending"
        || p.behavior_evidence != "not-run"
        || p.exception_review != "pending"
        || p.required_unresolved
            != candidate
                .semantic
                .unresolved
                .iter()
                .filter(|d| d.required)
                .count()
    {
        return Err("Revision cannot inherit authority or evidence".into());
    }
    Ok(())
}

pub(super) fn publish(
    paths: &[&Path],
    limits: &[usize],
    bytes: &[Vec<u8>],
    destination: &Path,
    candidate: &[u8],
    provenance: &[u8],
    review: &str,
) -> Result<(), String> {
    if candidate.len() > MAX_INPUT || review.len() > MAX_ARTIFACT || provenance.len() > MAX_ARTIFACT
    {
        return Err("Revised artifacts exceed publication budget".into());
    }
    verify_bytes(candidate, provenance, review.as_bytes())?;
    let target: Candidate = serde_json::from_slice(candidate).map_err(|e| e.to_string())?;
    let target_modules = module_artifacts::ModuleArtifacts::render(&target, review.len())?;
    let parent: Candidate = serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
    let parent_modules = module_artifacts::ModuleArtifacts::render(&parent, bytes[2].len())?;
    for ((p, limit), expected) in paths.iter().zip(limits).zip(bytes) {
        if crate::source_intake::read_regular(p, *limit)? != *expected {
            return Err("Revision input changed before publication".into());
        }
    }
    parent_modules.verify(paths[2])?;
    fs::create_dir(destination).map_err(|e| e.to_string())?;
    crate::output::write_new(&destination.join("candidate.json"), candidate)?;
    crate::output::write_new(&destination.join("review.md"), review.as_bytes())?;
    target_modules.publish(destination)?;
    crate::output::write_new(&destination.join("provenance.json"), provenance)?;
    Ok(())
}
