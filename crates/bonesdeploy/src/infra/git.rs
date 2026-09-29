use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

pub fn ensure_git_repository() -> Result<()> {
    let output =
        Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    Ok(())
}

pub fn validate_branch(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["check-ref-format", "--branch", branch])
        .output()
        .context("Failed to validate Git branch")?;
    if !output.status.success() {
        bail!("Invalid Git branch: {branch}");
    }
    Ok(())
}

/// Resolves the configured local branch to an immutable full object ID.
pub fn resolve_branch_commit(repo: &Path, branch: &str) -> Result<String> {
    let reference = paths::branch_ref(branch);
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--verify", "--end-of-options", &format!("{reference}^{{commit}}")])
        .output()
        .with_context(|| format!("Failed to resolve local branch '{branch}'"))?;
    if !output.status.success() {
        bail!("Local branch '{branch}' does not resolve to a commit");
    }

    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("Git returned an invalid commit ID for local branch '{branch}'");
    }
    Ok(revision)
}

/// Exports exactly one committed tree with `git archive`; the worktree is never read.
pub fn export_commit(repo: &Path, revision: &str, destination: &Path) -> Result<()> {
    let mut archive = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["archive", "--format=tar", revision])
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to archive commit {revision}"))?;
    let archive_stdout = archive.stdout.take().context("Git archive stdout was not piped")?;
    let extract_status = Command::new("tar")
        .args(["--extract", "--file=-", "--no-same-owner", "--no-same-permissions", "--directory"])
        .arg(destination)
        .stdin(Stdio::from(archive_stdout))
        .status()
        .with_context(|| format!("Failed to extract commit {revision}"))?;
    let archive_status = archive.wait().context("Failed to finish Git archive")?;
    if !archive_status.success() {
        bail!("Failed to archive commit {revision}: {archive_status}");
    }
    if !extract_status.success() {
        bail!("Failed to extract commit {revision}: {extract_status}");
    }
    Ok(())
}

pub fn clone_repository(url: &str, branch: &str, destination: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, url])
        .arg(destination)
        .status()
        .with_context(|| format!("Failed to clone {url}"))?;
    if !status.success() {
        bail!("Failed to clone {url} release tag {branch}");
    }
    Ok(())
}
