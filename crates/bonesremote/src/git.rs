use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

pub(crate) fn archive(repo_path: &Path, revision: &str) -> Result<Vec<u8>> {
    let repo_path = repo_path.to_str().context("Application repository path is not valid UTF-8")?;
    let output = Command::new("git")
        .args(["--git-dir", repo_path, "archive", "--format=tar", revision])
        .output()
        .with_context(|| format!("Failed to run git archive for revision {revision} in {repo_path}"))?;
    if !output.status.success() {
        bail!(
            "Failed to export source revision '{revision}' from {repo_path}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if !output.stderr.is_empty() {
        println!("[bonesdeploy] git archive reported: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(output.stdout)
}

pub(crate) fn resolve_revision_commit(repo_path: &Path, revision: &str) -> Result<String> {
    let repo_path = repo_path.to_str().context("Application repository path is not valid UTF-8")?;
    let output = Command::new("git")
        .args(["--git-dir", repo_path, "rev-parse", "--verify", &format!("{revision}^{{commit}}")])
        .output()
        .with_context(|| format!("Failed to resolve revision {revision} in {repo_path}"))?;
    if !output.status.success() {
        bail!(
            "Failed to resolve source revision '{revision}' to a commit in {repo_path}\n{}",
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
