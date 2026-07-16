use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use shared::{config, paths};

use crate::privileges;

const POST_RECEIVE_SCRIPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/hooks/post-receive"));

const ALLOWED_TOP_LEVEL_ENTRIES: &[&str] = &[paths::BONES_TOML, paths::DEPLOYMENT_DIR];

fn validate_site_name(site: &str) -> Result<()> {
    if site.is_empty() {
        bail!("Site name cannot be empty");
    }

    if site.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        return Ok(());
    }

    bail!("Invalid site name: {site}")
}

/// # Errors
///
/// Returns an error if the dataset is invalid or the control-plane state cannot
/// be updated safely.
pub fn import(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote site import")?;
    validate_site_name(site)?;

    let sites_root = paths::bonesremote_sites_root();
    fs::create_dir_all(&sites_root).with_context(|| format!("Failed to create {}", sites_root.display()))?;

    let staging_dir = unique_site_path(&sites_root, site, "incoming");
    fs::create_dir_all(&staging_dir).with_context(|| format!("Failed to create {}", staging_dir.display()))?;

    extract_stdin_archive(&staging_dir)?;
    validate_site_dataset(site, &staging_dir)?;
    replace_site_dir(site, &staging_dir)?;
    install_repo_post_receive_hook(site)?;
    println!("Imported site state for {site}.");
    Ok(())
}

fn install_repo_post_receive_hook(site: &str) -> Result<()> {
    let site_root = paths::bonesremote_site_root(site);
    write_post_receive_hook(&site_root)
}

fn write_post_receive_hook(site_root: &Path) -> Result<()> {
    let cfg = config::load(&site_root.join(paths::BONES_TOML))?;
    let target = Path::new(&cfg.repo_path).join(paths::HOOKS_DIR).join("post-receive");
    let target_parent = target.parent().context("post-receive hook target has no parent")?;

    fs::create_dir_all(target_parent).with_context(|| format!("Failed to create {}", target_parent.display()))?;
    fs::write(&target, POST_RECEIVE_SCRIPT).with_context(|| format!("Failed to write {}", target.display()))?;

    let mut perms =
        fs::metadata(&target).with_context(|| format!("Failed to stat {}", target.display()))?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&target, perms).with_context(|| format!("Failed to chmod {}", target.display()))?;

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
            fs::rename(&backup_dir, &site_root)
                .with_context(|| format!("Failed to restore previous site state from {}", backup_dir.display()))?;
        }
        return Err(error).with_context(|| format!("Failed to activate {}", site_root.display()));
    }

    if had_existing {
        fs::remove_dir_all(&backup_dir).with_context(|| format!("Failed to remove {}", backup_dir.display()))?;
    }

    Ok(())
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
    reject_symlinks_recurse(root)?;
    Ok(())
}

fn reject_symlinks_recurse(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_symlink() {
            bail!("Imported dataset cannot contain symlinks: {}", path.display());
        }
        if path.is_dir() {
            reject_symlinks_recurse(&path)?;
        }
    }
    Ok(())
}

fn unique_site_path(parent: &Path, site: &str, suffix: &str) -> PathBuf {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    parent.join(format!(".{site}.{suffix}.{stamp}"))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process;

    use anyhow::Result;

    use super::{validate_top_level_entries, write_post_receive_hook};
    use shared::paths;

    #[test]
    fn validate_top_level_entries_allows_single_config() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-site-buildtime-test-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join(paths::BONES_TOML), "")?;
        fs::create_dir_all(root.join(paths::DEPLOYMENT_DIR))?;

        let result = validate_top_level_entries(&root);
        fs::remove_dir_all(&root)?;
        assert!(result.is_ok());
        Ok(())
    }

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

    #[test]
    fn install_repo_post_receive_hook_writes_baked_trigger() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-site-hook-test-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }

        let repo_root = root.join("repos/unitapp.git");
        let site_root = root.join("sites/unitapp");
        fs::create_dir_all(&site_root)?;
        fs::write(
            site_root.join(paths::BONES_TOML),
            format!(
                r#"
[app]
remote_name = "production"
project_name = "unitapp"
repo_path = "{}"
project_root = "/srv/sites/unitapp"
[app.server]
ssh_user = "root"
host = "example.com"
port = "22"
[app.dns]
preview_domain = ""
[app.deploy]
branch = "main"
deploy_on_push = false
releases = 5
"#,
                repo_root.display()
            ),
        )?;

        let result = write_post_receive_hook(&site_root);

        let target = repo_root.join(paths::HOOKS_DIR).join("post-receive");
        let contents = fs::read_to_string(&target)?;
        let mode = fs::metadata(&target)?.permissions().mode() & 0o777;

        result?;
        assert!(contents.contains("bonesdeploy-post-receive-v1"));
        assert_eq!(mode, 0o755);
        fs::remove_dir_all(&root)?;
        Ok(())
    }
}
