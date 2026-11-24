use super::{Candidate, MAX_ARTIFACT, render};
use std::path::Path;

/// Derived module files named by the candidate. The review index and all module
/// files share one size budget.
pub(super) struct ModuleArtifacts(Vec<(String, String)>);

impl ModuleArtifacts {
    pub(super) fn render(candidate: &Candidate, index_len: usize) -> Result<Self, String> {
        let files = render::module_reviews(candidate)?;
        let total = files
            .iter()
            .fold(index_len, |len, (_, view)| len.saturating_add(view.len()));
        if total > MAX_ARTIFACT {
            return Err("Review index and modules exceed the artifact size limit".into());
        }
        Ok(Self(files))
    }

    pub(super) fn verify(&self, index_path: &Path) -> Result<(), String> {
        let directory = index_path.parent().unwrap_or_else(|| Path::new("."));
        for (name, view) in &self.0 {
            let path = directory.join(name);
            let observed = crate::source_intake::read_regular(&path, view.len())
                .map_err(|error| format!("Cannot verify module review {name}: {error}"))?;
            if observed != view.as_bytes() {
                return Err(format!("Module review {name} differs from its projection"));
            }
        }
        Ok(())
    }

    pub(super) fn publish(&self, destination: &Path) -> Result<(), String> {
        for (name, view) in &self.0 {
            crate::output::write_new(&destination.join(name), view.as_bytes())?;
        }
        Ok(())
    }
}
