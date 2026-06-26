use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result, bail};
use shared::{paths, registry};

use crate::config;
use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release wire")?;
    registry::validate_site_name(site)?;

    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path).context(super::deploy::registry_load_error())?;

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let shared_dir = release_state::shared_dir(&cfg);
    fs::create_dir_all(&shared_dir)
        .with_context(|| format!("Failed to ensure shared dir exists: {}", shared_dir.display()))?;

    for leaf in paths::SHARED_LEAVES {
        ensure_shared_leaf(&shared_dir, leaf)?;
        link_relative(&release_dir, leaf, &shared_dir.join(leaf))?;
    }

    Ok(())
}

fn ensure_shared_leaf(shared_dir: &Path, leaf: &str) -> Result<()> {
    let path = shared_dir.join(leaf);
    if path.exists() {
        return Ok(());
    }

    if leaf.ends_with(".sqlite") || leaf == paths::DOT_ENV {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create shared leaf parent {}", parent.display()))?;
        }
        fs::write(&path, "").with_context(|| format!("Failed to create shared leaf {}", path.display()))?;
        return Ok(());
    }

    fs::create_dir_all(&path).with_context(|| format!("Failed to create shared leaf {}", path.display()))?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use anyhow::Result;

    use super::{ensure_shared_leaf, link_relative, remove_if_present};

    fn temp_dir(label: &str) -> Result<PathBuf> {
        let dir = std::env::temp_dir().join(format!("bonesremote-wire-{label}-{}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn ensure_shared_leaf_creates_missing_files_and_dirs() -> Result<()> {
        let root = temp_dir("ensure_leaves")?;
        ensure_shared_leaf(&root, "storage")?;
        ensure_shared_leaf(&root, ".env")?;
        ensure_shared_leaf(&root, "database/database.sqlite")?;

        assert!(root.join("storage").is_dir());
        assert!(root.join(".env").is_file());
        assert!(root.join("database/database.sqlite").is_file());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn link_relative_creates_symlink_to_shared_target() -> Result<()> {
        let root = temp_dir("link_relative")?;
        let shared = root.join("shared/.env");
        fs::create_dir_all(shared.parent().unwrap())?;
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
}
