use std::io::{self, Write};

use anyhow::{Context, Result, anyhow, bail};
use console::style;
use inquire::{Confirm, Select, Text};

use crate::config::BonesConfig;
use crate::git;

pub fn choose_template(available_templates: &[String]) -> Result<Option<String>> {
    if available_templates.is_empty() {
        return Ok(None);
    }

    let choice = Select::new(
        "Would you like to use a template or build from scratch?",
        vec![String::from("Use a template"), String::from("Build from scratch")],
    )
    .with_help_message("Pick a stack to scaffold, or start from scratch")
    .prompt()?;

    if choice == "Build from scratch" {
        return Ok(None);
    }

    let template_name = Select::new("Which template stack would you like to use?", available_templates.to_vec())
        .with_help_message("Choose the framework stack to scaffold")
        .prompt()?;

    Ok(Some(template_name))
}

pub fn prompt_project_name(project_name_hint: &str, existing_config: Option<&BonesConfig>) -> Result<String> {
    let default_project_name = existing_config
        .map(|cfg| cfg.data.project_name.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(project_name_hint);
    Text::new("Project name:")
        .with_default(default_project_name)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_branch(existing_config: Option<&BonesConfig>) -> Result<String> {
    let default_branch =
        existing_config.map(|cfg| cfg.data.branch.as_str()).filter(|value| !value.is_empty()).unwrap_or("main");
    Text::new("Branch:")
        .with_default(default_branch)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_remote_name(existing_config: Option<&BonesConfig>) -> Result<String> {
    const CREATE_REMOTE_OPTION: &str = "Create new deployment remote";

    let remotes = git::list_remotes_with_urls()?;
    if remotes.is_empty() {
        return prompt_remote_name_text(existing_config);
    }

    let default_remote = existing_config.map(|cfg| cfg.data.remote_name.clone()).filter(|value| !value.is_empty());

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
        .with_help_message(
            "Choose the git remote that points to a fresh VPS for production deployment. Do not use 'origin' — that is your code host, not a deployment target.",
        )
        .raw_prompt()
        .map_err(|err| anyhow!(err))?;

    if choice.index == ordered_remotes.len() {
        return prompt_remote_name_text(existing_config);
    }

    let chosen = ordered_remotes[choice.index].name.clone();

    if chosen == "origin" {
        println!();
        println!("{}", style("WARNING:").yellow().bold());
        println!("You selected 'origin' as your deployment remote.");
        println!("'origin' typically points to your code host (e.g. GitHub, GitLab) — not to a VPS");
        println!("where bonesdeploy can deploy your application. Using it here will likely misconfigure");
        println!("deployment and push deployment infrastructure to the wrong place.");
        println!();
        let proceed = Confirm::new("Use 'origin' anyway?")
            .with_default(false)
            .with_help_message("Choose 'No' and create a new deployment remote instead")
            .prompt()
            .map_err(|err| anyhow!(err))?;
        if !proceed {
            bail!("Aborted: choose a remote that points to a fresh VPS, or create a new one.");
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
    existing_config: Option<&BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.host.clone());
    }

    let default_host =
        existing_config.map(|cfg| cfg.data.host.as_str()).filter(|value| !value.is_empty()).unwrap_or("");
    Text::new("Server host or IP:")
        .with_default(default_host)
        .with_help_message("e.g. deploy.example.com or 203.0.113.10")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_port(
    existing_config: Option<&BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.port.clone());
    }

    let default_port =
        existing_config.map(|cfg| cfg.data.port.as_str()).filter(|value| !value.is_empty()).unwrap_or("22");
    Text::new("SSH port:")
        .with_default(default_port)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn confirm_remote_setup() -> Result<bool> {
    println!();
    let mut lines = remote_setup_prompt_lines().into_iter();
    if let Some(header) = lines.next() {
        println!("{}", style(header).cyan().bold());
    }
    for line in lines {
        println!("{line}");
    }
    println!();
    print!("Set up the server now? [y/N] ");
    io::stdout().flush().context("Failed to flush confirmation prompt")?;

    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return Ok(false);
    }

    Ok(is_affirmative(&answer))
}

fn is_affirmative(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn remote_setup_prompt_lines() -> [&'static str; 8] {
    [
        "Remote setup",
        "This is intended for a fresh VPS, but is idempotent (can be run multiple times).",
        "You can use this to set up as many sites on your VPS as you would like. Run this once per site.",
        "It will:",
        "  - install bonesremote",
        "  - configure users, roles, firewalls, and AppArmor",
        "  - add your git repo to the server",
        "  - provision bonesdeploy resources",
    ]
}

fn prompt_remote_name_text(existing_config: Option<&BonesConfig>) -> Result<String> {
    let default_remote = existing_config
        .map(|cfg| cfg.data.remote_name.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("production");
    Text::new("Deployment remote name:")
        .with_default(default_remote)
        .with_help_message("bonesdeploy will add this local git remote if it does not exist")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_ssl_domain(existing_config: Option<&BonesConfig>) -> Result<String> {
    let default_domain =
        existing_config.map(|cfg| cfg.ssl.domain.as_str()).filter(|value| !value.is_empty()).unwrap_or("");
    Text::new("SSL domain:")
        .with_default(default_domain)
        .with_help_message("e.g. app.example.com")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_ssl_email(existing_config: Option<&BonesConfig>) -> Result<String> {
    let default_email =
        existing_config.map(|cfg| cfg.ssl.email.as_str()).filter(|value| !value.is_empty()).unwrap_or("");
    Text::new("SSL email:")
        .with_default(default_email)
        .with_help_message("e.g. ops@example.com")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

#[cfg(test)]
mod tests {
    use super::{is_affirmative, remote_setup_prompt_lines};

    /// Accepts common yes values like y, yes, and YES.
    #[test]
    fn confirmation_parser_accepts_common_yes_values() {
        assert!(is_affirmative("y"));
        assert!(is_affirmative(" yes "));
        assert!(is_affirmative("YES"));
    }

    /// Rejects non-affirmative values like empty string, n, and no.
    #[test]
    fn confirmation_parser_rejects_non_affirmative_values() {
        assert!(!is_affirmative(""));
        assert!(!is_affirmative("n"));
        assert!(!is_affirmative("no"));
    }

    /// Describes firewall configuration in the remote setup prompt.
    #[test]
    fn remote_setup_prompt_lines_include_firewall_configuration() {
        let joined = remote_setup_prompt_lines().join("\n");

        assert!(joined.contains("firewalls"), "remote setup prompt should describe firewall configuration\n{joined}");
    }
}
