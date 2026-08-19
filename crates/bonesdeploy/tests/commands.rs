use anyhow::Result;
use bonesdeploy::cli::args::{Cli, Command};
use bonesdeploy::commands::{doctor, secrets, skill, update};
use bonesdeploy::frameworks::Framework;
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
fn verbose_remote_report_preserves_pending_state() {
    assert!(doctor::render_remote_doctor_output(
        "bonesremote doctor\n  • deploy branch 'main' has not been pushed yet\n",
        true
    ));
    assert!(!doctor::render_remote_doctor_output("bonesremote doctor\n✓ All checks passed.\n", true));
}

#[test]
fn strip_ansi_removes_sgr_color_sequences() {
    assert_eq!(
        doctor::strip_ansi("\x1b[1;33m•\x1b[0m deploy branch 'master' has not been pushed yet"),
        "• deploy branch 'master' has not been pushed yet"
    );
    assert_eq!(doctor::strip_ansi("plain text"), "plain text");
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
fn guide_compatibility_command_still_parses() -> Result<()> {
    let parsed = Cli::try_parse_from(["bonesdeploy", "guide", "--format", "json"])?;
    assert!(matches!(parsed.command, Command::Guide { .. }));
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
