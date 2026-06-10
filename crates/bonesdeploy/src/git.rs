use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use git_url_parse::GitUrl;

#[derive(Debug, Clone)]
pub struct RemoteConnectionDetails {
    pub user: String,
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

pub fn list_remotes_with_urls() -> Result<Vec<RemoteInfo>> {
    let names = list_remotes()?;
    let mut remotes = Vec::with_capacity(names.len());
    for name in names {
        let url = remote_url(&name)?;
        remotes.push(RemoteInfo { name, url });
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
    let git_url = GitUrl::parse(url).ok()?;

    if git_url.scheme() != Some("ssh") {
        return None;
    }

    let host = git_url.host()?.to_string();
    if host.is_empty() {
        return None;
    }

    let raw_path = git_url.path();
    let path = if raw_path.starts_with('/') { raw_path.to_string() } else { format!("/{raw_path}") };

    // Reject relative SCP paths (e.g. git@host:repo.git with no leading / after colon)
    let is_scp = !url.contains("://");
    if is_scp {
        let after_colon = url.split_once(':').map_or("", |(_, right)| right);
        if !after_colon.starts_with('/') {
            return None;
        }
    }

    if !has_git_extension(&path) {
        return None;
    }

    Some(RemoteConnectionDetails {
        user: git_url.user().map_or_else(|| String::from("git"), ToString::to_string),
        host,
        port: git_url.port().map_or_else(|| String::from("22"), |p| p.to_string()),
        repo_path: path,
    })
}

fn has_git_extension(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("git"))
}

#[cfg(test)]
mod tests {
    use shared::paths;

    use super::parse_remote_url;

    fn repo_path(name: &str) -> String {
        paths::default_repo_path_for(name)
    }

    /// Parses the host, port, and repository path from a full SSH-style URL.
    #[test]
    fn parse_ssh_style_url_parses_host_port_and_repo_path() {
        let details = parse_remote_url(&format!("ssh://git@example.com:2222{}", repo_path("myapp")));
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "2222");
            assert_eq!(details.repo_path, repo_path("myapp"));
        }
    }

    /// Defaults the SSH port to 22 when not explicitly provided in the URL.
    #[test]
    fn parse_ssh_style_url_defaults_port_to_22() {
        let details = parse_remote_url(&format!("ssh://git@example.com{}", repo_path("myapp")));
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, repo_path("myapp"));
        }
    }

    /// Parses an absolute repo path from an SCP-style remote URL.
    #[test]
    fn parse_scp_style_url_parses_absolute_repo_path() {
        let details = parse_remote_url(&format!("git@example.com:{}", repo_path("myapp")));
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, repo_path("myapp"));
        }
    }

    /// Rejects repo paths that do not end with `.git`.
    #[test]
    fn parse_remote_url_rejects_non_git_paths() {
        let non_git_path = repo_path("myapp").trim_end_matches(".git").to_string();
        assert!(parse_remote_url(&format!("ssh://git@example.com:22{non_git_path}")).is_none());
        assert!(parse_remote_url(&format!("git@example.com:{non_git_path}")).is_none());
    }

    /// Rejects relative SCP paths that can resolve differently across hosts.
    #[test]
    fn parse_remote_url_rejects_relative_scp_paths() {
        assert!(parse_remote_url("git@example.com:myapp.git").is_none());
    }

    /// Rejects non-SSH URLs that cannot be used with SSH deployment connections.
    #[test]
    fn parse_remote_url_rejects_non_ssh_urls() {
        assert!(parse_remote_url("https://example.com/org/repo.git").is_none());
    }
}
