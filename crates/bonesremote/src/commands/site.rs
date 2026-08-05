use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::{self, validate_site_name};
use bonesdeploy_core::paths;

use crate::commands::ensure_site_idle;
use crate::privileges;
use crate::release::SiteMutation;

const POST_RECEIVE_SCRIPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/hooks/post-receive"));

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

    if let Err(error) = import_staged_site(site, &staging_dir) {
        if let Err(cleanup_error) = fs::remove_dir_all(&staging_dir) {
            return Err(error).context(format!(
                "Failed to clean up import staging directory {}: {cleanup_error}",
                staging_dir.display()
            ));
        }
        return Err(error);
    }
    println!("Imported site state for {site}.");
    Ok(())
}

/// # Errors
///
/// Returns an error if the revision does not exist or the config dataset is invalid.
pub fn receive(site: &str, revision: &str) -> Result<()> {
    privileges::ensure_root("bonesremote site receive")?;
    validate_site_name(site)?;

    let sites_root = paths::bonesremote_sites_root();
    fs::create_dir_all(&sites_root).with_context(|| format!("Failed to create {}", sites_root.display()))?;

    let staging_dir = unique_site_path(&sites_root, site, "incoming");
    fs::create_dir_all(&staging_dir).with_context(|| format!("Failed to create {}", staging_dir.display()))?;

    if let Err(error) = receive_staged_site(site, revision, &staging_dir) {
        if let Err(cleanup_error) = fs::remove_dir_all(&staging_dir) {
            return Err(error).context(format!(
                "Failed to clean up receive staging directory {}: {cleanup_error}",
                staging_dir.display()
            ));
        }
        return Err(error);
    }
    println!("Imported .bones config for {site}.");
    Ok(())
}

fn import_staged_site(site: &str, staging_dir: &Path) -> Result<()> {
    extract_stdin_archive(staging_dir)?;
    finalize_imported_site(site, staging_dir)
}

fn receive_staged_site(site: &str, revision: &str, staging_dir: &Path) -> Result<()> {
    let bones_repo = paths::default_bones_repo_path_for(site);
    let archive = Command::new("git")
        .args(["--git-dir", &bones_repo, "archive", "--format=tar", revision])
        .output()
        .with_context(|| format!("Failed to archive revision {revision} from {bones_repo}"))?;
    if !archive.status.success() {
        bail!("git archive failed: {}", String::from_utf8_lossy(&archive.stderr));
    }
    let mut child = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(staging_dir)
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to run tar for config receive")?;
    {
        let mut stdin = child.stdin.take().context("tar stdin was not piped")?;
        use std::io::Write as _;
        stdin.write_all(&archive.stdout).context("Failed to write archive to tar")?;
    }
    let status = child.wait().context("tar process failed")?;
    if !status.success() {
        bail!("Failed to extract config dataset");
    }
    finalize_imported_site(site, staging_dir)
}

fn finalize_imported_site(site: &str, staging_dir: &Path) -> Result<()> {
    validate_site_dataset(site, staging_dir)?;

    // Import/receive replaces live site state, so it is serialized with every
    // other mutation and only runs after the site is proven idle.
    let _mutation = SiteMutation::acquire(site)?;
    ensure_site_idle(site)?;
    write_post_receive_hook(staging_dir)?;
    replace_site_dir(site, staging_dir)
}

fn write_post_receive_hook(site_root: &Path) -> Result<()> {
    let cfg = config::load(&site_root.join(paths::BONES_TOML))?;
    validate_repo_path(&cfg.repo_path, &cfg.project_name)?;
    let target = Path::new(&cfg.repo_path).join(paths::HOOKS_DIR).join("post-receive");
    write_hook_file(&target)
}

fn write_hook_file(target: &Path) -> Result<()> {
    let target_parent = target.parent().context("post-receive hook target has no parent")?;

    fs::create_dir_all(target_parent).with_context(|| format!("Failed to create {}", target_parent.display()))?;
    fs::write(target, POST_RECEIVE_SCRIPT).with_context(|| format!("Failed to write {}", target.display()))?;

    let mut perms = fs::metadata(target).with_context(|| format!("Failed to stat {}", target.display()))?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(target, perms).with_context(|| format!("Failed to chmod {}", target.display()))?;

    Ok(())
}

// Confused-deputy guard: the imported bones.toml supplies `repo_path`, and this
// function writes `<repo_path>/hooks/post-receive` as root. Reject anything that
// is not the canonical site repository under the configured parent so an
// imported dataset cannot redirect the hook write at an unintended path.
fn validate_repo_path(repo_path: &str, project_name: &str) -> Result<()> {
    let expected = paths::default_repo_path_for(project_name);
    if repo_path == expected {
        return Ok(());
    }
    bail!(
        "Imported repo_path '{repo_path}' does not match the expected site repository '{expected}'; refusing to write hook outside the configured repository parent"
    );
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
    reject_plaintext_env_files(root)?;
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

fn reject_plaintext_env_files(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("Failed to read {}", root.display()))? {
        let entry = entry?;
        if entry.file_name() == paths::DOT_ENV {
            bail!("Imported dataset contains plaintext .env: {}", entry.path().display());
        }
        if entry.file_type()?.is_dir() {
            reject_plaintext_env_files(&entry.path())?;
        }
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
mod tests;
