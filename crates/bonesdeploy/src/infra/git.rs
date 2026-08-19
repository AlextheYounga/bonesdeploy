use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

#[derive(Debug, Clone)]
pub struct RemoteConnectionDetails {
    pub host: String,
    pub port: String,
    pub repo_path: String,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

pub fn ensure_git_repository() -> Result<()> {
    let output =
        Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    Ok(())
}

pub fn remote_exists(remote_name: &str) -> Result<bool> {
    remote_exists_at(Path::new("."), remote_name)
}

pub fn add_remote(remote_name: &str, remote_url: &str) -> Result<()> {
    add_remote_at(Path::new("."), remote_name, remote_url)
}

pub fn remote_exists_at(repo: &Path, remote_name: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["remote", "get-url", remote_name])
        .output()
        .context("Failed to run git")?;
    Ok(output.status.success())
}

pub fn add_remote_at(repo: &Path, remote_name: &str, remote_url: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["remote", "add", remote_name, remote_url])
        .status()
        .with_context(|| format!("Failed to add git remote '{remote_name}'"))?;

    if !status.success() {
        bail!("Failed to add git remote '{remote_name}'");
    }

    Ok(())
}

pub fn branch_exists(branch: &str) -> Result<bool> {
    branch_exists_at(Path::new("."), branch)
}

pub fn branch_exists_at(repo: &Path, branch: &str) -> Result<bool> {
    let ref_name = paths::branch_ref(branch);
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "--verify", &ref_name])
        .output()
        .context("Failed to run git")?;
    Ok(output.status.success())
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

pub fn list_remotes_with_urls() -> Result<Vec<RemoteInfo>> {
    let output = Command::new("git").args(["remote", "-v"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Failed to list git remotes");
    }

    let mut remotes = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(url) = parts.next() else {
            continue;
        };
        let Some(kind) = parts.next() else {
            continue;
        };
        if kind != "(fetch)" {
            continue;
        }
        remotes.push(RemoteInfo { name: name.to_string(), url: url.to_string() });
    }

    Ok(remotes)
}

pub fn remote_url(remote_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", remote_name])
        .output()
        .with_context(|| format!("Failed to read URL for remote '{remote_name}'"))?;

    if !output.status.success() {
        bail!("Failed to read URL for remote '{remote_name}'");
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        bail!("Git remote '{remote_name}' has an empty URL");
    }

    Ok(url)
}

pub fn infer_remote_connection_details(remote_name: &str) -> Result<Option<RemoteConnectionDetails>> {
    let url = remote_url(remote_name)?;
    Ok(parse_remote_url(&url))
}

pub fn parse_remote_url(url: &str) -> Option<RemoteConnectionDetails> {
    parse_ssh_style_url(url.trim()).or_else(|| parse_scp_style_url(url.trim()))
}

fn parse_ssh_style_url(url: &str) -> Option<RemoteConnectionDetails> {
    if !url.starts_with("ssh://") {
        return None;
    }

    let rest = &url[6..];
    let slash_idx = rest.find('/')?;
    let authority = &rest[..slash_idx];
    let path = rest[slash_idx..].trim();

    let (_, host_port) = authority.rsplit_once('@').unwrap_or(("", authority));
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, "22"));

    if host.is_empty() || Path::new(path).extension().is_none_or(|ext| !ext.eq_ignore_ascii_case("git")) {
        return None;
    }

    Some(RemoteConnectionDetails { host: host.to_string(), port: port.to_string(), repo_path: path.to_string() })
}

fn parse_scp_style_url(url: &str) -> Option<RemoteConnectionDetails> {
    if url.contains("://") {
        return None;
    }

    let (left, right) = url.split_once(':')?;
    let right = right.trim();
    if !right.starts_with('/') {
        return None;
    }

    let (_, host) = left.trim().rsplit_once('@').unwrap_or(("", left.trim()));

    if host.is_empty() || Path::new(right).extension().is_none_or(|ext| !ext.eq_ignore_ascii_case("git")) {
        return None;
    }

    Some(RemoteConnectionDetails { host: host.to_string(), port: "22".to_string(), repo_path: right.to_string() })
}
