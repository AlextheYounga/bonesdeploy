use anyhow::Result;
use bonesdeploy::infra::{git, server_request, ssh};
use bonesdeploy_core::config::Bones;

#[test]
fn branch_validation_uses_git_reference_rules() {
    assert!(git::validate_branch("release/production").is_ok());
    assert!(git::validate_branch("release..production").is_err());
    assert!(git::validate_branch("-production").is_err());
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
    Ok(())
}

#[test]
fn shell_quote_preserves_single_quotes() {
    assert_eq!(ssh::shell_quote("site's"), "'site'\\''s'");
}
