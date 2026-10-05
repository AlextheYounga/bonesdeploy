use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;
use tempfile::TempDir;

use super::inventory::{BuildInventory, EntryType};
use crate::infra::git;

pub struct BuildContext {
    context: TempDir,
    pub revision: String,
    inventory: BuildInventory,
}

impl BuildContext {
    pub fn path(&self) -> &Path {
        self.context.path()
    }

    pub(crate) fn entry(&self, path: &Path) -> Option<super::inventory::Entry> {
        self.inventory.get(path)
    }

    pub(crate) fn replace_inventory(&mut self, inventory: BuildInventory) {
        self.inventory = inventory;
    }

    pub(crate) fn record_generated(&mut self, path: &Path, entry_type: EntryType, mode: u32) -> Result<()> {
        self.inventory.record(path, entry_type, mode)
    }

    #[cfg(test)]
    pub(crate) fn from_tempdir(context: TempDir, revision: String) -> Self {
        Self { context, revision, inventory: BuildInventory::default() }
    }
}

/// Exports the configured committed source tree for either deployment backend.
pub fn export(config: &Bones) -> Result<BuildContext> {
    let context =
        tempfile::Builder::new().prefix("bonesdeploy-build-").tempdir().context("Failed to create build context")?;
    let repo = env::current_dir().context("Failed to determine project directory")?;
    let revision = git::resolve_branch_commit(&repo, &config.branch)?;
    let inventory = git::export_commit(&repo, &revision, context.path())?;
    sanitize_exported_context(context.path())?;

    Ok(BuildContext { context, revision, inventory })
}

/// Removes the committed runtime environment before anything can read the
/// exported context. Build configuration remains available through `.env.build`.
pub fn sanitize_exported_context(source_context: &Path) -> Result<()> {
    let root_env = source_context.join(paths::DOT_ENV);
    match fs::symlink_metadata(&root_env) {
        Ok(metadata) if metadata.file_type().is_file() || metadata.file_type().is_symlink() => {
            fs::remove_file(&root_env).with_context(|| format!("Failed to remove exported {}", paths::DOT_ENV))?;
        }
        Ok(metadata) => {
            bail!("Exported {} has unsupported file type: {:?}", paths::DOT_ENV, metadata.file_type());
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("Failed to inspect exported {}", paths::DOT_ENV)),
    }
    Ok(())
}
