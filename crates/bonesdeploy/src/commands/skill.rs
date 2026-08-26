use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use serde::Serialize;

use crate::cli::args::{GuideFormat, SkillCommand};
use crate::config;
use crate::infra::assets::skill;
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
    print!("{}", skill::orientation()?);
    Ok(())
}

pub fn list_docs() {
    for name in skill::doc_names() {
        println!("{name}");
    }
}

pub fn print_doc(name: &str) -> Result<()> {
    print!("{}", skill::doc(name)?);
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
    let bones_toml = Path::new(paths::DOT_ENV);

    if !bones_toml.exists() {
        return Ok(uninitialized_report(&project));
    }

    let cfg = config::load(bones_toml).with_context(|| format!("Failed to read {}", bones_toml.display()))?;

    if !server_ready(&cfg).await.context("Unable to determine server readiness")? {
        return Ok(server_missing_report(cfg));
    }

    if !site_ready(&cfg).await.context("Unable to determine site readiness")? {
        return Ok(site_missing_report(cfg));
    }

    let ssl_enabled =
        cfg.ssl_enabled || remote_ssl_enabled(&cfg).await.context("Unable to determine remote SSL status")?;

    if ssl_enabled { Ok(ready_report(cfg)) } else { Ok(ssl_missing_report(cfg)) }
}

pub fn prompt_free_init_command(project: &str) -> String {
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

fn server_missing_report(cfg: config::Bones) -> Report {
    let command = String::from("bonesdeploy server setup --yes");
    let commands = vec![
        command.clone(),
        String::from("bonesdeploy site setup --yes"),
        ssl_command(&cfg),
        String::from("bonesdeploy deploy"),
    ];
    Report {
        project: cfg.project_name.clone(),
        state: String::from("server_missing"),
        state_label: String::from("initialized, server baseline missing."),
        missing: vec![String::from("server_baseline")],
        commands,
        next: next_command(&command, true, true),
        cfg: Some(cfg),
    }
}

fn site_missing_report(cfg: config::Bones) -> Report {
    let command = String::from("bonesdeploy site setup --yes");
    let commands = vec![command.clone(), ssl_command(&cfg), String::from("bonesdeploy deploy")];
    Report {
        project: cfg.project_name.clone(),
        state: String::from("site_missing"),
        state_label: String::from("server ready, site not provisioned."),
        missing: vec![String::from("site_base"), String::from("runtime"), String::from("doctor_pass")],
        commands,
        next: next_command(&command, true, true),
        cfg: Some(cfg),
    }
}

fn ssl_missing_report(cfg: config::Bones) -> Report {
    let command = ssl_command(&cfg);
    let commands = vec![command.clone(), String::from("bonesdeploy deploy")];
    Report {
        project: cfg.project_name.clone(),
        state: String::from("ssl_missing"),
        state_label: String::from("site provisioned, HTTPS missing."),
        missing: vec![String::from("ssl")],
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
    format!("bonesdeploy site ssl --yes --domain {domain} --email {email}")
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

async fn server_ready(cfg: &config::Bones) -> Result<bool> {
    let Ok(session) = ssh::connect_privileged(cfg).await else {
        return Ok(false);
    };

    let bonesremote_installed = ssh::run_cmd(&session, "command -v bonesremote >/dev/null 2>&1").await.is_ok();

    let host_doctor_ok = if bonesremote_installed {
        ssh::run_cmd(&session, "bonesremote doctor >/dev/null 2>&1").await.is_ok()
    } else {
        false
    };

    session.close().await?;
    Ok(host_doctor_ok)
}

async fn site_ready(cfg: &config::Bones) -> Result<bool> {
    let Ok(session) = ssh::connect_privileged(cfg).await else {
        return Ok(false);
    };

    let registry_path = Path::new(&cfg.project_root).join(paths::SHARED_DIR).join(paths::DOT_ENV);
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
    use super::{ready_report, server_missing_report, site_missing_report, ssl_missing_report, uninitialized_report};
    use crate::config;

    fn config() -> config::Bones {
        let mut cfg = config::Bones::default();
        cfg.project_name = "atlas".to_string();
        cfg.domain = "atlas.example.com".to_string();
        cfg.email = "ops@example.com".to_string();
        cfg
    }

    #[test]
    fn readiness_reports_follow_the_server_site_ssl_sequence() {
        let uninitialized = uninitialized_report("atlas");
        assert_eq!(uninitialized.state, "uninitialized");
        assert!(uninitialized.next.command.starts_with("bonesdeploy init"));

        let server_missing = server_missing_report(config());
        assert_eq!(server_missing.state, "server_missing");
        assert_eq!(server_missing.next.command, "bonesdeploy server setup --yes");

        let site_missing = site_missing_report(config());
        assert_eq!(site_missing.state, "site_missing");
        assert_eq!(site_missing.next.command, "bonesdeploy site setup --yes");

        let ssl_missing = ssl_missing_report(config());
        assert_eq!(ssl_missing.state, "ssl_missing");
        assert_eq!(
            ssl_missing.next.command,
            "bonesdeploy site ssl --yes --domain atlas.example.com --email ops@example.com"
        );

        let ready = ready_report(config());
        assert_eq!(ready.state, "ready");
        assert_eq!(ready.next.command, "bonesdeploy deploy");
    }
}
