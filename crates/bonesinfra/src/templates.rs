use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

/// Resolves managed templates exclusively from a project's committed
/// `infra/templates/` snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateResolver {
    root: PathBuf,
}

impl TemplateResolver {
    #[must_use]
    pub fn new(project_root: &Path) -> Self {
        Self { root: project_root.join(paths::LOCAL_INFRA_TEMPLATES_DIR) }
    }

    /// Resolves one template path without allowing it to escape the managed
    /// project-local template tree.
    ///
    /// # Errors
    /// Returns an error for absolute paths, traversal, or missing templates.
    pub fn resolve(&self, relative: impl AsRef<Path>) -> Result<PathBuf> {
        let relative = relative.as_ref();
        if relative.is_absolute()
            || relative.components().any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
        {
            bail!("Template path must remain within {}", self.root.display());
        }
        let template_root = self
            .root
            .canonicalize()
            .with_context(|| format!("Failed to resolve project template root {}", self.root.display()))?;
        let template = template_root.join(relative);
        if !template.is_file() {
            bail!(
                "Project-local template {} is missing. Run `bonesdeploy update` to refresh managed templates.",
                template.display()
            );
        }
        let resolved = template
            .canonicalize()
            .with_context(|| format!("Failed to resolve project-local template {}", template.display()))?;
        if !resolved.starts_with(&template_root) {
            bail!("Template path must remain within {}", self.root.display());
        }
        Ok(resolved)
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
