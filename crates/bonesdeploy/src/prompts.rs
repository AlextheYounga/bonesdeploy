use anyhow::{Result, anyhow};
use inquire::{Select, Text};

use crate::config::BonesConfig;
use crate::git;

pub fn choose_template(available_templates: &[String]) -> Result<Option<String>> {
    let mut options = Vec::with_capacity(available_templates.len() + 1);
    options.push(String::from("Build from scratch"));
    options.extend(available_templates.iter().map(|name| format!("Use template: {name}")));

    let choice = Select::new("How would you like to initialize this project?", options)
        .with_help_message("Choose scratch for the current flow, or pick a template scaffold")
        .prompt()?;

    if choice == "Build from scratch" {
        return Ok(None);
    }

    let template_name = choice.strip_prefix("Use template: ").unwrap_or_default().to_string();

    if template_name.is_empty() {
        return Ok(None);
    }

    Ok(Some(template_name))
}

pub fn prompt_project_name(project_name_hint: &str, seed: Option<&BonesConfig>) -> Result<String> {
    let default_project_name =
        seed.map(|cfg| cfg.data.project_name.as_str()).filter(|value| !value.is_empty()).unwrap_or(project_name_hint);
    Text::new("Project name:").with_default(default_project_name).prompt().map_err(|err| anyhow!(err))
}

pub fn prompt_branch(seed: Option<&BonesConfig>) -> Result<String> {
    let default_branch = seed.map(|cfg| cfg.data.branch.as_str()).filter(|value| !value.is_empty()).unwrap_or("main");
    Text::new("Branch:").with_default(default_branch).prompt().map_err(|err| anyhow!(err))
}

pub fn prompt_remote_name(seed: Option<&BonesConfig>) -> Result<String> {
    const CREATE_REMOTE_OPTION: &str = "Create new deployment remote";

    let remotes = git::list_remotes()?;
    if remotes.is_empty() {
        return prompt_remote_name_text(seed);
    }

    let default_remote = seed.map(|cfg| cfg.data.remote_name.clone()).filter(|value| !value.is_empty());
    let mut options = order_remotes_with_default(remotes, default_remote);
    options.push(String::from(CREATE_REMOTE_OPTION));

    let choice = Select::new("Deployment remote:", options)
        .with_help_message("Choose the git remote bonesdeploy will manage")
        .prompt()
        .map_err(|err| anyhow!(err))?;

    if choice == CREATE_REMOTE_OPTION {
        return prompt_remote_name_text(seed);
    }

    Ok(choice)
}

pub fn prompt_host(
    seed: Option<&BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.host.clone());
    }

    let default_host = seed.map(|cfg| cfg.data.host.as_str()).filter(|value| !value.is_empty()).unwrap_or("");
    Text::new("Server host or IP:")
        .with_default(default_host)
        .with_help_message("e.g. deploy.example.com or 203.0.113.10")
        .prompt()
        .map_err(|err| anyhow!(err))
}

pub fn prompt_port(
    seed: Option<&BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.port.clone());
    }

    let default_port = seed.map(|cfg| cfg.data.port.as_str()).filter(|value| !value.is_empty()).unwrap_or("22");
    Text::new("SSH port:").with_default(default_port).prompt().map_err(|err| anyhow!(err))
}

fn prompt_remote_name_text(seed: Option<&BonesConfig>) -> Result<String> {
    let default_remote =
        seed.map(|cfg| cfg.data.remote_name.as_str()).filter(|value| !value.is_empty()).unwrap_or("production");
    Text::new("Deployment remote name:")
        .with_default(default_remote)
        .with_help_message("bonesdeploy will create this git remote if it does not exist")
        .prompt()
        .map_err(|err| anyhow!(err))
}

fn order_remotes_with_default(remotes: Vec<String>, default_remote: Option<String>) -> Vec<String> {
    let Some(default_remote) = default_remote else {
        return remotes;
    };
    if !remotes.contains(&default_remote) {
        return remotes;
    }

    let mut ordered = Vec::with_capacity(remotes.len());
    ordered.push(default_remote.clone());
    ordered.extend(remotes.into_iter().filter(|name| name != &default_remote));
    ordered
}
