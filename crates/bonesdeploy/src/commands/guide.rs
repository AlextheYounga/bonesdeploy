use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use shared::config as shared_config;
use shared::paths;
use shared::paths::bonesremote_registry_path;

use crate::cli::args::GuideFormat;
use crate::config;
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

pub async fn run(format: GuideFormat) -> Result<()> {
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
    let web_root = runtime_web_root()?;
    let setup_complete = remote_setup_complete(&cfg, &web_root).await.unwrap_or(false);

    if !setup_complete {
        return Ok(initialized_report(cfg, false));
    }

    let ssl_enabled = cfg.ssl_enabled || remote_ssl_enabled(&cfg, &web_root).await.unwrap_or(false);

    if ssl_enabled { Ok(ready_report(cfg)) } else { Ok(initialized_report(cfg, true)) }
}

pub(crate) fn prompt_free_init_command(project: &str) -> String {
    format!("bonesdeploy init --yes --project-name {project} --host <host>")
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
    let (state, state_label, missing, commands) = if setup_complete {
        let command = ssl_command(&cfg);
        (
            String::from("setup_complete_ssl_missing"),
            String::from("setup complete, HTTPS missing."),
            vec![String::from("ssl")],
            vec![command, String::from("bonesdeploy deploy")],
        )
    } else {
        (
            String::from("initialized_setup_missing"),
            String::from("initialized, setup not complete."),
            vec![
                String::from("remote_bootstrap"),
                String::from("runtime"),
                String::from("bones_sync"),
                String::from("doctor_pass"),
            ],
            vec![String::from("bonesdeploy setup --yes"), ssl_command(&cfg), String::from("bonesdeploy deploy")],
        )
    };

    let next = next_command(&commands[0], true, true);

    Report { project: cfg.project_name.clone(), state, state_label, missing, commands, next, cfg: Some(cfg) }
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

fn runtime_web_root() -> Result<String> {
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;
    Ok(runtime.web_root)
}

async fn remote_setup_complete(cfg: &config::Bones, web_root: &str) -> Result<bool> {
    let Ok(session) = ssh::connect_privileged(cfg).await else {
        return Ok(false);
    };

    if ssh::run_cmd(&session, "command -v bonesremote >/dev/null 2>&1").await.is_err() {
        session.close().await?;
        return Ok(false);
    }

    let registry_path = bonesremote_registry_path(&cfg.project_name);
    let sync_ok =
        ssh::run_cmd(&session, &format!("test -r {}", shell_quote(&registry_path.display().to_string()))).await.is_ok();

    let paths = cfg.deployment_paths(web_root);
    let current_ok = ssh::run_cmd(&session, &format!("test -e {}", shell_quote(&paths.current))).await.is_ok();

    session.close().await?;

    Ok(sync_ok && current_ok)
}

pub(crate) async fn remote_ssl_enabled(cfg: &config::Bones, web_root: &str) -> Result<bool> {
    if cfg.domain.is_empty() {
        return Ok(false);
    }

    let session = ssh::connect_privileged(cfg).await?;
    let path = cfg.deployment_paths(web_root).nginx_site_available;
    let command = format!(
        "test -r {path} && grep -Fq {domain} {path} && grep -Fq 'listen 443 ssl;' {path}",
        path = shell_quote(&path),
        domain = shell_quote(&format!("server_name {};", cfg.domain)),
    );
    let enabled = ssh::run_cmd(&session, &command).await.is_ok();
    session.close().await?;

    Ok(enabled)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{initialized_report, prompt_free_init_command, ready_report, uninitialized_report};
    use crate::config::Bones;

    fn sample_cfg() -> Bones {
        Bones {
            project_name: String::from("atlas"),
            host: String::from("deploy.example.com"),
            port: String::from("22"),
            repo_path: String::from("/home/git/atlas.git"),
            project_root: String::from("/srv/sites/atlas"),
            branch: String::from("main"),
            ssl_enabled: false,
            ..Default::default()
        }
    }

    #[test]
    fn init_command_is_prompt_free() {
        assert_eq!(prompt_free_init_command("atlas"), "bonesdeploy init --yes --project-name atlas --host <host>");
    }

    #[test]
    fn uninitialized_report_starts_with_init() {
        let report = uninitialized_report("atlas");
        assert_eq!(report.commands[0], "bonesdeploy init --yes --project-name atlas --host <host>");
    }

    #[test]
    fn initialized_report_suggests_setup_then_ssl_then_deploy() {
        let report = initialized_report(sample_cfg(), false);
        assert_eq!(report.commands[0], "bonesdeploy setup --yes");
        assert_eq!(report.commands[1], "bonesdeploy remote ssl --yes --domain <domain> --email <email>");
        assert_eq!(report.commands[2], "bonesdeploy deploy");
    }

    #[test]
    fn ready_report_suggests_deploy() {
        let mut cfg = sample_cfg();
        cfg.ssl_enabled = true;
        let report = ready_report(cfg);
        assert_eq!(report.commands, vec![String::from("bonesdeploy deploy")]);
    }
}
