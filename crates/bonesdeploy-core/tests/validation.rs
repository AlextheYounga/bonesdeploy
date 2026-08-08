//! Project and script-name validation for the `bonesdeploy-core` library.

use bonesdeploy_core::config::{is_numbered_shell_script, validate_project_name};

#[test]
fn accepts_site_identifiers() {
    assert!(validate_project_name("nexttest-2").is_ok());
}

#[test]
fn rejects_unit_name_syntax_and_reserved_targets() {
    assert!(validate_project_name("multi-user.target").is_err());
    assert!(validate_project_name("shop_admin").is_err());
    assert!(validate_project_name("reboot").is_err());
    assert!(validate_project_name("multi-user").is_err());
}

#[test]
fn numbered_shell_scripts_require_digits_underscore_and_name() {
    assert!(is_numbered_shell_script("01_build.sh"));
    assert!(!is_numbered_shell_script("999_prepare.sh"));
    assert!(!is_numbered_shell_script("1_prepare.sh"));
    assert!(!is_numbered_shell_script("build.sh"));
    assert!(!is_numbered_shell_script("01build.sh"));
    assert!(!is_numbered_shell_script("01_.sh"));
    assert!(!is_numbered_shell_script("01_build.py"));
}
