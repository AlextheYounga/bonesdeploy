use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::privileges;

fn resolved_tmp_root(site: &str) -> PathBuf {
    PathBuf::from(paths::default_project_root_for(site)).join(paths::TMP_BUILDS_DIR)
}

pub fn run(snapshot: &super::DeploymentSnapshot, context_dir: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release checkout")?;

    let repo_path = snapshot.repo_path.to_str().context("Application repository path is not valid UTF-8")?;
    let archive_output = Command::new("git")
        .args(["--git-dir", repo_path, "archive", "--format=tar", &snapshot.revision])
        .output()
        .with_context(|| {
            format!("Failed to run git archive for revision {} in {}", snapshot.revision, snapshot.repo_path.display())
        })?;
    let git_stderr = String::from_utf8_lossy(&archive_output.stderr).into_owned();

    if !archive_output.status.success() {
        bail!(
            "Failed to export source revision '{}' from {}\n{git_stderr}",
            snapshot.revision,
            snapshot.repo_path.display()
        );
    }

    if !git_stderr.is_empty() {
        println!("[bonesdeploy] git archive reported: {git_stderr}");
    }

    let mut archive = archive_output.stdout.as_slice();

    let mut tar = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(&context_dir)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start tar extraction into {}", context_dir.display()))?;

    let mut tar_stdin = tar.stdin.take().context("tar stdin was not piped")?;
    io::copy(&mut archive, &mut tar_stdin).context("Failed to stream git archive into tar")?;
    drop(tar_stdin);

    let tar_output = tar.wait_with_output().context("Failed to finish tar extraction")?;

    if !tar_output.status.success() {
        bail!(
            "Failed to extract source archive into build context {}\n{}",
            context_dir.display(),
            String::from_utf8_lossy(&tar_output.stderr)
        );
    }

    let deployment_dir = context_dir.join(paths::LOCAL_INFRA_DIR).join(paths::DEPLOYMENT_DIR);
    if deployment_dir.is_dir() {
        println!("Using deployment files from {}", deployment_dir.display());
    }
    println!("Exported source for {} into {}", snapshot.revision, context_dir.display());
    Ok(())
}

/// Resolves a revision (branch name or commit-ish) to the full commit hash in
/// the site's bare repo. The resolved hash feeds the release identity so every
/// release records exactly which commit was exported.
pub(crate) fn resolve_revision_commit(repo_path: &Path, revision: &str) -> Result<String> {
    let repo_path_string = repo_path.to_str().context("Application repository path is not valid UTF-8")?;
    let output = Command::new("git")
        .args(["--git-dir", repo_path_string, "rev-parse", "--verify", &format!("{revision}^{{commit}}")])
        .output()
        .with_context(|| format!("Failed to resolve revision {revision} in {}", repo_path.display()))?;
    if !output.status.success() {
        bail!(
            "Failed to resolve source revision '{revision}' to a commit in {}\n{}",
            repo_path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.len() < 8 {
        bail!("Resolved revision '{revision}' did not yield a valid commit hash");
    }
    Ok(sha)
}

pub(crate) fn repository_has_refs(repo_path: &Path) -> Result<bool> {
    let repo_path = repo_path.to_str().context("Bare repository path is not valid UTF-8")?;
    let output = Command::new("git")
        .args(["--git-dir", repo_path, "for-each-ref", "--format=%(refname)"])
        .output()
        .with_context(|| format!("Failed to inspect refs in {repo_path}"))?;
    if !output.status.success() {
        bail!("Failed to inspect refs in {repo_path}\n{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(!output.stdout.is_empty())
}

pub(crate) fn branch_exists(repo_path: &Path, branch: &str) -> Result<bool> {
    let repo_path = repo_path.to_str().context("Bare repository path is not valid UTF-8")?;
    let ref_name = format!("refs/heads/{branch}");
    let output = Command::new("git")
        .args(["--git-dir", repo_path, "rev-parse", "--verify", &ref_name])
        .output()
        .with_context(|| format!("Failed to inspect branch {branch} in {repo_path}"))?;
    Ok(output.status.success())
}

pub fn ensure_build_context(snapshot: &super::DeploymentSnapshot) -> Result<PathBuf> {
    let root = resolved_tmp_root(&snapshot.site);
    fs::create_dir_all(&root).with_context(|| format!("Failed to create tmp builds root: {}", root.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let context = root.join(format!("build-{}-{nanos}", snapshot.site));
    fs::create_dir_all(&context).with_context(|| format!("Failed to create build context {}", context.display()))?;
    Ok(context)
}

pub fn cleanup_build_context(site: &str, context: &Path) -> Result<()> {
    if context.exists() {
        fs::remove_dir_all(context).with_context(|| format!("Failed to remove build context {}", context.display()))?;
    }
    let root = if let Some(parent) = context.parent() { parent.to_path_buf() } else { resolved_tmp_root(site) };
    if root.exists() && fs::read_dir(&root)?.next().is_none() {
        fs::remove_dir(&root).ok();
    }
    Ok(())
}
