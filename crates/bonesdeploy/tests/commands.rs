use std::path::Path;

use anyhow::Result;
use bonesdeploy::cli::args::{Cli, Command, ServerCommand, SiteCommand, TunnelCommand};
use bonesdeploy::commands::{secrets, site, skill, update};
use bonesdeploy::frameworks::Framework;
use bonesdeploy::infra::{shared_import_command, shared_install_environment_command};
use clap::Parser;

#[test]
fn secrets_framework_rejects_invalid_and_accepts_blank_templates() {
    let result = secrets::framework_for_secrets("not-a-framework");
    assert!(result.as_ref().is_err());
    if let Err(error) = result {
        assert!(error.to_string().contains("Invalid TEMPLATE value"));
    }
    assert!(secrets::framework_for_secrets("  ").is_ok_and(|framework| framework == Framework::Custom));
}

#[test]
fn gpg_fingerprint_parser_preserves_machine_output_behavior() {
    let fingerprint = "ABCDEF1234567890ABCDEF1234567890ABCDEF";
    let output =
        format!("tru::1:1754651437:0:3:1:3\nfpr:::::::::{fingerprint}:\nuid:::::::::Test <test@example.com>:\n");
    assert_eq!(secrets::gpg::extract_fingerprint(&output).as_deref(), Some(fingerprint));
    assert_eq!(
        secrets::gpg::extract_fingerprint("tru::1:1754651437:0:3:1:3\nuid:::::::::Test <test@example.com>:\n"),
        None
    );
}

#[test]
fn verbose_remote_report_ignores_removed_repository_readiness() {
    assert!(!site::render_remote_doctor_output(
        "bonesremote doctor\n  • deploy branch 'main' has not been pushed yet\n",
        true
    ));
    assert!(!site::render_remote_doctor_output(
        "bonesremote doctor\n  • repository has no refs yet. Run 'git push <remote> <branch>' before the first deploy.\n",
        true
    ));
    assert!(site::render_remote_doctor_output(
        "bonesremote doctor\n  • shared environment is missing: /srv/sites/demo/shared/.env. Run 'bonesdeploy secrets push' first.\n",
        false
    ));
}

#[test]
fn prompt_free_init_command_parses_with_cli() -> Result<()> {
    let command = skill::prompt_free_init_command("atlas");
    let mut parts = command.split_whitespace();
    assert_eq!(parts.next(), Some("bonesdeploy"));
    let argv: Vec<&str> = parts.collect();
    let parsed = Cli::try_parse_from(["bonesdeploy"].into_iter().chain(argv.iter().copied()))
        .map_err(|error| anyhow::anyhow!("guide init command should parse, got: {error}"))?;
    assert!(matches!(parsed.command, Command::Init { .. }));
    Ok(())
}

#[test]
fn removed_guide_command_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "guide", "--format", "json"]).is_err());
}

#[test]
fn removed_remote_command_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "remote", "bootstrap"]).is_err());
}

#[test]
fn removed_build_mode_option_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "init", "--build-mode", "local"]).is_err());
}

#[test]
fn removed_deployment_remote_option_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "init", "--remote", "production"]).is_err());
}

#[test]
fn site_export_accepts_an_optional_output_path() -> Result<()> {
    let parsed = Cli::try_parse_from(["bonesdeploy", "site", "export", "--output", "exports/atlas.zip"])?;
    assert!(matches!(
        parsed.command,
        Command::Site {
            command: SiteCommand::Export { output: Some(path) }
        } if path == Path::new("exports/atlas.zip")
    ));

    let parsed = Cli::try_parse_from(["bonesdeploy", "site", "export"])?;
    assert!(matches!(parsed.command, Command::Site { command: SiteCommand::Export { output: None } }));
    Ok(())
}

#[test]
fn site_import_accepts_archive_and_confirmation_flag() -> Result<()> {
    let parsed = Cli::try_parse_from(["bonesdeploy", "site", "import", "shared.zip", "--yes"])?;
    assert!(matches!(
        parsed.command,
        Command::Site { command: SiteCommand::Import { archive, yes: true } }
            if archive == Path::new("shared.zip")
    ));
    Ok(())
}

#[test]
fn shared_operation_commands_are_fixed_and_site_quoted() {
    assert_eq!(shared_import_command("a site's"), "sudo -n bonesremote shared import --site 'a site'\\''s'");
    assert_eq!(
        shared_install_environment_command("demo"),
        "sudo -n bonesremote shared install-environment --site 'demo'"
    );
}

#[test]
fn removed_flat_status_command_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "status"]).is_err());
}

#[test]
fn removed_flat_manifest_command_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "manifest"]).is_err());
}

#[test]
fn removed_flat_releases_command_is_rejected() {
    assert!(Cli::try_parse_from(["bonesdeploy", "releases"]).is_err());
}

#[test]
fn server_and_site_commands_parse_under_their_scopes() -> Result<()> {
    let server = Cli::try_parse_from(["bonesdeploy", "server", "setup", "--yes"])?;
    assert!(matches!(server.command, Command::Server { command: ServerCommand::Setup { yes: true } }));

    let site = Cli::try_parse_from(["bonesdeploy", "site", "doctor", "--local"])?;
    assert!(matches!(site.command, Command::Site { command: SiteCommand::Doctor { local: true, .. } }));

    let delete = Cli::try_parse_from(["bonesdeploy", "site", "delete", "--yes"])?;
    assert!(matches!(delete.command, Command::Site { command: SiteCommand::Delete { yes: true } }));
    Ok(())
}

#[test]
fn quick_tunnel_commands_parse_under_the_site_scope() -> Result<()> {
    let start = Cli::try_parse_from(["bonesdeploy", "site", "tunnel", "start", "--yes"])?;
    assert!(matches!(
        start.command,
        Command::Site { command: SiteCommand::Tunnel { command: TunnelCommand::Start { yes: true } } }
    ));

    let stop = Cli::try_parse_from(["bonesdeploy", "site", "tunnel", "stop", "--yes"])?;
    assert!(matches!(
        stop.command,
        Command::Site { command: SiteCommand::Tunnel { command: TunnelCommand::Stop { yes: true } } }
    ));

    let status = Cli::try_parse_from(["bonesdeploy", "site", "tunnel", "status"])?;
    assert!(matches!(
        status.command,
        Command::Site { command: SiteCommand::Tunnel { command: TunnelCommand::Status } }
    ));
    Ok(())
}

#[test]
fn release_tags_accept_semver_and_reject_unexpected_values() -> Result<()> {
    assert_eq!(update::parse_release_tag(Some("v0.7.3"))?, "0.7.3");
    assert_eq!(update::parse_release_tag(Some("v0.7.3-rc.1+build"))?, "0.7.3-rc.1+build");
    assert!(update::parse_release_tag(Some("0.7.3")).is_err());
    assert!(update::parse_release_tag(Some("v0.7.3/tag")).is_err());
    assert!(update::parse_release_tag(None).is_err());
    Ok(())
}

#[test]
fn remote_update_downloads_versioned_release_and_checksum() {
    let command = update::release::bonesremote_download_command("0.7.3", "/usr/local/bin");
    assert!(command.contains("releases/download/v0.7.3"));
    assert!(command.contains("bonesremote-x86_64-unknown-linux-musl.sha256"));
    assert!(command.contains("sha256sum --check"));
    assert!(command.contains("uname -m"));
    assert!(command.contains("bonesremote 0.7.3"));
    assert!(command.contains("install -o root -g root -m 0755"));
    assert!(command.contains("'/usr/local/bin/bonesremote.tmp'"));
    assert!(command.contains("'/usr/local/bin/bonesremote'"));
}
