use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "updates/"]
pub struct UpdateAssets;

pub fn materialize_all(temp_dir: &Path) -> Result<()> {
    for file_path in UpdateAssets::iter() {
        let Some(asset) = UpdateAssets::get(&file_path) else {
            continue;
        };

        let dest = temp_dir.join(file_path.as_ref());
        write_asset_file(&dest, asset.data.as_ref())?;
    }

    Ok(())
}

fn write_asset_file(dest: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    fs::write(dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if dest.extension().is_some_and(|ext| ext == "yml" || ext == "yaml" || ext == "sh") {
        fs::set_permissions(dest, fs::Permissions::from_mode(0o644))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}

pub fn materialize_playbook(temp_dir: &Path) -> Result<PathBuf> {
    materialize_all(temp_dir)?;
    Ok(temp_dir.join("playbooks/update-bonesremote.yml"))
}

#[cfg(test)]
mod tests {
    use super::UpdateAssets;

    /// Ensures the update playbook is embedded as an asset.
    #[test]
    fn update_playbook_is_embedded() {
        assert!(UpdateAssets::get("playbooks/update-bonesremote.yml").is_some(), "update playbook should be embedded");
    }

    /// Ensures the migration manifest is embedded as an asset.
    #[test]
    fn migration_manifest_is_embedded() {
        assert!(UpdateAssets::get("migrations/manifest.yml").is_some(), "migration manifest should be embedded");
    }
}
