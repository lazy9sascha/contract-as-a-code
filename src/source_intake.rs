//! Bounded source syntax observations, never requirements or execution evidence.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::rc::Rc;
use std::{
    collections::HashMap,
    fs,
    io::Read,
    ops::ControlFlow,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tree_sitter::{Node, ParseOptions, Parser};

type Scope = Option<Rc<ScopeNode>>;
struct ScopeNode {
    name: String,
    parent: Scope,
    byte_length: usize,
}
fn extend_scope(scope: &Scope, name: String) -> Scope {
    Some(Rc::new(ScopeNode {
        byte_length: scope.as_ref().map_or(0, |p| p.byte_length) + name.len(),
        name,
        parent: scope.clone(),
    }))
}
fn scope_names(scope: &Scope) -> Vec<String> {
    let mut names = Vec::new();
    let mut current = scope.as_ref();
    while let Some(node) = current {
        names.push(node.name.clone());
        current = node.parent.as_ref();
    }
    names.reverse();
    names
}

const FORMAT: u32 = 1;
const PROFILE: &str = "source-syntax-v1";
const MAX_FILE: usize = 1024 * 1024;
const MAX_TOTAL: usize = 8 * MAX_FILE;
const MAX_BASELINE: usize = 16 * MAX_FILE;
const MAX_NODES: usize = 100_000;
const MAX_OBSERVATIONS: usize = 2048;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub authority: String,
    pub kind: String,
    pub name: Option<String>,
    pub enclosing_scope: Vec<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub excerpt: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Analysis {
    pub status: String,
    pub parser: Option<String>,
    pub scope: String,
    pub limitations: Vec<String>,
    pub observations: Vec<Observation>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub resolved_path: String,
    pub selected_paths: Vec<String>,
    pub sha256: String,
    pub byte_length: usize,
    pub analysis: Analysis,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub format_version: u32,
    pub profile: String,
    pub mode: String,
    pub files: Vec<SourceFile>,
    pub authority: String,
    pub execution: String,
    pub owner_acceptance: String,
}
pub(crate) struct CapturedFile {
    resolved: PathBuf,
    selected: Vec<PathBuf>,
    bytes: Vec<u8>,
    analysis: Analysis,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// NONBLOCK prevents FIFO open from waiting; it is not a filesystem I/O deadline.
pub(crate) fn read_regular(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    read_regular_then(path, limit, || {})
}
fn read_regular_then(
    path: &Path,
    limit: usize,
    after_read: impl FnOnce(),
) -> Result<Vec<u8>, String> {
    if !fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_file()
    {
        return Err(format!("Not a regular file: {}", path.display()));
    }
    #[cfg(unix)]
    let file = {
        use rustix::fs::{Mode, OFlags, open};
        fs::File::from(
            open(
                path,
                OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|e| format!("Cannot open {}: {e}", path.display()))?,
        )
    };
    #[cfg(not(unix))]
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let before = file.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() {
        return Err(format!("Not a regular file: {}", path.display()));
    }
    if before.len() > limit as u64 {
        return Err(format!(
            "File exceeds {limit} byte intake limit: {}",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    (&file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    after_read();
    let after = file.metadata().map_err(|e| e.to_string())?;
    let named = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if bytes.len() > limit
        || !same_metadata(&before, &after)
        || !same_metadata(&after, &named)
        || !named.file_type().is_file()
        || bytes.len() as u64 != after.len()
    {
        return Err(format!("Source changed during capture: {}", path.display()));
    }
    Ok(bytes)
}
fn same_metadata(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    let same = a.len() == b.len() && a.modified().ok() == b.modified().ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        same && a.dev() == b.dev()
            && a.ino() == b.ino()
            && a.ctime() == b.ctime()
            && a.ctime_nsec() == b.ctime_nsec()
    }
    #[cfg(not(unix))]
    {
        same
    }
}

pub(crate) fn capture_sources(paths: &[PathBuf]) -> Result<Vec<CapturedFile>, String> {
    if paths.len() > 128 {
        return Err("At most 128 selected source paths are supported".into());
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut indexes = HashMap::new();
    let mut files: Vec<CapturedFile> = Vec::new();
    let mut total = 0;
    for path in paths {
        let selected = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        let resolved = fs::canonicalize(&selected)
            .map_err(|e| format!("Cannot resolve {}: {e}", selected.display()))?;
        if selected.to_str().is_none() || resolved.to_str().is_none() {
            return Err("Source identities must use UTF-8 paths".into());
        }
        if let Some(index) = indexes.get(&resolved).copied() {
            let file: &mut CapturedFile = &mut files[index];
            if !file.selected.contains(&selected) {
                file.selected.push(selected);
            }
            continue;
        }
        let bytes = read_regular(&resolved, MAX_FILE)?;
        if fs::canonicalize(&selected).map_err(|e| e.to_string())? != resolved {
            return Err("Selected source changed during capture".into());
        }
        total += bytes.len();
        if total > MAX_TOTAL {
            return Err("Selected sources exceed the 8 MiB intake limit".into());
        }
        let analysis = analyze(&resolved, &bytes);
        indexes.insert(resolved.clone(), files.len());
        files.push(CapturedFile {
            resolved,
            selected: vec![selected],
            bytes,
            analysis,
        });
    }
    Ok(files)
}
pub(crate) fn make_baseline(mode: &str, base: &Path, files: Vec<CapturedFile>) -> Baseline {
    Baseline {
        format_version: FORMAT,
        profile: PROFILE.into(),
        mode: mode.into(),
        authority: "observation".into(),
        execution: "not-run".into(),
        owner_acceptance: "pending".into(),
        files: files
            .into_iter()
            .map(|file| SourceFile {
                resolved_path: file
                    .resolved
                    .to_str()
                    .expect("validated source path")
                    .to_owned(),
                path: relative_path(base, &file.resolved),
                selected_paths: file
                    .selected
                    .iter()
                    .map(|p| relative_path(base, p))
                    .collect(),
                sha256: digest(&file.bytes),
                byte_length: file.bytes.len(),
                analysis: file.analysis,
            })
            .collect(),
    }
}
pub(crate) fn encode(baseline: &Baseline) -> Result<String, String> {
    let text = serde_json::to_string_pretty(baseline).map_err(|e| e.to_string())?;
    if text.len() > MAX_BASELINE {
        return Err("Baseline exceeds the 16 MiB report limit".into());
    }
    Ok(text + "\n")
}
fn relative_path(base: &Path, target: &Path) -> String {
    let a: Vec<_> = base.components().collect();
    let b: Vec<_> = target.components().collect();
    let shared = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut path = PathBuf::new();
    for _ in shared..a.len() {
        path.push("..");
    }
    for part in &b[shared..] {
        path.push(part.as_os_str());
    }
    path.to_str()
        .expect("validated source and destination paths")
        .to_owned()
}
fn empty_analysis(status: &str, parser: Option<&str>) -> Analysis {
    Analysis { status: status.into(), parser: parser.map(str::to_owned), scope: "explicit syntax only; not behavior or requirements".into(), limitations: vec!["No source execution, imports, macro expansion, cfg evaluation, type or name resolution, decorator execution, or runtime failure inference.".into(), "Selected files only; external dependencies and whole-project semantic currentness are unknown.".into()], observations: Vec::new() }
}
fn analyze(path: &Path, bytes: &[u8]) -> Analysis {
    let Ok(source) = std::str::from_utf8(bytes) else {
        return empty_analysis("unsupported-encoding", None);
    };
    let (language, method, rust) = match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => (
            tree_sitter_rust::LANGUAGE.into(),
            "tree-sitter 0.27.2 / rust 0.24.2",
            true,
        ),
        Some("py") => (
            tree_sitter_python::LANGUAGE.into(),
            "tree-sitter 0.27.2 / python 0.25.0",
            false,
        ),
        _ => return empty_analysis("unsupported-language", None),
    };
    let mut analysis = empty_analysis("observed-syntax", Some(method));
    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        analysis.status = "unsupported-parser".into();
        return analysis;
    }
    let started = Instant::now();
    let mut progress = |_: &tree_sitter::ParseState| {
        if started.elapsed() > Duration::from_secs(1) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let options = ParseOptions::new().progress_callback(&mut progress);
    let tree = parser.parse_with_options(
        &mut |offset, _| bytes.get(offset..).unwrap_or_default(),
        None,
        Some(options),
    );
    let Some(tree) = tree else {
        analysis.status = "parser-budget-exceeded".into();
        return analysis;
    };
    if tree.root_node().has_error() {
        analysis.status = "parse-error".into();
        return analysis;
    }
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(
            source
                .bytes()
                .enumerate()
                .filter_map(|(i, b)| (b == b'\n').then_some(i + 1)),
        )
        .collect();
    let mut stack = vec![(tree.root_node(), None::<Rc<ScopeNode>>, 0usize)];
    let mut visited = 0;
    let mut excerpt_bytes = 0;
    while let Some((node, scope, depth)) = stack.pop() {
        visited += 1;
        if visited > MAX_NODES
            || depth > 128
            || analysis.observations.len() > MAX_OBSERVATIONS
            || excerpt_bytes > MAX_FILE
            || started.elapsed() > Duration::from_secs(2)
        {
            analysis.status = "parser-budget-exceeded".into();
            analysis.observations.clear();
            return analysis;
        }
        if node.is_error() || node.is_missing() {
            analysis.status = "parse-error".into();
            analysis.observations.clear();
            return analysis;
        }
        let kind = node.kind();
        let declaration = if rust {
            matches!(kind, "function_item" | "function_signature_item")
        } else {
            kind == "function_definition"
        };
        let failure_site = !rust && matches!(kind, "raise_statement" | "assert_statement");
        let decorator = !rust && kind == "decorator";
        let mut child_scope = scope.clone();
        if declaration || failure_site || decorator {
            if analysis.observations.len() >= MAX_OBSERVATIONS {
                analysis.status = "parser-budget-exceeded".into();
                analysis.observations.clear();
                return analysis;
            }
            let end = if declaration {
                node.child_by_field_name("body")
                    .map_or(node.end_byte(), |body| body.start_byte())
            } else {
                node.end_byte()
            };
            let Some(raw) = source.get(node.start_byte()..end) else {
                analysis.status = "invalid-source-span".into();
                analysis.observations.clear();
                return analysis;
            };
            let excerpt = raw.trim_end();
            if excerpt.len() > 16384 {
                analysis.status = "parser-budget-exceeded".into();
                analysis.observations.clear();
                return analysis;
            }
            let name = node
                .child_by_field_name("name")
                .and_then(|n| source.get(n.byte_range()))
                .map(str::to_owned);
            if declaration && name.as_ref().is_none_or(|name| name.is_empty()) {
                analysis.status = "invalid-source-span".into();
                analysis.observations.clear();
                return analysis;
            }
            let evidence_size = excerpt.len()
                + name.as_ref().map_or(0, String::len)
                + scope.as_ref().map_or(0, |scope| scope.byte_length);
            if excerpt_bytes + evidence_size > MAX_FILE {
                analysis.status = "parser-budget-exceeded".into();
                analysis.observations.clear();
                return analysis;
            }
            let end_byte = node.start_byte() + excerpt.len();
            let (end_line, end_column) = position(&line_starts, end_byte);
            excerpt_bytes += evidence_size;
            analysis.observations.push(Observation {
                authority: "observation".into(),
                kind: if declaration {
                    "declaration"
                } else if kind == "raise_statement" {
                    "raise-syntax"
                } else if decorator {
                    "decorator-syntax"
                } else {
                    "assert-syntax"
                }
                .into(),
                name: name.clone(),
                enclosing_scope: scope_names(&scope),
                start_byte: node.start_byte(),
                end_byte,
                start_line: node.start_position().row + 1,
                start_column: node.start_position().column + 1,
                end_line,
                end_column,
                excerpt: excerpt.into(),
            });
            if declaration && let Some(name) = name {
                child_scope = extend_scope(&scope, name);
            }
        } else if matches!(kind, "mod_item" | "trait_item" | "class_definition") {
            if let Some(name) = node
                .child_by_field_name("name")
                .and_then(|n| source.get(n.byte_range()))
            {
                child_scope = extend_scope(&scope, name.into());
            }
        } else if kind == "impl_item"
            && let Some(ty) = node
                .child_by_field_name("type")
                .and_then(|n| source.get(n.byte_range()))
        {
            let prefix = node
                .child_by_field_name("trait")
                .and_then(|n| source.get(n.byte_range()))
                .map_or_else(|| "impl ".to_owned(), |t| format!("impl {t} for "));
            child_scope = extend_scope(&scope, format!("{prefix}{ty}"));
        }
        let mut cursor = node.walk();
        let children: Vec<Node<'_>> = node.named_children(&mut cursor).collect();
        for child in children.into_iter().rev() {
            stack.push((child, child_scope.clone(), depth + 1));
        }
    }
    analysis
}
fn position(line_starts: &[usize], byte: usize) -> (usize, usize) {
    let row = line_starts.partition_point(|start| *start <= byte) - 1;
    (row + 1, byte - line_starts[row] + 1)
}

#[derive(Serialize)]
pub struct FileCheck {
    pub path: String,
    pub currentness: String,
    pub recorded_analysis_status: String,
    pub reason: String,
}
#[derive(Serialize)]
pub struct BaselineCheck {
    pub baseline_sha256: String,
    pub baseline_resolved_path: String,
    pub profile: String,
    pub scope: String,
    pub files: Vec<FileCheck>,
    pub all_current: bool,
    pub execution: String,
    pub owner_acceptance: String,
}
fn verify(
    baseline: &Baseline,
    base: &Path,
    baseline_sha256: String,
    baseline_resolved_path: String,
) -> Result<BaselineCheck, String> {
    if baseline.format_version != FORMAT
        || baseline.profile != PROFILE
        || !matches!(baseline.mode.as_str(), "plugin" | "migration")
        || baseline.authority != "observation"
        || baseline.execution != "not-run"
        || baseline.owner_acceptance != "pending"
    {
        return Err("Unsupported baseline format, profile, or authority fields".into());
    }
    if baseline.files.is_empty() || baseline.files.len() > 128 {
        return Err("Invalid baseline file count".into());
    }
    if baseline
        .files
        .iter()
        .try_fold(0usize, |sum, file| sum.checked_add(file.byte_length))
        .is_none_or(|total| total > MAX_TOTAL)
    {
        return Err("Baseline exceeds total source budget".into());
    }
    let mut seen = std::collections::HashSet::new();
    let mut report = BaselineCheck {
        baseline_sha256,
        baseline_resolved_path,
        profile: PROFILE.into(),
        scope: "Selected captured files and recorded syntax only; dependency and semantic currentness remain unknown.".into(),
        files: Vec::new(),
        all_current: true,
        execution: "not-run".into(),
        owner_acceptance: "pending".into(),
    };
    for file in &baseline.files {
        if !Path::new(&file.resolved_path).is_absolute()
            || Path::new(&file.path).is_absolute()
            || file.selected_paths.is_empty()
            || file.selected_paths.len() > 128
            || file
                .selected_paths
                .iter()
                .any(|p| Path::new(p).is_absolute())
            || file.sha256.len() != 64
            || !file.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || file.byte_length > MAX_FILE
        {
            return Err("Invalid baseline source record".into());
        }
        if !seen.insert(&file.resolved_path) {
            return Err("Duplicate resolved source in baseline".into());
        }
        let expected = base.join(&file.path);
        let status = (|| -> Result<(&str, String), String> {
            let resolved = fs::canonicalize(&expected)
                .map_err(|e| format!("Recorded source is unavailable: {e}"))?;
            // Canonical recorded path identifies the selected target; aliases remain separately checked.
            if Path::new(&file.resolved_path) != resolved {
                return Ok((
                    "retargeted",
                    "Recorded resolved path no longer identifies the same target".into(),
                ));
            }
            for selected in &file.selected_paths {
                if fs::canonicalize(base.join(selected))
                    .map_err(|e| format!("Selected source is unavailable: {e}"))?
                    != resolved
                {
                    return Ok((
                        "retargeted",
                        "Selected path resolves to a different source".into(),
                    ));
                }
            }
            let bytes = read_regular(&resolved, MAX_FILE)?;
            if digest(&bytes) != file.sha256 || bytes.len() != file.byte_length {
                return Ok((
                    "stale",
                    "Source bytes differ from the recorded revision".into(),
                ));
            }
            if analyze(&resolved, &bytes) != file.analysis {
                return Ok((
                    "invalid-evidence",
                    "Recorded syntax evidence or parser profile does not match this revision"
                        .into(),
                ));
            }
            Ok(("current","Recorded bytes and bounded syntax evidence match; no semantic verification is implied".into()))
        })();
        let (currentness, reason) = match status {
            Ok((state, reason)) => (state.to_owned(), reason),
            Err(reason) => ("unavailable".into(), reason),
        };
        report.all_current &= currentness == "current";
        report.files.push(FileCheck {
            path: file.path.clone(),
            currentness,
            recorded_analysis_status: file.analysis.status.clone(),
            reason,
        });
    }
    Ok(report)
}
/// A captured syntax context, not requirements or ongoing currentness.
#[derive(Serialize)]
pub struct EvidenceContext {
    pub kind: &'static str,
    pub profile: &'static str,
    pub authority: &'static str,
    pub execution: &'static str,
    pub owner_acceptance: &'static str,
    pub source_base: String,
    pub files: Vec<SourceFile>,
    pub baseline_receipt: Option<BaselineCheck>,
    pub timing: &'static str,
}

pub fn load_evidence(
    sources: &[PathBuf],
    historical: Option<&Path>,
) -> Result<EvidenceContext, String> {
    if historical.is_some() && !sources.is_empty() {
        return Err("--source and --baseline are mutually exclusive".into());
    }
    if let Some(path) = historical {
        let (baseline, report) = inspect_baseline(path).map_err(|(_, message)| message)?;
        let base = Path::new(&report.baseline_resolved_path)
            .parent()
            .ok_or("Baseline has no parent")?;
        if !report.all_current {
            return Err("Baseline is stale, retargeted, unavailable, or has invalid evidence; historical evidence cannot support a current question".into());
        }
        Ok(EvidenceContext {
            kind: "source-syntax-context",
            profile: PROFILE,
            authority: "observation",
            execution: "not-run",
            owner_acceptance: "pending",
            source_base: base
                .to_str()
                .ok_or("Baseline base must be UTF-8")?
                .to_owned(),
            files: baseline.files,
            baseline_receipt: Some(report),
            timing: "Checked snapshot only; files may change after capture. No semantic currentness is established.",
        })
    } else {
        let base = fs::canonicalize(std::env::current_dir().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let identity = base.to_str().ok_or("Source base must be UTF-8")?.to_owned();
        let baseline = make_baseline("plugin", &base, capture_sources(sources)?);
        encode(&baseline)?;
        Ok(EvidenceContext {
            kind: "source-syntax-context",
            profile: PROFILE,
            authority: "observation",
            execution: "not-run",
            owner_acceptance: "pending",
            source_base: identity,
            files: baseline.files,
            baseline_receipt: None,
            timing: "Freshly captured snapshot only; files may change after capture. No semantic currentness is established.",
        })
    }
}

fn inspect_baseline(path: &Path) -> Result<(Baseline, BaselineCheck), (i32, String)> {
    let absolute = fs::canonicalize(path).map_err(|e| (2, e.to_string()))?;
    let identity = absolute
        .to_str()
        .ok_or((2, "Baseline path must be UTF-8".into()))?;
    let bytes = read_regular(&absolute, MAX_BASELINE).map_err(|e| (2, e))?;
    let baseline: Baseline = serde_json::from_slice(&bytes)
        .map_err(|e| (1, format!("Unsupported or malformed baseline: {e}")))?;
    let report = verify(
        &baseline,
        absolute.parent().unwrap_or(Path::new(".")),
        digest(&bytes),
        identity.to_owned(),
    )
    .map_err(|e| (1, e))?;
    Ok((baseline, report))
}

pub fn run_check(path: &Path) -> i32 {
    match inspect_baseline(path) {
        Ok((_, report)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("serialize baseline report")
            );
            if report.all_current { 0 } else { 1 }
        }
        Err((code, error)) => {
            eprintln!("ERROR: {error}");
            code
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_detects_replacement_with_identical_bytes() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("file.rs");
        let replacement = d.path().join("replacement.rs");
        fs::write(&p, "fn same() {}\n").unwrap();
        fs::write(&replacement, "fn same() {}\n").unwrap();
        assert!(read_regular_then(&p, MAX_FILE, || fs::rename(&replacement, &p).unwrap()).is_err());
    }

    #[test]
    fn capture_detects_change_after_read() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("file.rs");
        fs::write(&p, "fn old() {}\n").unwrap();
        assert!(
            read_regular_then(&p, MAX_FILE, || fs::write(&p, "fn replacement() {}\n")
                .unwrap())
            .is_err()
        );
    }
}
