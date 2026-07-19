use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use shared::paths;
use shared::paths::bonesremote_bones_toml_path;

use crate::cli::args::{GuideFormat, SkillCommand};
use crate::config;
use crate::infra::embedded;
use crate::infra::ssh;

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub project: String,
    pub state: String,
    pub state_label: String,
    pub missing: Vec<String>,
    pub commands: Vec<String>,
    pub next: NextCommand,
    #[serde(skip)]
    pub cfg: Option<config::Bones>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NextCommand {
    pub command: String,
    pub mutates: bool,
    pub contacts_remote: bool,
    pub prompt_free_command: String,
}

pub async fn dispatch(command: Option<&SkillCommand>) -> Result<()> {
    match command {
        None => print_orientation(),
        Some(SkillCommand::Next { format }) => run_next(*format).await,
        Some(SkillCommand::List) => {
            list_docs();
            Ok(())
        }
        Some(SkillCommand::Doc { name }) => print_doc(name),
    }
}

pub fn print_orientation() -> Result<()> {
    print!("{}", embedded::skill_orientation()?);
    Ok(())
}

pub fn list_docs() {
    for name in embedded::skill_doc_names() {
        println!("{name}");
    }
}

pub fn print_doc(name: &str) -> Result<()> {
    print!("{}", embedded::skill_doc(name)?);
    Ok(())
}

pub async fn run_next(format: GuideFormat) -> Result<()> {
    let report = build_report().await?;

    match format {
        GuideFormat::Text => print_text(&report),
        GuideFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }

    Ok(())
}

pub async fn build_report() -> Result<Report> {
    let project = config::repo_directory_name()?;
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);

    if !bones_toml.exists() {
        return Ok(uninitialized_report(&project));
    }

    let cfg = config::load(bones_toml).with_context(|| format!("Failed to read {}", bones_toml.display()))?;
    let setup_complete = remote_setup_complete(&cfg).await.context("Unable to determine remote setup status")?;

    if !setup_complete {
        return Ok(initialized_report(cfg, false));
    }

    let ssl_enabled =
        cfg.ssl_enabled || remote_ssl_enabled(&cfg).await.context("Unable to determine remote SSL status")?;

    if ssl_enabled { Ok(ready_report(cfg)) } else { Ok(initialized_report(cfg, true)) }
}

pub(crate) fn prompt_free_init_command(project: &str) -> String {
    format!("bonesdeploy init --non-interactive --project-name {project} --host <host>")
}

fn uninitialized_report(project: &str) -> Report {
    let command = prompt_free_init_command(project);
    Report {
        project: project.to_string(),
        state: String::from("uninitialized"),
        state_label: String::from("not initialized."),
        missing: vec![String::from("init")],
        commands: vec![command.clone()],
        next: next_command(&command, true, false),
        cfg: None,
    }
}

fn initialized_report(cfg: config::Bones, setup_complete: bool) -> Report {
    if setup_complete {
        let command = ssl_command(&cfg);
        let commands = vec![command.clone(), String::from("bonesdeploy deploy")];
        return Report {
            project: cfg.project_name.clone(),
            state: String::from("setup_complete_ssl_missing"),
            state_label: String::from("setup complete, HTTPS missing."),
            missing: vec![String::from("ssl")],
            commands,
            next: next_command(&command, true, true),
            cfg: Some(cfg),
        };
    }

    let command = String::from("bonesdeploy setup --yes");
    let commands = vec![command.clone(), ssl_command(&cfg), String::from("bonesdeploy deploy")];
    Report {
        project: cfg.project_name.clone(),
        state: String::from("initialized_setup_missing"),
        state_label: String::from("initialized, setup not complete."),
        missing: vec![
            String::from("remote_bootstrap"),
            String::from("runtime"),
            String::from("bones_sync"),
            String::from("doctor_pass"),
        ],
        commands,
        next: next_command(&command, true, true),
        cfg: Some(cfg),
    }
}

fn ready_report(cfg: config::Bones) -> Report {
    let command = String::from("bonesdeploy deploy");

    Report {
        project: cfg.project_name.clone(),
        state: String::from("ready"),
        state_label: String::from("ready."),
        missing: Vec::new(),
        commands: vec![command.clone()],
        next: next_command(&command, true, true),
        cfg: Some(cfg),
    }
}

fn next_command(command: &str, mutates: bool, contacts_remote: bool) -> NextCommand {
    NextCommand { command: command.to_string(), mutates, contacts_remote, prompt_free_command: command.to_string() }
}

fn ssl_command(cfg: &config::Bones) -> String {
    let domain = if cfg.domain.is_empty() { String::from("<domain>") } else { cfg.domain.clone() };
    let email = if cfg.email.is_empty() { String::from("<email>") } else { cfg.email.clone() };
    format!("bonesdeploy remote ssl --yes --domain {domain} --email {email}")
}

fn print_text(report: &Report) {
    println!("Project: {}", report.project);
    println!("State: {}", report.state_label);
    println!();

    for (index, command) in report.commands.iter().enumerate() {
        if index == 0 {
            println!("Next: {command}");
        } else {
            println!("Then: {command}");
        }
    }
}

async fn remote_setup_complete(cfg: &config::Bones) -> Result<bool> {
    let session = ssh::connect_privileged(cfg).await?;

    if ssh::run_cmd(&session, "command -v bonesremote >/dev/null 2>&1").await.is_err() {
        session.close().await?;
        return Ok(false);
    }

    let registry_path = bonesremote_bones_toml_path(&cfg.project_name);
    let sync_ok =
        ssh::run_cmd(&session, &format!("test -r {}", ssh::shell_quote(&registry_path.display().to_string())))
            .await
            .is_ok();

    let current = Path::new(&cfg.project_root).join(paths::CURRENT_LINK);
    let current_ok =
        ssh::run_cmd(&session, &format!("test -e {}", ssh::shell_quote(&current.display().to_string()))).await.is_ok();

    session.close().await?;

    Ok(sync_ok && current_ok)
}

pub(crate) async fn remote_ssl_enabled(cfg: &config::Bones) -> Result<bool> {
    if cfg.domain.is_empty() {
        return Ok(false);
    }

    let session = ssh::connect_privileged(cfg).await?;
    let nginx_site_available =
        Path::new(paths::ETC_NGINX_SITES_AVAILABLE).join(format!("{}.conf", cfg.project_name)).display().to_string();
    let command = format!(
        "test -r {path} && grep -Fq {domain} {path} && grep -Fq 'listen 443 ssl;' {path}",
        path = ssh::shell_quote(&nginx_site_available),
        domain = ssh::shell_quote(&format!("server_name {};", cfg.domain)),
    );
    let enabled = ssh::run_cmd(&session, &command).await.is_ok();
    session.close().await?;

    Ok(enabled)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use clap::Parser;

    use crate::cli::args::Cli;
    use crate::commands::skill::prompt_free_init_command;

    /// The guide's prompt-free init command must stay valid against the real CLI.
    /// If the flag drifts again, copy-pasting the suggestion breaks for new users.
    #[test]
    fn prompt_free_init_command_parses_with_cli() -> Result<()> {
        let command = prompt_free_init_command("atlas");
        let mut parts = command.split_whitespace();
        assert_eq!(parts.next(), Some("bonesdeploy"));
        let argv: Vec<&str> = parts.collect();
        let parsed = Cli::try_parse_from(["bonesdeploy"].into_iter().chain(argv.iter().copied()))
            .map_err(|err| anyhow::anyhow!("guide init command should parse, got: {err}"))?;
        // Force-use parsed to confirm it is the expected variant.
        assert!(matches!(parsed.command, crate::cli::args::Command::Init { .. }));
        Ok(())
    }

    #[test]
    fn prompt_free_init_command_uses_non_interactive_flag() {
        let command = prompt_free_init_command("atlas");
        assert!(command.contains("--non-interactive"), "expected --non-interactive, got: {command}");
        assert!(!command.contains("--yes"), "stale --yes flag leaked into guide: {command}");
    }
}
