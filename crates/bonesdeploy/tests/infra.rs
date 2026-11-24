use bonesdeploy::infra::{artifact_deploy_command, remove_site_command, sync_control_plane_command};
use bonesdeploy::infra::{git, ssh};

#[test]
fn artifact_deploy_command_uses_the_current_remote_cli_form() {
    assert_eq!(artifact_deploy_command("demo"), "sudo -n bonesremote deploy --site 'demo'");
}

#[test]
fn sync_command_is_the_other_sudoed_deploy_command() {
    assert_eq!(sync_control_plane_command("demo"), "sudo -n bonesremote config sync --site 'demo'");
}

#[test]
fn remove_site_command_runs_a_sudoed_remote_cleanup() {
    assert_eq!(remove_site_command("demo"), "sudo -n bonesremote remove-site --site 'demo'");
}

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
