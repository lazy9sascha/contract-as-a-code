//! Read-only declared graph lookup. Connectivity is not conformance or causal impact.
use super::revisions::ExactReference;
use super::*;
use serde_json::{Value, json};
use std::collections::VecDeque;
fn reachable(start: &str, edges: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let mut seen = BTreeSet::from([start.to_string()]);
    let mut queue = VecDeque::from([start.to_string()]);
    while let Some(id) = queue.pop_front() {
        if let Some(targets) = edges.get(&id) {
            for target in targets {
                if seen.insert(target.clone()) {
                    queue.push_back(target.clone());
                }
            }
        }
    }
    seen.remove(start);
    seen
}
/// Returns one bounded terminal-safe JSON record after verifying and rereading inputs.
pub fn verify_trace(
    candidate_path: &Path,
    provenance_path: &Path,
    review_path: &Path,
    selected_id: &str,
) -> Result<String, String> {
    id(selected_id)?;
    let paths = [candidate_path, provenance_path, review_path];
    let limits = [MAX_INPUT, MAX_ARTIFACT, MAX_ARTIFACT];
    let bytes = paths
        .iter()
        .zip(limits)
        .map(|(p, l)| crate::source_intake::read_regular(p, l))
        .collect::<Result<Vec<_>, _>>()?;
    verify_bytes(&bytes[0], &bytes[1], &bytes[2])?;
    let candidate: Candidate = serde_json::from_slice(&bytes[0]).map_err(|e| e.to_string())?;
    let modules = module_artifacts::ModuleArtifacts::render(&candidate, bytes[2].len())?;
    modules.verify(review_path)?;
    let provenance: Value = serde_json::from_slice(&bytes[1]).map_err(|e| e.to_string())?;
    let manifest = revisions(&candidate)?;
    let references: BTreeMap<_, _> = manifest
        .objects
        .iter()
        .map(|o| {
            (
                o.id.clone(),
                ExactReference {
                    namespace: manifest.namespace.clone(),
                    package_revision: manifest.package_revision.clone(),
                    kind: o.kind.clone(),
                    id: o.id.clone(),
                    entity_revision: o.revision.clone(),
                },
            )
        })
        .collect();
    let selected = references
        .get(selected_id)
        .ok_or("Unknown exact trace ID")?;
    let bodies: BTreeMap<String, Value> = candidate
        .semantic
        .entities
        .iter()
        .map(|e| {
            Ok((
                e.id().to_string(),
                serde_json::to_value(e).map_err(|e| e.to_string())?,
            ))
        })
        .chain(candidate.semantic.unresolved.iter().map(|d| {
            Ok((
                d.id.clone(),
                serde_json::to_value(d).map_err(|e| e.to_string())?,
            ))
        }))
        .collect::<Result<_, String>>()?;
    let mut forward: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut reverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in &manifest.references {
        forward
            .entry(edge.source.id.clone())
            .or_default()
            .insert(edge.target.id.clone());
        reverse
            .entry(edge.target.id.clone())
            .or_default()
            .insert(edge.source.id.clone());
    }
    let f = reachable(selected_id, &forward);
    let r = reachable(selected_id, &reverse);
    let mut connected = f.union(&r).cloned().collect::<BTreeSet<_>>();
    connected.insert(selected_id.into());
    let cases=candidate.semantic.entities.iter().filter(|e|e.kind()=="case" && connected.contains(e.id())).map(|e|json!({"reference":references[e.id()],"body":bodies[e.id()],"execution_evidence":"not-recorded-in-supplied-artifacts"})).collect::<Vec<_>>();
    let decisions = candidate
        .semantic
        .unresolved
        .iter()
        .map(|d| json!({"reference":references[&d.id],"body":bodies[&d.id],"status":"unresolved"}))
        .collect::<Vec<_>>();
    let transition_policy = provenance
        .get("revision_transition")
        .and_then(|t| t.get("evidence_policy"))
        .cloned()
        .unwrap_or_else(|| json!("not-applicable"));
    let report = json!({"format":"caac-candidate-trace-v1","source":{"candidate_sha256":digest(&bytes[0]),"provenance_sha256":digest(&bytes[1]),"review_sha256":digest(&bytes[2]),"provenance_format":provenance["format"],"package_revision":manifest.package_revision},"artifact_consistency":"verified","selected":{"reference":selected,"body":bodies[selected_id],"mapping":manifest.presentation.iter().filter(|p|p.target.id==selected_id).collect::<Vec<_>>()},"direct_incoming":manifest.references.iter().filter(|e|e.target.id==selected_id).collect::<Vec<_>>(),"direct_outgoing":manifest.references.iter().filter(|e|e.source.id==selected_id).collect::<Vec<_>>(),"forward_reachable":f.iter().map(|id|&references[id]).collect::<Vec<_>>(),"reverse_reachable":r.iter().map(|id|&references[id]).collect::<Vec<_>>(),"reachable_declared_cases":cases,"unresolved_decisions":decisions,"evidence_scope":{"source_behavior_evidence":provenance["behavior_evidence"],"source_transition_evidence_policy":transition_policy,"execution_evidence":"not-recorded-in-supplied-artifacts","implementation_links":"not-represented","test_links":"not-represented","semantic_review":"not-supplied","owner_acceptance":candidate.authority.owner_acceptance,"implementation_admission":candidate.authority.implementation_admission,"completion":"not-assessed"},"limits":{"reachability":"directed declared graph paths only; not causation, impact, satisfaction or evidence coverage","case_selection":"selected object plus union of forward and reverse reachable objects; not undirected neighbors","decision_selection":"all unresolved decisions in this package, including unlinked decisions","trust":"unsigned artifact consistency; historical truth and owner acceptance are not authenticated","execution":"does not establish whether tests ran elsewhere"}});
    if json(&report)?.len() > MAX_ARTIFACT {
        return Err("Trace report exceeds serialization budget".into());
    }
    let encoded = crate::suggest::safe_json(&report);
    if encoded.len() >= MAX_ARTIFACT {
        return Err("Escaped trace report exceeds 32 MiB output budget".into());
    }
    for ((p, limit), expected) in paths.iter().zip(limits).zip(&bytes) {
        if crate::source_intake::read_regular(p, limit)? != *expected {
            return Err("Trace input changed before reporting".into());
        }
    }
    modules.verify(review_path)?;
    Ok(encoded)
}
