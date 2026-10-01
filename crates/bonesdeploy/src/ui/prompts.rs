use anyhow::{Result, anyhow};
use inquire::{Confirm, Select, Text};
use serde_json::Value;

use crate::config::Bones;
use crate::frameworks::{Framework, Question, QuestionKind};
use bonesdeploy_core::config::RuntimeBackend;

fn config_default<'a>(
    existing_config: Option<&'a Bones>,
    accessor: impl Fn(&'a Bones) -> &'a str,
    fallback: &'a str,
) -> &'a str {
    existing_config
        .and_then(|cfg| {
            let value = accessor(cfg);
            (!value.is_empty()).then_some(value)
        })
        .unwrap_or(fallback)
}

pub fn prompt_framework_questions(
    questions: &[Question],
    defaults: &serde_json::Map<String, Value>,
) -> Result<serde_json::Map<String, Value>> {
    let mut answers = defaults.clone();

    for question in questions {
        let current = answers.get(question.key).cloned().unwrap_or_else(|| question.default_value());
        let answer: Value = match question.kind {
            QuestionKind::Bool { default } => {
                let default_bool = current.as_bool().unwrap_or(default);
                let choice =
                    Confirm::new(question.label).with_default(default_bool).prompt().map_err(|err| anyhow!(err))?;
                Value::Bool(choice)
            }
            QuestionKind::Choice { choices, default } => {
                let choices: Vec<String> = choices.iter().map(|c| (*c).to_string()).collect();
                let default_idx = current
                    .as_str()
                    .and_then(|d| choices.iter().position(|c| c == d))
                    .unwrap_or_else(|| choices.iter().position(|c| c == default).unwrap_or(0));
                let choice = Select::new(question.label, choices.clone())
                    .with_starting_cursor(default_idx)
                    .prompt()
                    .map_err(|err| anyhow!(err))?;
                Value::String(choice)
            }
            QuestionKind::Text { default } => {
                let default_str = current.as_str().unwrap_or(default);
                let input = Text::new(question.label).with_default(default_str).prompt().map_err(|err| anyhow!(err))?;
                Value::String(input)
            }
        };
        answers.insert(question.key.to_string(), answer);
    }

    Ok(answers)
}

pub fn choose_template(available_templates: &[String]) -> Result<Option<String>> {
    if available_templates.is_empty() {
        return Ok(None);
    }

    let choice =
        Select::new("Framework template:", vec![String::from("Use a template"), String::from("Build from scratch")])
            .with_help_message("Choose the app framework to configure")
            .prompt()?;

    if choice == "Build from scratch" {
        return Ok(None);
    }

    let display_names: Vec<String> = available_templates
        .iter()
        .map(|template| Framework::parse(template).map(|framework| framework.display_name()))
        .collect::<Result<_>>()?;
    let chosen_display = Select::new("Template:", display_names).prompt()?;
    let idx = available_templates
        .iter()
        .position(|template| {
            Framework::parse(template).is_ok_and(|framework| framework.display_name() == chosen_display)
        })
        .unwrap_or(0);
    let template_name = available_templates[idx].clone();

    Ok(Some(template_name))
}

pub fn prompt_project_name(project_name_hint: &str, existing_config: Option<&Bones>) -> Result<String> {
    let default_project_name = config_default(existing_config, |cfg| cfg.project_name.as_str(), project_name_hint);
    Text::new("Project name:")
        .with_default(default_project_name)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_runtime_backend(existing_config: Option<&Bones>) -> Result<String> {
    let options = vec![String::from("Native"), String::from("Docker Compose")];
    let default = existing_config.map_or(0, |cfg| match cfg.runtime.backend {
        RuntimeBackend::Native => 0,
        RuntimeBackend::Docker => 1,
    });

    Select::new("How should the application run?", options)
        .with_starting_cursor(default)
        .prompt()
        .map_err(|err| anyhow!(err))
}

pub fn prompt_branch(existing_config: Option<&Bones>) -> Result<String> {
    let default_branch = config_default(existing_config, |cfg| cfg.branch.as_str(), "main");
    Text::new("Branch:")
        .with_default(default_branch)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_host(existing_config: Option<&Bones>) -> Result<String> {
    let default_host = config_default(existing_config, |cfg| cfg.host.as_str(), "");
    Text::new("Server host or IP:")
        .with_default(default_host)
        .with_help_message("e.g. deploy.example.com or 203.0.113.10")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_port(existing_config: Option<&Bones>) -> Result<String> {
    let default_port = config_default(existing_config, |cfg| cfg.port.as_str(), "22");
    Text::new("SSH port:")
        .with_default(default_port)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn confirm_server_setup() -> Result<bool> {
    confirm_prompt("Set up server baseline?", "Server setup prepares the VPS host for all projects.")
}

pub fn confirm_site_setup() -> Result<bool> {
    confirm_prompt("Set up site?", "Site setup provisions this project on the server after the baseline is ready.")
}

pub fn confirm_site_delete(project_name: &str) -> Result<bool> {
    println!();
    println!("This permanently removes remote resources for {project_name}.");
    let entered =
        Text::new(&format!("Type {project_name} to confirm deletion:")).prompt().map_err(|err| anyhow!(err))?;
    Ok(entered.trim() == project_name)
}

pub fn confirm_site_import(project_name: &str) -> Result<bool> {
    println!();
    println!("This replaces non-environment shared data for {project_name} and briefly stops its services.");
    let entered = Text::new(&format!("Type {project_name} to confirm import:")).prompt().map_err(|err| anyhow!(err))?;
    Ok(entered.trim() == project_name)
}

pub fn confirm_site_runtime() -> Result<bool> {
    confirm_prompt("Apply runtime setup?", "Runtime setup installs app services for this project.")
}

pub fn confirm_site_tunnel_start() -> Result<bool> {
    confirm_prompt(
        "Start a Cloudflare Quick Tunnel?",
        "This installs cloudflared and exposes the site through an ephemeral public URL.",
    )
}

pub fn confirm_site_tunnel_stop() -> Result<bool> {
    confirm_prompt(
        "Stop and remove the Cloudflare Quick Tunnel?",
        "The current trycloudflare.com URL will stop working.",
    )
}

pub fn confirm_site_ssl() -> Result<bool> {
    confirm_prompt("Configure HTTPS?", "HTTPS requires DNS to point at this server.")
}

pub fn confirm_server_helpers() -> Result<bool> {
    confirm_prompt("Install server helper tools?", "Helper tools install shell and editor utilities on the server.")
}

fn confirm_prompt(prompt: &str, message: &str) -> Result<bool> {
    println!();
    println!("{message}");
    println!();
    Confirm::new(prompt).with_default(false).prompt().map_err(|err| anyhow!(err))
}

pub fn prompt_ssl_domain(existing_config: Option<&Bones>) -> Result<String> {
    let default_domain = config_default(existing_config, |cfg| cfg.domain.as_str(), "");
    Text::new("Domain:")
        .with_default(default_domain)
        .with_help_message("e.g. app.example.com")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_ssl_email(existing_config: Option<&Bones>) -> Result<String> {
    let default_email = config_default(existing_config, |cfg| cfg.email.as_str(), "");
    Text::new("Let's Encrypt email:")
        .with_default(default_email)
        .with_help_message("e.g. ops@example.com")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}
