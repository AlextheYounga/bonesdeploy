use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use shared::{config, paths, registry};
use walkdir::WalkDir;

use crate::privileges;

const ALLOWED_TOP_LEVEL_ENTRIES: &[&str] =
    &[paths::BONES_TOML, paths::RUNTIME_TOML, paths::DEPLOYMENT_DIR, paths::HOOKS_DIR];

/// # Errors
///
/// Returns an error if the dataset is invalid or the control-plane state cannot
/// be updated safely.
pub fn import(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote site import")?;
    registry::validate_site_name(site)?;

    let sites_root = paths::bonesremote_sites_root();
    fs::create_dir_all(&sites_root).with_context(|| format!("Failed to create {}", sites_root.display()))?;

    let staging_dir = unique_site_path(&sites_root, site, "incoming");
    fs::create_dir_all(&staging_dir).with_context(|| format!("Failed to create {}", staging_dir.display()))?;

    extract_stdin_archive(&staging_dir)?;
    validate_site_dataset(site, &staging_dir)?;
    write_registry(&staging_dir)?;
    replace_site_dir(site, &staging_dir)?;
    println!("Imported site state for {site}.");
    Ok(())
}

/// # Errors
///
/// Returns an error if the site state does not exist or cannot be archived.
pub fn export(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote site export")?;
    registry::validate_site_name(site)?;

    let site_root = paths::bonesremote_site_root(site);
    if !site_root.is_dir() {
        bail!("Remote site state does not exist: {}", site_root.display());
    }

    validate_site_dataset(site, &site_root)?;
    stream_site_archive(&site_root)
}

fn extract_stdin_archive(destination: &Path) -> Result<()> {
    let status = Command::new("tar")
        .args(["-xzf", "-", "-C"])
        .arg(destination)
        .status()
        .context("Failed to run tar for site import")?;

    if status.success() {
        return Ok(());
    }

    bail!("Failed to extract remote site dataset")
}

fn validate_site_dataset(site: &str, root: &Path) -> Result<()> {
    validate_top_level_entries(root)?;
    reject_symlinks(root)?;

    let bones_path = root.join(paths::BONES_TOML);
    if !bones_path.is_file() {
        bail!("Missing {} in imported site dataset", paths::BONES_TOML);
    }

    let bones = config::load(&bones_path)?;
    if bones.project_name != site {
        bail!("Imported site dataset is for '{}', expected '{}'", bones.project_name, site);
    }

    Ok(())
}

fn validate_top_level_entries(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("Failed to read {}", root.display()))? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { bail!("Imported dataset contains a non-UTF-8 entry") };

        if ALLOWED_TOP_LEVEL_ENTRIES.contains(&name) {
            continue;
        }

        bail!("Imported dataset contains unsupported entry: {name}");
    }

    Ok(())
}

fn reject_symlinks(root: &Path) -> Result<()> {
    for entry in WalkDir::new(root).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            bail!("Imported dataset cannot contain symlinks: {}", entry.path().display());
        }
    }

    Ok(())
}

fn write_registry(root: &Path) -> Result<()> {
    let bones = config::load(&root.join(paths::BONES_TOML))?;
    let registry = registry::Registry::derive(&bones);
    let registry_toml = toml::to_string_pretty(&registry).context("Failed to serialize site registry")?;
    fs::write(root.join(paths::REGISTRY_TOML), registry_toml).context("Failed to write site registry")?;
    Ok(())
}

fn replace_site_dir(site: &str, staging_dir: &Path) -> Result<()> {
    let site_root = paths::bonesremote_site_root(site);
    let backup_dir = unique_site_path(&paths::bonesremote_sites_root(), site, "backup");
    let had_existing = site_root.exists();

    if had_existing {
        fs::rename(&site_root, &backup_dir)
            .with_context(|| format!("Failed to move existing site state {} out of the way", site_root.display()))?;
    }

    if let Err(error) = fs::rename(staging_dir, &site_root) {
        if had_existing {
            fs::rename(&backup_dir, &site_root).ok();
        }
        return Err(error).with_context(|| format!("Failed to activate {}", site_root.display()));
    }

    if had_existing {
        fs::remove_dir_all(&backup_dir).with_context(|| format!("Failed to remove {}", backup_dir.display()))?;
    }

    Ok(())
}

fn stream_site_archive(site_root: &Path) -> Result<()> {
    let mut command = Command::new("tar");
    command.arg("-czf").arg("-").arg("-C").arg(site_root);

    for entry in ALLOWED_TOP_LEVEL_ENTRIES {
        let path = site_root.join(entry);
        if path.exists() {
            command.arg(entry);
        }
    }

    let status = command.stdout(Stdio::inherit()).status().context("Failed to run tar for site export")?;
    if status.success() {
        return Ok(());
    }

    bail!("Failed to export remote site dataset")
}

fn unique_site_path(parent: &Path, site: &str, suffix: &str) -> PathBuf {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    parent.join(format!(".{site}.{suffix}.{stamp}"))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::validate_top_level_entries;

    #[test]
    fn validate_top_level_entries_rejects_unexpected_file() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-site-test-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("oops.txt"), "bad")?;

        let result = validate_top_level_entries(&root);

        fs::remove_dir_all(&root)?;
        assert!(result.is_err());
        Ok(())
    }
}
