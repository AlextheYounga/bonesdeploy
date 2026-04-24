use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct RemoteConnectionDetails {
    pub host: String,
    pub port: String,
    pub git_dir: String,
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
    let output = Command::new("git").args(["remote", "get-url", remote_name]).output().context("Failed to run git")?;
    Ok(output.status.success())
}

pub fn add_remote(remote_name: &str, remote_url: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["remote", "add", remote_name, remote_url])
        .status()
        .with_context(|| format!("Failed to add git remote '{remote_name}'"))?;

    if !status.success() {
        bail!("Failed to add git remote '{remote_name}'");
    }

    Ok(())
}

pub fn list_remotes() -> Result<Vec<String>> {
    let output = Command::new("git").args(["remote"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Failed to list git remotes");
    }

    let remotes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

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

fn parse_remote_url(url: &str) -> Option<RemoteConnectionDetails> {
    parse_ssh_style_url(url).or_else(|| parse_scp_style_url(url))
}

fn parse_ssh_style_url(url: &str) -> Option<RemoteConnectionDetails> {
    if !url.starts_with("ssh://") {
        return None;
    }

    let without_scheme = &url[6..];
    let slash_index = without_scheme.find('/')?;
    let authority = &without_scheme[..slash_index];
    let path = &without_scheme[slash_index..];

    let host_port = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) if !host.is_empty() && !port.is_empty() => (host.to_string(), port.to_string()),
        _ => (host_port.to_string(), String::from("22")),
    };

    if host.is_empty() || !has_git_extension(path) {
        return None;
    }

    Some(RemoteConnectionDetails { host, port, git_dir: path.to_string() })
}

fn parse_scp_style_url(url: &str) -> Option<RemoteConnectionDetails> {
    if url.contains("://") {
        return None;
    }

    let (left, right) = url.split_once(':')?;
    let host = left.rsplit_once('@').map_or(left, |(_, host)| host);
    if !right.starts_with('/') {
        return None;
    }
    let git_dir = right.to_string();

    if host.is_empty() || !has_git_extension(&git_dir) {
        return None;
    }

    Some(RemoteConnectionDetails { host: host.to_string(), port: String::from("22"), git_dir })
}

fn has_git_extension(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("git"))
}
