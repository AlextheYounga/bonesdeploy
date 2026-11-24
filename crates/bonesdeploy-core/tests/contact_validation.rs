use bonesdeploy_core::config::{validate_domain, validate_email};

#[test]
fn domain_validation_accepts_dns_names_and_rejects_shell_input() {
    assert!(validate_domain("app.example.com").is_ok());
    assert!(validate_domain("example.com;touch-/tmp/bad").is_err());
    assert!(validate_domain("-bad.example.com").is_err());
    assert!(validate_domain("localhost").is_err());
}

#[test]
fn email_validation_accepts_certbot_addresses_and_rejects_shell_input() {
    assert!(validate_email("ops+deploy@example.com").is_ok());
    assert!(validate_email("ops@example.com --debug").is_err());
    assert!(validate_email("missing-domain@example").is_err());
    assert!(validate_email("two@example.com@invalid.test").is_err());
}
