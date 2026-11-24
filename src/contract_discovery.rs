//! Metadata-only suggestions in explicit bounded directory scopes.
use serde::Serialize;
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
};
const MAX_DIRECTORIES: usize = 16;
const MAX_ENTRIES: usize = 4096;
const MAX_SUGGESTIONS: usize = 128;
const MAX_OUTPUT: usize = 32 * 1024 * 1024;
#[derive(Debug, Serialize)]
pub struct Suggestion {
    pub path: String,
    pub directory: String,
    pub filename: String,
    pub byte_length: u64,
    pub reason: &'static str,
    pub authority: &'static str,
}
#[derive(Debug, Default, Serialize)]
pub struct Omissions {
    pub hidden: usize,
    pub directories: usize,
    pub symlinks: usize,
    pub special_files: usize,
    pub other_extensions: usize,
}
#[derive(Debug, Default)]
pub struct Options {
    pub depth: u8,
}
#[derive(Debug, Serialize)]
pub struct Frontier {
    pub path: String,
    pub status: &'static str,
}
#[derive(Debug, Serialize)]
pub struct Discovery {
    pub format: &'static str,
    pub directories: Vec<String>,
    pub depth: u8,
    pub visited_directories: Vec<String>,
    pub frontier: Vec<Frontier>,
    pub suggestions: Vec<Suggestion>,
    pub omissions: Omissions,
    pub outcome: &'static str,
    pub root_assessment: &'static str,
    pub comparison: &'static str,
    pub owner_acceptance: &'static str,
    pub limits: Vec<&'static str>,
}
fn hidden(path: &Path) -> bool {
    path.components().any(|c| match c {
        Component::Normal(s) => s.to_string_lossy().starts_with('.'),
        _ => false,
    })
}
fn directory(path: &Path) -> Result<(PathBuf, fs::Metadata), String> {
    if hidden(path) {
        return Err("Hidden directory components are not supported".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_dir() {
        return Err("Selected scope must be a directory, not a symlink or special file".into());
    }
    let resolved = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if hidden(&resolved) {
        return Err("Canonical scope has hidden directory components".into());
    }
    if resolved.to_str().is_none() {
        return Err("Directory path must be UTF-8".into());
    }
    Ok((resolved, metadata))
}
fn same(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    let equal = a.len() == b.len() && a.modified().ok() == b.modified().ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        equal
            && a.dev() == b.dev()
            && a.ino() == b.ino()
            && a.ctime() == b.ctime()
            && a.ctime_nsec() == b.ctime_nsec()
    }
    #[cfg(not(unix))]
    {
        equal
    }
}
/// List immediate suggestions without opening document bodies.
/// Metadata rechecks detect observed changes but do not lock a directory tree.
pub fn discover(paths: &[PathBuf]) -> Result<Discovery, String> {
    discover_with_options(paths, &Options::default())
}
/// Opt-in bounded-depth metadata traversal; depth zero examines selected roots only.
pub fn discover_with_options(paths: &[PathBuf], options: &Options) -> Result<Discovery, String> {
    if options.depth > 4 {
        return Err("Depth must be between 0 and 4".into());
    }
    if paths.is_empty() || paths.len() > MAX_DIRECTORIES {
        return Err("Select between 1 and 16 directories".into());
    }
    let mut scopes = BTreeSet::new();
    let mut snapshots = Vec::new();
    for path in paths {
        let (resolved, metadata) = directory(path)?;
        scopes.insert(resolved.clone());
        snapshots.push((path.clone(), resolved, metadata));
    }
    let mut suggestions = Vec::new();
    let mut omissions = Omissions::default();
    let mut entries = 0;
    let mut files = Vec::new();
    let mut queue: VecDeque<_> = scopes.iter().map(|p| (p.clone(), 0u8)).collect();
    let mut visited = BTreeSet::new();
    let mut frontier = BTreeSet::new();
    while let Some((scope, depth)) = queue.pop_front() {
        if !visited.insert(scope.clone()) {
            continue;
        }
        let (resolved, metadata) = directory(&scope)?;
        if resolved != scope {
            return Err("Traversal directory identity changed".into());
        }
        snapshots.push((scope.clone(), resolved, metadata));
        for entry in fs::read_dir(&scope).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            entries += 1;
            if entries > MAX_ENTRIES {
                return Err(
                    "Discovery exceeds 4096 directory entries; narrow explicit scopes".into(),
                );
            }
            let name = entry.file_name();
            if name.to_string_lossy().starts_with('.') {
                omissions.hidden += 1;
                continue;
            }
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            let kind = metadata.file_type();
            if kind.is_symlink() {
                omissions.symlinks += 1;
                continue;
            }
            if kind.is_dir() {
                let (resolved, before) = directory(&path)?;
                if resolved != path {
                    return Err("Child directory identity changed".into());
                }
                snapshots.push((path.clone(), resolved.clone(), before));
                if depth < options.depth {
                    queue.push_back((resolved, depth + 1));
                } else {
                    frontier.insert(resolved);
                }
                continue;
            }
            if !kind.is_file() {
                omissions.special_files += 1;
                continue;
            }
            let name = name.to_str().ok_or("Visible filename must be UTF-8")?;
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(extension.as_str(), "md" | "markdown") {
                omissions.other_extensions += 1;
                continue;
            }
            if suggestions.len() >= MAX_SUGGESTIONS {
                return Err("Discovery exceeds 128 suggestions; narrow explicit scopes".into());
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            suggestions.push(Suggestion {
                path: path.to_str().ok_or("Suggestion path must be UTF-8")?.into(),
                directory: scope.to_str().unwrap().into(),
                filename: name.into(),
                byte_length: metadata.len(),
                reason: if matches!(stem.as_str(), "readme" | "index") {
                    "navigation-name-hint-only"
                } else {
                    "markdown-in-explicit-directory"
                },
                authority: "unverified-selection-suggestion",
            });
            files.push((path, metadata));
        }
    }
    for (path, before) in files {
        let after = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !after.file_type().is_file() || !same(&before, &after) {
            return Err("Suggested file metadata changed during discovery".into());
        }
    }
    for (path, resolved, before) in snapshots {
        let (after_path, after) = directory(&path)?;
        if after_path != resolved || !same(&before, &after) {
            return Err("Directory changed during discovery".into());
        }
    }
    frontier.retain(|p| !visited.contains(p));
    omissions.directories = frontier.len();
    suggestions.sort_by(|a, b| a.path.cmp(&b.path));
    let result = Discovery {
        format: "caac-contract-discovery-v1",
        directories: scopes
            .into_iter()
            .map(|p| p.to_str().unwrap().to_owned())
            .collect(),
        depth: options.depth,
        visited_directories: visited
            .into_iter()
            .map(|p| p.to_str().unwrap().to_owned())
            .collect(),
        frontier: frontier
            .into_iter()
            .map(|p| Frontier {
                path: p.to_str().unwrap().to_owned(),
                status: "depth-limit-not-traversed",
            })
            .collect(),
        outcome: if suggestions.is_empty() {
            "no-candidates-in-declared-scope"
        } else {
            "selection-required"
        },
        suggestions,
        omissions,
        root_assessment: "not-established-by-metadata-inventory",
        comparison: "not-performed",
        owner_acceptance: "pending",
        limits: vec![
            "Bounded metadata inventory only; default depth zero is nonrecursive; no document bodies, instructions, links, Git status, or semantic roles are inspected.",
            "Visible Markdown filenames, including untracked files, are suggestions only. README/index hints do not determine correctness or root authority.",
            "Hidden entries, symlinks, special files and other extensions are omitted; frontier directories are not traversed at the selected depth. Empty results do not establish project-wide contract absence.",
            "Paths and metadata may be private; this result does not authorize sharing or collection. Explicit selection is required for collect.",
            "Metadata rechecks detect observed changes, not all concurrent races; this is not an atomic filesystem snapshot or security boundary against concurrent replacement.",
        ],
    };
    terminal_json(&result)?;
    Ok(result)
}
/// Encode one bounded terminal-safe record, with no surrounding success prose.
pub fn terminal_json(result: &Discovery) -> Result<String, String> {
    let encoded = crate::suggest::safe_json(result);
    if encoded.len() > MAX_OUTPUT {
        return Err("Discovery exceeds 32 MiB serialized output limit".into());
    }
    Ok(encoded)
}
