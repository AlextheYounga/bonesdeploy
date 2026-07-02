use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Component, Path};

use anyhow::{Context, Result, bail};
use shared::config::{self, SharedPath, SharedPathType};
use shared::paths;

use crate::privileges;
use crate::release::state as release_state;

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
    let runtime =
        config::load_runtime(&paths::bonesremote_site_root(site)).context("Failed to load remote runtime state")?;

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

    for shared_path in &runtime.shared.paths {
        validate_shared_path(shared_path)?;
        let target = shared_dir.join(&shared_path.path);
        ensure_shared_leaf(&target, &shared_path.path_type)?;
        link_relative(&release_dir, &shared_path.path, &target)?;
    }

    Ok(())
}

fn validate_shared_path(shared_path: &SharedPath) -> Result<()> {
    let path = Path::new(&shared_path.path);
    if shared_path.path.is_empty() || path.is_absolute() {
        bail!("Invalid shared path in runtime.toml: {}", shared_path.path);
    }

    if !path.components().all(|component| matches!(component, Component::Normal(_))) {
        bail!("Invalid shared path in runtime.toml: {}", shared_path.path);
    }

    Ok(())
}

fn ensure_shared_leaf(path: &Path, path_type: &SharedPathType) -> Result<()> {
    match path_type {
        SharedPathType::File if path.is_file() => return Ok(()),
        SharedPathType::Dir if path.is_dir() => return Ok(()),
        _ => {}
    }

    bail!(
        "Required shared path is missing or has the wrong type: {}. Provision the runtime shared paths before deploying.",
        path.display()
    )
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
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process;

    use anyhow::Result;

    use shared::config::SharedPathType;

    use super::{ensure_shared_leaf, link_relative, remove_if_present, validate_shared_path};

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
        assert!(ensure_shared_leaf(&missing, &SharedPathType::Dir).is_err());

        fs::create_dir_all(&missing)?;
        ensure_shared_leaf(&missing, &SharedPathType::Dir)?;
        assert!(ensure_shared_leaf(&missing, &SharedPathType::File).is_err());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn validate_shared_path_rejects_absolute_and_parent_paths() {
        assert!(validate_shared_path(&shared_path("storage", SharedPathType::Dir)).is_ok());
        assert!(validate_shared_path(&shared_path("/srv/storage", SharedPathType::Dir)).is_err());
        assert!(validate_shared_path(&shared_path("../storage", SharedPathType::Dir)).is_err());
        assert!(validate_shared_path(&shared_path(".", SharedPathType::Dir)).is_err());
        assert!(validate_shared_path(&shared_path("", SharedPathType::Dir)).is_err());
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

    fn shared_path(path: &str, path_type: SharedPathType) -> shared::config::SharedPath {
        shared::config::SharedPath { path: path.to_string(), path_type }
    }
}
