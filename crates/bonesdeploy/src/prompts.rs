use anyhow::{Result, anyhow};
use inquire::{Select, Text};

use crate::config::{BonesConfig, Data, PermissionDefaults, Permissions, Releases, Runtime, Ssl};
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

pub fn collect(project_name_hint: &str) -> Result<BonesConfig> {
    collect_from_seed(project_name_hint, None)
}

pub fn collect_from_seed(project_name_hint: &str, seed: Option<&BonesConfig>) -> Result<BonesConfig> {
    let project_name = prompt_project_name(project_name_hint, seed)?;
    let branch = prompt_branch(seed)?;
    let remote_name = prompt_remote_name(seed)?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host = prompt_host(seed, inferred_remote.as_ref())?;
    let port = prompt_port(seed, inferred_remote.as_ref())?;
    let git_dir = prompt_git_dir(&project_name, seed, inferred_remote.as_ref())?;
    let live_root = default_live_root(&project_name, seed);
    let deploy_root = default_deploy_root(&project_name, seed);
    let deploy_on_push = seed.is_none_or(|cfg| cfg.data.deploy_on_push);
    let deploy_user = seed
        .map(|cfg| cfg.permissions.defaults.deploy_user.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("git")
        .to_string();
    let service_user = seed
        .map(|cfg| cfg.permissions.defaults.service_user.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(&project_name)
        .to_string();
    let group = seed
        .map(|cfg| cfg.permissions.defaults.group.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("www-data")
        .to_string();
    let dir_mode = seed
        .map(|cfg| cfg.permissions.defaults.dir_mode.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("750")
        .to_string();
    let file_mode = seed
        .map(|cfg| cfg.permissions.defaults.file_mode.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("640")
        .to_string();
    let releases_keep = seed.map_or(5, |cfg| cfg.releases.keep.max(1));
    let shared_paths = seed
        .map(|cfg| cfg.releases.shared_paths.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from(".env"), String::from("storage")]);
    let path_overrides = seed.map_or_else(Vec::new, |cfg| cfg.permissions.paths.clone());

    Ok(BonesConfig {
        data: Data { remote_name, project_name, host, port, git_dir, live_root, deploy_root, branch, deploy_on_push },
        permissions: Permissions {
            defaults: PermissionDefaults { deploy_user, service_user, group, dir_mode, file_mode },
            paths: path_overrides,
        },
        releases: Releases { keep: releases_keep, shared_paths },
        runtime: seed.map_or_else(Runtime::default, |cfg| cfg.runtime.clone()),
        ssl: seed.map_or_else(Ssl::default, |cfg| cfg.ssl.clone()),
    })
}

fn prompt_remote_name(seed: Option<&BonesConfig>) -> Result<String> {
    const CREATE_REMOTE_OPTION: &str = "Create new deployment remote";

    let remotes = git::list_remotes()?;
    if remotes.is_empty() {
        let default_remote =
            seed.map(|cfg| cfg.data.remote_name.as_str()).filter(|value| !value.is_empty()).unwrap_or("production");
        return Text::new("Deployment remote name:")
            .with_default(default_remote)
            .with_help_message("bonesdeploy will create this git remote if it does not exist")
            .prompt()
            .map_err(|err| anyhow!(err));
    }

    let default_remote = seed.map(|cfg| cfg.data.remote_name.clone()).filter(|value| !value.is_empty());
    let mut options = if let Some(default_remote) = default_remote {
        if remotes.contains(&default_remote) {
            let mut ordered = Vec::with_capacity(remotes.len());
            ordered.push(default_remote.clone());
            ordered.extend(remotes.into_iter().filter(|name| name != &default_remote));
            ordered
        } else {
            remotes
        }
    } else {
        remotes
    };
    options.push(String::from(CREATE_REMOTE_OPTION));

    let choice = Select::new("Deployment remote:", options)
        .with_help_message("Choose the git remote bonesdeploy will manage")
        .prompt()
        .map_err(|err| anyhow!(err))?;

    if choice == CREATE_REMOTE_OPTION {
        let default_remote =
            seed.map(|cfg| cfg.data.remote_name.as_str()).filter(|value| !value.is_empty()).unwrap_or("production");
        return Text::new("Deployment remote name:")
            .with_default(default_remote)
            .with_help_message("bonesdeploy will create this git remote if it does not exist")
            .prompt()
            .map_err(|err| anyhow!(err));
    }

    Ok(choice)
}

fn prompt_port(seed: Option<&BonesConfig>, inferred_remote: Option<&git::RemoteConnectionDetails>) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.port.clone());
    }

    let default_port = seed.map(|cfg| cfg.data.port.as_str()).filter(|value| !value.is_empty()).unwrap_or("22");
    Text::new("SSH port:").with_default(default_port).prompt().map_err(|err| anyhow!(err))
}

fn prompt_project_name(project_name_hint: &str, seed: Option<&BonesConfig>) -> Result<String> {
    let default_project_name =
        seed.map(|cfg| cfg.data.project_name.as_str()).filter(|value| !value.is_empty()).unwrap_or(project_name_hint);
    Text::new("Project name:").with_default(default_project_name).prompt().map_err(|err| anyhow!(err))
}

fn prompt_host(seed: Option<&BonesConfig>, inferred_remote: Option<&git::RemoteConnectionDetails>) -> Result<String> {
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

fn prompt_git_dir(
    project_name: &str,
    seed: Option<&BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.git_dir.clone());
    }

    let default_git_dir = seed
        .map(|cfg| cfg.data.git_dir.as_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("/home/git/{project_name}.git"), |value| value.replace("<project_name>", project_name));
    Text::new("Git directory (could not infer from remote URL):")
        .with_default(&default_git_dir)
        .prompt()
        .map_err(|err| anyhow!(err))
}

fn default_live_root(project_name: &str, seed: Option<&BonesConfig>) -> String {
    seed.map(|cfg| cfg.data.live_root.as_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("/var/www/{project_name}"), |value| value.replace("<project_name>", project_name))
}

fn default_deploy_root(project_name: &str, seed: Option<&BonesConfig>) -> String {
    seed.map(|cfg| cfg.data.deploy_root.as_str()).filter(|value| !value.is_empty()).map_or_else(
        || format!("/srv/deployments/{project_name}"),
        |value| value.replace("<project_name>", project_name),
    )
}

fn prompt_branch(seed: Option<&BonesConfig>) -> Result<String> {
    let default_branch = seed.map(|cfg| cfg.data.branch.as_str()).filter(|value| !value.is_empty()).unwrap_or("main");
    Text::new("Branch:").with_default(default_branch).prompt().map_err(|err| anyhow!(err))
}

pub fn prompt_bootstrap_ssh_user(seed: Option<&BonesConfig>) -> Result<String> {
    let default_user = seed
        .map(|cfg| cfg.permissions.defaults.deploy_user.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("root");
    Text::new("Server SSH user for initial setup:")
        .with_default(default_user)
        .with_help_message("Used only for the first ansible run before deploy user access is ready")
        .prompt()
        .map_err(|err| anyhow!(err))
}
