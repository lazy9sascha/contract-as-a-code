//! Safe publication of generated artifacts. Existing files require explicit ownership.
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

/// Resolve a destination through its parent without following a final symlink.
/// This also resolves aliases for destinations whose parent does not exist yet.
pub fn destination_key(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let name = absolute.file_name().ok_or("Output must name a file")?;
    let mut parent = absolute
        .parent()
        .ok_or("Output must have a parent")?
        .to_path_buf();
    let mut missing = Vec::new();
    while !parent.exists() {
        missing.push(
            parent
                .file_name()
                .ok_or("Cannot resolve output parent")?
                .to_owned(),
        );
        parent = parent
            .parent()
            .ok_or("Cannot resolve output parent")?
            .to_path_buf();
    }
    let mut resolved = fs::canonicalize(parent).map_err(|e| e.to_string())?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    resolved.push(name);
    Ok(resolved)
}

pub fn require_absent(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(format!("Output already exists: {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Cannot inspect {}: {e}", path.display())),
    }
}

fn stage(path: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    for _ in 0..100 {
        let serial = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(".caac-{}-{serial}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temporary) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(bytes).and_then(|()| file.sync_all()) {
                    let _ = fs::remove_file(&temporary);
                    return Err(e.to_string());
                }
                return Ok(temporary);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("Cannot allocate temporary output file".to_owned())
}

/// Publish complete bytes without overwriting an existing file or symlink.
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    require_absent(path)?;
    let temporary = stage(path, bytes)?;
    // A same-directory hard link publishes atomically and refuses any existing target.
    let result = fs::hard_link(&temporary, path)
        .map_err(|e| format!("Cannot create {}: {e}", path.display()));
    let _ = fs::remove_file(temporary);
    result
}

/// Replace a file produced by this sync session only if its bytes remain unchanged.
/// This detects edits between polls; it is not a lock against simultaneous writers.
pub fn replace_owned(path: &Path, expected: &[u8], bytes: &[u8]) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() || fs::read(path).map_err(|e| e.to_string())? != expected {
        return Err(format!(
            "Output changed outside this sync session: {}",
            path.display()
        ));
    }
    let temporary = stage(path, bytes)?;
    let result = (|| {
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !metadata.file_type().is_file() || fs::read(path).map_err(|e| e.to_string())? != expected
        {
            return Err(format!(
                "Output changed outside this sync session: {}",
                path.display()
            ));
        }
        fs::rename(&temporary, path).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(temporary);
    result
}
