use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::SiteMutation;

pub fn run(mutation: &SiteMutation, _snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release wire")?;

    let release_name = mutation.required_staged_release()?;
    let release_dir = mutation.release_dir(&release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let shared_dir = mutation.shared_dir();
    if !shared_dir.is_dir() {
        bail!(
            "Shared root is missing: {}. Run 'bonesdeploy remote setup' or runtime provisioning first.",
            shared_dir.display()
        );
    }

    let shared_env = shared_dir.join(paths::DOT_ENV);
    if !shared_env.is_file() {
        bail!(
            "Shared environment file is missing: {}. Run 'bonesdeploy remote setup' or secrets provisioning first.",
            shared_env.display()
        );
    }

    link_relative(&release_dir, paths::DOT_ENV, &shared_env)?;

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
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process;

    use anyhow::Result;

    use bonesdeploy_core::paths;

    use super::{link_relative, remove_if_present};

    fn temp_dir(label: &str) -> Result<PathBuf> {
        let dir = env::temp_dir().join(format!("bonesremote-wire-{label}-{}", process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
        Ok(dir)
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
        link_relative(&release, paths::DOT_ENV, &shared)?;

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
