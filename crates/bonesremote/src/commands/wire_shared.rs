use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result, bail};
use shared::config;
use shared::paths;

use crate::privileges;
use crate::release_state;

fn validate_site_name(site: &str) -> Result<()> {
    if site.is_empty() {
        bail!("Site name cannot be empty");
    }

    if site.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        return Ok(());
    }

    bail!("Invalid site name: {site}")
}

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release wire")?;
    validate_site_name(site)?;

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path).context("Failed to load remote site state")?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg.project_root, &release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let shared_dir = release_state::shared_dir(&cfg.project_root);
    if !shared_dir.is_dir() {
        bail!(
            "Shared root is missing: {}. Run 'bonesdeploy remote setup' or runtime provisioning first.",
            shared_dir.display()
        );
    }

    for leaf in paths::SHARED_LEAVES {
        let target = shared_dir.join(leaf);
        ensure_shared_leaf(&target)?;
        link_relative(&release_dir, leaf, &target)?;
    }

    link_public_storage(&release_dir)?;

    Ok(())
}

fn ensure_shared_leaf(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    bail!("Required shared path is missing: {}. Provision the runtime shared paths before deploying.", path.display())
}

fn link_relative(release_dir: &Path, relative: &str, target: &Path) -> Result<()> {
    let link_path = release_dir.join(relative);
    remove_if_present(&link_path)?;
    symlink(target, &link_path)
        .with_context(|| format!("Failed to link {} -> {}", link_path.display(), target.display()))?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory {}", path.display()))?;
    }
    Ok(())
}

fn link_public_storage(release_dir: &Path) -> Result<()> {
    let public_dir = release_dir.join("public");
    if !public_dir.is_dir() {
        return Ok(());
    }

    let link_path = public_dir.join("storage");
    remove_if_present(&link_path)?;
    symlink(Path::new("../storage/app/public"), &link_path)
        .with_context(|| format!("Failed to link {} -> ../storage/app/public", link_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process;

    use anyhow::Result;

    use super::{ensure_shared_leaf, link_public_storage, link_relative, remove_if_present};

    fn temp_dir(label: &str) -> Result<PathBuf> {
        let dir = env::temp_dir().join(format!("bonesremote-wire-{label}-{}", process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn ensure_shared_leaf_requires_existing_path() -> Result<()> {
        let root = temp_dir("ensure_leaves")?;
        let missing = root.join("storage");
        assert!(ensure_shared_leaf(&missing).is_err());

        fs::create_dir_all(&missing)?;
        ensure_shared_leaf(&missing)?;

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn link_relative_creates_symlink_to_shared_target() -> Result<()> {
        let root = temp_dir("link_relative")?;
        let shared = root.join("shared/.env");
        let parent = shared.parent().ok_or_else(|| anyhow::anyhow!("shared test path should have a parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(&shared, "FOO=bar\n")?;
        fs::set_permissions(&shared, PermissionsExt::from_mode(0o600))?;

        let release = root.join("releases/now");
        fs::create_dir_all(&release)?;
        link_relative(&release, ".env", &shared)?;

        let link = release.join(".env");
        assert!(link.is_symlink());
        let linked_target = fs::read_link(&link)?;
        assert_eq!(linked_target, shared);
        assert_eq!(fs::read_to_string(&link)?, "FOO=bar\n");

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn remove_if_present_handles_files_dirs_and_missing() -> Result<()> {
        let root = temp_dir("remove_if_present")?;
        let missing = root.join("missing");
        remove_if_present(&missing)?;

        let file = root.join("file.txt");
        fs::write(&file, "x")?;
        remove_if_present(&file)?;
        assert!(!file.exists());

        let dir = root.join("dir");
        fs::create_dir_all(&dir)?;
        remove_if_present(&dir)?;
        assert!(!dir.exists());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn link_public_storage_links_into_release_storage_tree() -> Result<()> {
        let root = temp_dir("public_storage")?;
        let release = root.join("releases/now");
        fs::create_dir_all(release.join("public"))?;

        link_public_storage(&release)?;

        let link = release.join("public/storage");
        assert!(link.is_symlink());
        assert_eq!(fs::read_link(&link)?, PathBuf::from("../storage/app/public"));

        fs::remove_dir_all(&root).ok();
        Ok(())
    }
}
