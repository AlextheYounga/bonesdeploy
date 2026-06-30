use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

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

fn parse_remote_url(url: &str) -> Option<RemoteConnectionDetails> {
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

    let (user, host_port) = authority.rsplit_once('@').unwrap_or(("", authority));
    let _user = if user.is_empty() { "git" } else { user };
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

    let (user, host) = left.trim().rsplit_once('@').unwrap_or(("", left.trim()));
    let _user = if user.is_empty() { "git" } else { user };

    if host.is_empty() || Path::new(right).extension().is_none_or(|ext| !ext.eq_ignore_ascii_case("git")) {
        return None;
    }

    Some(RemoteConnectionDetails { host: host.to_string(), port: "22".to_string(), repo_path: right.to_string() })
}

#[cfg(test)]
mod tests {
    use super::{parse_remote_url, parse_scp_style_url, parse_ssh_style_url};

    #[test]
    fn parse_ssh_style_url_parses_host_port_and_repo_path() {
        let details = parse_ssh_style_url("ssh://git@example.com:2222/home/git/myapp.git");
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "2222");
            assert_eq!(details.repo_path, "/home/git/myapp.git");
        }
    }

    #[test]
    fn parse_ssh_style_url_defaults_port_to_22() {
        let details = parse_ssh_style_url("ssh://git@example.com/home/git/myapp.git");
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, "/home/git/myapp.git");
        }
    }

    #[test]
    fn parse_scp_style_url_parses_absolute_repo_path() {
        let details = parse_scp_style_url("git@example.com:/home/git/myapp.git");
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, "/home/git/myapp.git");
        }
    }

    #[test]
    fn parse_scp_style_url_trims_surrounding_whitespace() {
        let details = parse_scp_style_url("git@example.com : /home/git/myapp.git");
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, "/home/git/myapp.git");
        }
    }

    #[test]
    fn parse_remote_url_rejects_non_git_paths() {
        assert!(parse_remote_url("ssh://git@example.com:22/home/git/myapp").is_none());
        assert!(parse_remote_url("git@example.com:/home/git/myapp").is_none());
    }

    #[test]
    fn parse_remote_url_rejects_relative_scp_paths() {
        assert!(parse_remote_url("git@example.com:myapp.git").is_none());
    }

    #[test]
    fn parse_remote_url_rejects_non_ssh_urls() {
        assert!(parse_remote_url("https://example.com/org/repo.git").is_none());
    }
}
