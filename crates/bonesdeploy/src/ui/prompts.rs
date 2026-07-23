use anyhow::{Result, anyhow, bail};
use console::style;
use inquire::{Confirm, MultiSelect, Select, Text};
use serde_json::Value;

use crate::config::Bones;
use crate::infra::git;
use crate::runtimes::{Question, QuestionKind};

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

pub fn prompt_runtime_questions(
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

fn display_name(template: &str) -> String {
    match template {
        "next" => String::from("Next.js"),
        "sveltekit" => String::from("SvelteKit"),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        }
    }
}

pub fn choose_template(available_templates: &[String]) -> Result<Option<String>> {
    if available_templates.is_empty() {
        return Ok(None);
    }

    let choice =
        Select::new("Runtime template:", vec![String::from("Use a template"), String::from("Build from scratch")])
            .with_help_message("Choose the app runtime to configure")
            .prompt()?;

    if choice == "Build from scratch" {
        return Ok(None);
    }

    let display_names: Vec<String> = available_templates.iter().map(|t| display_name(t)).collect();
    let chosen_display = Select::new("Template:", display_names).prompt()?;
    let idx = available_templates.iter().position(|t| display_name(t) == chosen_display).unwrap_or(0);
    let template_name = available_templates[idx].clone();

    Ok(Some(template_name))
}

pub fn choose_database_services(services: &[&str]) -> Result<Vec<String>> {
    MultiSelect::new("Database services:", services.to_vec())
        .with_help_message("All services listen on localhost; use SSH port forwarding for remote access.")
        .prompt()
        .map(|selected| selected.into_iter().map(str::to_string).collect())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_project_name(project_name_hint: &str, existing_config: Option<&Bones>) -> Result<String> {
    let default_project_name = config_default(existing_config, |cfg| cfg.project_name.as_str(), project_name_hint);
    Text::new("Project name:")
        .with_default(default_project_name)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_branch(existing_config: Option<&Bones>) -> Result<String> {
    let default_branch = config_default(existing_config, |cfg| cfg.branch.as_str(), "master");
    Text::new("Branch:")
        .with_default(default_branch)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_remote_name(existing_config: Option<&Bones>) -> Result<String> {
    const CREATE_REMOTE_OPTION: &str = "Create new deployment remote";

    let remotes = git::list_remotes_with_urls()?;
    if remotes.is_empty() {
        return prompt_remote_name_text(existing_config);
    }

    let default_remote = existing_config.and_then(|cfg| {
        let value = cfg.remote_name.as_str();
        (!value.is_empty()).then(|| cfg.remote_name.clone())
    });

    let preferred = default_remote.or_else(|| {
        let has_production = remotes.iter().any(|r| r.name == "production");
        if has_production { Some(String::from("production")) } else { None }
    });

    let mut ordered_remotes = Vec::with_capacity(remotes.len());
    if let Some(ref pref) = preferred
        && let Some(pos) = remotes.iter().position(|r| r.name == *pref)
    {
        ordered_remotes.push(remotes[pos].clone());
        ordered_remotes.extend(remotes.iter().enumerate().filter(|(i, _)| *i != pos).map(|(_, r)| r.clone()));
    }
    if ordered_remotes.is_empty() {
        ordered_remotes = remotes;
    }

    let mut display_options: Vec<String> = ordered_remotes.iter().map(remote_display_label).collect();
    display_options.push(String::from(CREATE_REMOTE_OPTION));

    let choice = Select::new("Deployment remote:", display_options)
        .with_help_message("Choose the VPS remote, not your code host.")
        .raw_prompt()
        .map_err(|err| anyhow!(err))?;

    if choice.index == ordered_remotes.len() {
        return prompt_remote_name_text(existing_config);
    }

    let chosen = ordered_remotes[choice.index].name.clone();

    if chosen == "origin" {
        println!("{} origin usually points to your code host, not your VPS.", style("Warning:").yellow().bold());
        let proceed = Confirm::new("Use 'origin' anyway?")
            .with_default(false)
            .with_help_message("Choose No unless origin points to your VPS.")
            .prompt()
            .map_err(|err| anyhow!(err))?;
        if !proceed {
            bail!("Choose a deployment remote that points to your VPS.");
        }
    }

    Ok(chosen)
}

fn remote_display_label(remote: &git::RemoteInfo) -> String {
    if remote.name == "origin" {
        format!("{} ({}) — not a deployment remote", remote.name, remote.url)
    } else {
        format!("{} ({})", remote.name, remote.url)
    }
}

pub fn prompt_host(
    existing_config: Option<&Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.host.clone());
    }

    let default_host = config_default(existing_config, |cfg| cfg.host.as_str(), "");
    Text::new("Server host or IP:")
        .with_default(default_host)
        .with_help_message("e.g. deploy.example.com or 203.0.113.10")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_port(
    existing_config: Option<&Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.port.clone());
    }

    let default_port = config_default(existing_config, |cfg| cfg.port.as_str(), "22");
    Text::new("SSH port:")
        .with_default(default_port)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn confirm_remote_setup() -> Result<bool> {
    confirm_prompt("Bootstrap remote server?", "Remote bootstrap prepares the VPS for this project.")
}

pub fn confirm_remote_runtime() -> Result<bool> {
    confirm_prompt("Apply runtime setup?", "Runtime setup installs app services for this project.")
}

pub fn confirm_remote_ssl() -> Result<bool> {
    confirm_prompt("Configure HTTPS?", "HTTPS requires DNS to point at this server.")
}

pub fn confirm_remote_helpers() -> Result<bool> {
    confirm_prompt("Install remote helper tools?", "Helper tools install shell and editor utilities on the server.")
}

pub fn confirm_remote_dbs() -> Result<bool> {
    confirm_prompt(
        "Provision database services?",
        "Database services will be bound to localhost and credentials written to shared/.env.",
    )
}

fn confirm_prompt(prompt: &str, message: &str) -> Result<bool> {
    println!();
    println!("{message}");
    println!();
    Confirm::new(prompt).with_default(false).prompt().map_err(|err| anyhow!(err))
}

fn prompt_remote_name_text(existing_config: Option<&Bones>) -> Result<String> {
    let default_remote =
        existing_config.map(|cfg| cfg.remote_name.as_str()).filter(|value| !value.is_empty()).unwrap_or("production");
    Text::new("Deployment remote name:")
        .with_default(default_remote)
        .with_help_message("Created if missing.")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
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
