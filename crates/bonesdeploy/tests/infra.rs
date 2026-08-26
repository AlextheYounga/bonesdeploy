use std::path::Path;

use anyhow::Result;
use bonesdeploy::infra::{git, server_request, ssh};
use bonesdeploy_core::config::Bones;

#[test]
fn branch_exists_reports_missing_branch_without_error() {
    assert_eq!(git::branch_exists_at(Path::new("/path/that/does/not/exist"), "main").ok(), Some(false));
}

#[test]
fn server_request_contains_only_connection_fields() -> Result<()> {
    let mut config = Bones::default();
    config.host = "example.com".into();
    config.ssh_user = "deploy".into();
    config.port = "2222".into();

    let request: serde_json::Value = serde_json::from_str(&server_request(&config)?)?;

    assert_eq!(request["server"]["host"], "example.com");
    assert_eq!(request["server"]["ssh_user"], "deploy");
    assert_eq!(request["server"]["port"], "2222");
    assert!(request.get("site").is_none());
    assert!(request.get("services").is_none());
    Ok(())
}

#[test]
fn ssh_style_remote_urls_parse_host_port_and_path() {
    let details = git::parse_remote_url("ssh://git@example.com:2222/home/git/myapp.git");
    assert!(details.is_some());
    if let Some(details) = details {
        assert_eq!(details.host, "example.com");
        assert_eq!(details.port, "2222");
        assert_eq!(details.repo_path, "/home/git/myapp.git");
    }

    let details = git::parse_remote_url("ssh://git@example.com/home/git/myapp.git");
    assert!(details.is_some());
    if let Some(details) = details {
        assert_eq!(details.host, "example.com");
        assert_eq!(details.port, "22");
        assert_eq!(details.repo_path, "/home/git/myapp.git");
    }
}

#[test]
fn scp_style_remote_urls_parse_absolute_paths_and_whitespace() {
    for url in ["git@example.com:/home/git/myapp.git", "git@example.com : /home/git/myapp.git"] {
        let details = git::parse_remote_url(url);
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, "/home/git/myapp.git");
        }
    }
}

#[test]
fn remote_url_parser_rejects_unsupported_urls() {
    assert!(git::parse_remote_url("ssh://git@example.com:22/home/git/myapp").is_none());
    assert!(git::parse_remote_url("git@example.com:/home/git/myapp").is_none());
    assert!(git::parse_remote_url("git@example.com:myapp.git").is_none());
    assert!(git::parse_remote_url("https://example.com/org/repo.git").is_none());
}

#[test]
fn shell_quote_preserves_single_quotes() {
    assert_eq!(ssh::shell_quote("site's"), "'site'\\''s'");
}

#[test]
fn remote_command_failure_includes_stdout_and_stderr() {
    let message = ssh::remote_command_failure(
        "bonesremote doctor --site demo",
        b"issue one\nissue two\n",
        b"Doctor found 2 issues\n",
    );
    assert!(message.contains("Remote command failed: bonesremote doctor --site demo"));
    assert!(message.contains("stdout:\nissue one\nissue two"));
    assert!(message.contains("stderr:\nDoctor found 2 issues"));
}
