use bonesdeploy::infra::{git, ssh};

#[test]
fn branch_validation_uses_git_reference_rules() {
    assert!(git::validate_branch("release/production").is_ok());
    assert!(git::validate_branch("release..production").is_err());
    assert!(git::validate_branch("-production").is_err());
}

#[test]
fn shell_quote_preserves_single_quotes() {
    assert_eq!(ssh::shell_quote("site's"), "'site'\\''s'");
}
