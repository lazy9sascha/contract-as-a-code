//! Retained crawk output claims, not analyzer execution or compiler resolution.
use crate::contract_collection::{BaselineLink, Document, capture};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
const LIMIT: usize = 1024 * 1024;
// Bound repeated ambiguity lists before allocating per-edge copies.
const ENDPOINT_MATCH_LIMIT: usize = 65_536;
#[derive(Default)]
pub struct Selection {
    pub modules: PathBuf,
    pub deps: PathBuf,
    pub snapshot: PathBuf,
    pub source_root: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    revision: String,
    files: Vec<SnapshotFile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotFile {
    path: String,
    sha256: String,
}
#[derive(Serialize, Clone)]
pub struct Module {
    pub target: String,
    pub module: String,
    pub start_byte: usize,
    pub end_byte: usize,
}
#[derive(Serialize)]
pub struct Endpoint {
    pub label: String,
    pub inventory_matches: Vec<usize>,
    pub status: &'static str,
}
#[derive(Serialize)]
pub struct Dependency {
    pub source: Endpoint,
    pub target: Endpoint,
    pub api_annotation: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
}
#[derive(Serialize)]
pub struct MatchedFile {
    pub baseline_index: usize,
    pub manifest_path: String,
    pub sha256: String,
}
#[derive(Serialize)]
pub struct Evidence {
    pub profile: &'static str,
    pub selected_source_root: String,
    pub resolved_source_root: String,
    pub snapshot_revision_claim: String,
    pub artifacts: Vec<Document>,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
    pub selected_files_match: Vec<MatchedFile>,
    pub unverified_manifest_files: Vec<String>,
    pub analyzer_producer: &'static str,
    pub analyzer_options: &'static str,
    pub snapshot_association: &'static str,
    pub limits: Vec<&'static str>,
}
fn module_label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 512
        && s.split("::").all(|part| {
            let p = part.strip_prefix("r#").unwrap_or(part);
            !p.is_empty()
                && p.bytes().enumerate().all(|(i, b)| {
                    matches!(b, b'_' | b'-')
                        || b.is_ascii_alphabetic()
                        || (i > 0 && b.is_ascii_digit())
                })
        })
}
fn target_label(s: &str) -> bool {
    s == "lib"
        || ["bin:", "test:"].iter().any(|prefix| {
            s.strip_prefix(prefix).is_some_and(|v| {
                !v.is_empty()
                    && v.len() <= 128
                    && v.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
            })
        })
}
fn lines(text: &str) -> Result<Vec<(usize, usize, &str)>, String> {
    let mut offset = 0;
    let mut out = Vec::new();
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let body = body.strip_suffix('\r').unwrap_or(body);
        if body.is_empty() || body.chars().any(|c| c.is_control()) {
            return Err("Empty or control-containing analyzer output line".into());
        }
        out.push((offset, offset + body.len(), body));
        offset += line.len();
    }
    Ok(out)
}
fn endpoint(label: &str, matches: &[usize]) -> Endpoint {
    let status = match matches.len() {
        0 => "unresolved",
        1 => "inventory-unambiguous",
        _ => "ambiguous",
    };
    Endpoint {
        label: label.into(),
        inventory_matches: matches.to_vec(),
        status,
    }
}
pub(crate) fn prepare(
    selection: &Selection,
    baseline: &BaselineLink,
    destination: &Path,
) -> Result<Evidence, String> {
    if [
        &selection.modules,
        &selection.deps,
        &selection.snapshot,
        &selection.source_root,
    ]
    .iter()
    .any(|p| p.as_os_str().is_empty())
    {
        return Err("All four crawk selections are required".into());
    }
    let selected_root = if selection.source_root.is_absolute() {
        selection.source_root.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(&selection.source_root)
    };
    let metadata = fs::symlink_metadata(&selected_root).map_err(|e| e.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Analyzer source root must be a selected nonsymlink directory".into());
    }
    let root = fs::canonicalize(&selected_root).map_err(|e| e.to_string())?;
    let root_string = root
        .to_str()
        .ok_or("Source root must be UTF-8")?
        .to_string();
    let key = crate::output::destination_key(destination)?;
    let mut artifacts = Vec::new();
    for (path, role) in [
        (&selection.modules, "crawk-module-output"),
        (&selection.deps, "crawk-dependency-output"),
        (&selection.snapshot, "caller-snapshot-manifest"),
    ] {
        let (resolved, selected, bytes) = capture(path, LIMIT)?;
        if key == resolved || key.starts_with(&resolved) || resolved.starts_with(&key) {
            return Err("Output overlaps analyzer input".into());
        }
        let text = String::from_utf8(bytes).map_err(|_| "Analyzer input must be UTF-8")?;
        artifacts.push(Document {
            selected_paths: vec![selected],
            resolved_path: resolved.to_string_lossy().into_owned(),
            sha256: crate::candidate::digest(text.as_bytes()),
            byte_length: text.len(),
            text,
            role,
            authority: "unverified-analyzer-or-caller-claim",
        });
    }
    let snapshot: Snapshot = serde_json::from_str(&artifacts[2].text).map_err(|e| e.to_string())?;
    if snapshot.revision.is_empty()
        || snapshot.revision.len() > 128
        || !snapshot
            .revision
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        || snapshot.files.is_empty()
        || snapshot.files.len() > 4096
    {
        return Err("Invalid snapshot revision or file count".into());
    }
    let mut inventory = BTreeMap::new();
    for f in snapshot.files {
        let path = Path::new(&f.path);
        if f.path.is_empty()
            || f.path.len() > 4096
            || f.path.contains('\\')
            || f.path.chars().any(char::is_control)
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || f.path
                .split('/')
                .any(|x| x.is_empty() || x == "." || x == "..")
            || f.sha256.len() != 64
            || !f
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || inventory.insert(f.path, f.sha256).is_some()
        {
            return Err("Invalid or duplicate manifest path/checksum".into());
        }
    }
    let mut matched = Vec::new();
    for (index, file) in baseline.evidence.files.iter().enumerate() {
        let relative = Path::new(&file.resolved_path)
            .strip_prefix(&root)
            .map_err(|_| "Baseline source is outside declared analyzer root")?
            .to_str()
            .ok_or("Source path must be UTF-8")?;
        if inventory.get(relative) != Some(&file.sha256) {
            return Err("Selected baseline file does not match snapshot manifest".into());
        }
        matched.push(MatchedFile {
            baseline_index: index,
            manifest_path: relative.into(),
            sha256: file.sha256.clone(),
        });
    }
    if matched.is_empty() {
        return Err("Analyzer evidence requires selected baseline files".into());
    }
    let unverified = inventory
        .keys()
        .filter(|p| !matched.iter().any(|m| m.manifest_path == **p))
        .cloned()
        .collect();
    let mut modules = Vec::new();
    let mut seen = BTreeSet::new();
    for (start, end, line) in lines(&artifacts[0].text)? {
        let (tag, label) = line
            .strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .ok_or("Module output requires explicit target tags")?;
        if !label.starts_with(' ') {
            return Err("Invalid module output separator".into());
        }
        let label = label.trim();
        if !target_label(tag)
            || !module_label(label)
            || !seen.insert((tag.to_string(), label.to_string()))
            || modules.len() >= 4096
        {
            return Err("Invalid, duplicate or excessive module records".into());
        }
        modules.push(Module {
            target: tag.into(),
            module: label.into(),
            start_byte: start,
            end_byte: end,
        });
    }
    if modules.is_empty() {
        return Err("Module inventory must be nonempty".into());
    }
    let mut by_label: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, module) in modules.iter().enumerate() {
        by_label.entry(&module.module).or_default().push(index);
    }
    let mut match_slots = 0usize;
    let mut dependencies = Vec::new();
    let mut seen = BTreeSet::new();
    for (start, end, line) in lines(&artifacts[1].text)? {
        let (source, tail) = line
            .split_once(" -> ")
            .ok_or("Invalid dependency output separator")?;
        let (target, annotation) = match tail.split_once(" [") {
            Some((target, rest)) => (
                target,
                Some(
                    rest.strip_suffix(']')
                        .ok_or("Invalid dependency API annotation")?,
                ),
            ),
            None => (tail, None),
        };
        if !module_label(source)
            || !module_label(target)
            || annotation.is_some_and(|a| a.is_empty() || a.len() > 16384)
            || !seen.insert((source.to_string(), target.to_string()))
            || dependencies.len() >= 8192
        {
            return Err("Invalid, duplicate or excessive dependency records".into());
        }
        let source_matches = by_label.get(source).map(Vec::as_slice).unwrap_or(&[]);
        let target_matches = by_label.get(target).map(Vec::as_slice).unwrap_or(&[]);
        let edge_slots = source_matches.len() + target_matches.len();
        if edge_slots > ENDPOINT_MATCH_LIMIT - match_slots {
            return Err(
                "Aggregate endpoint inventory-match resource limit exceeded (65536 slots)".into(),
            );
        }
        match_slots += edge_slots;
        dependencies.push(Dependency {
            source: endpoint(source, source_matches),
            target: endpoint(target, target_matches),
            api_annotation: annotation.map(str::to_owned),
            start_byte: start,
            end_byte: end,
        });
    }
    Ok(Evidence {
        profile: "crawk-0.7.1-plain-v1",
        selected_source_root: selected_root
            .to_str()
            .ok_or("Source root must be UTF-8")?
            .into(),
        resolved_source_root: root_string,
        snapshot_revision_claim: snapshot.revision,
        artifacts,
        modules,
        dependencies,
        selected_files_match: matched,
        unverified_manifest_files: unverified,
        analyzer_producer: "not-observed",
        analyzer_options: "not-observed",
        snapshot_association: "caller-claim-not-authenticated",
        limits: vec![
            "Only explicit selected baseline files match manifest hashes; remaining manifest paths are never opened. Whole graph currentness is not established.",
            "The profile identifies output grammar, not an authenticated analyzer version, binary or invocation. No analyzer is executed.",
            "Endpoint matches identify inventory labels only, not compiler-resolved targets. Ambiguous and missing endpoints remain unresolved.",
            "Missing edges, target failures, cfg/macros, dynamic relations and shared storage coupling are not repaired or inferred. Counts do not establish responsibilities or contract sizes.",
            "Raw analyzer citations prove output text only, not source spans, requirements or behavior.",
        ],
    })
}
