use crate::config;
use crate::git;

use anyhow::{Result, anyhow};
use shared::paths;

pub struct InitArgs {
    pub non_interactive: bool,
    pub setup_remote: bool,
    pub project_name: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
}

pub fn collect_non_interactive(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
    let project_name = resolve_project_name(args, existing_config, project_name_hint)?;
    let remote_name = resolve_remote_name(args, existing_config);
    let inferred_remote = infer_remote_details(&remote_name)?;
    let host = resolve_host(args, existing_config, inferred_remote.as_ref())?;
    let branch = resolve_branch(args, existing_config);
    let port = resolve_port(args, existing_config, inferred_remote.as_ref());

    let values = NonInteractiveValues { project_name, remote_name, branch, host, port };
    Ok(build_config(values, existing_config, inferred_remote.as_ref()))
}

fn resolve_project_name(
    args: &InitArgs,
    existing_config: Option<&config::Bones>,
    project_name_hint: &str,
) -> Result<String> {
    args.project_name
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.project_name)))
        .or_else(|| {
            let name = project_name_hint.to_string();
            (!name.is_empty()).then_some(name)
        })
        .ok_or_else(|| {
            anyhow!(
                "{} --project-name is required in non-interactive mode.\n\
                 Usage: bonesdeploy init --non-interactive --project-name <name> --host <host>",
                console::style("Error:").red().bold(),
            )
        })
}

fn resolve_remote_name(args: &InitArgs, existing_config: Option<&config::Bones>) -> String {
    args.remote
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.remote_name)))
        .unwrap_or_else(|| String::from("production"))
}

fn infer_remote_details(remote_name: &str) -> Result<Option<git::RemoteConnectionDetails>> {
    if git::remote_exists(remote_name)? { git::infer_remote_connection_details(remote_name) } else { Ok(None) }
}

fn resolve_host(
    args: &InitArgs,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    args.host
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.host)))
        .or_else(|| inferred_remote.map(|details| details.host.clone()))
        .ok_or_else(|| {
            anyhow!(
                "{} --host is required in non-interactive mode.\n\
                 Usage: bonesdeploy init --non-interactive --project-name <name> --host <host>",
                console::style("Error:").red().bold(),
            )
        })
}

fn resolve_branch(args: &InitArgs, existing_config: Option<&config::Bones>) -> String {
    args.branch
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.branch)))
        .unwrap_or_else(|| String::from("main"))
}

fn resolve_port(
    args: &InitArgs,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    args.port
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.port)))
        .or_else(|| inferred_remote.map(|details| details.port.clone()))
        .unwrap_or_else(|| String::from("22"))
}

struct NonInteractiveValues {
    project_name: String,
    remote_name: String,
    branch: String,
    host: String,
    port: String,
}

fn build_config(
    values: NonInteractiveValues,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> config::Bones {
    let NonInteractiveValues { project_name, remote_name, branch, host, port } = values;

    let repo_path = resolve_repo_path(&project_name, existing_config, inferred_remote);
    let project_root = seed_path_override(
        existing_config,
        |cfg| &cfg.project_root,
        &project_name,
        config::default_project_root_for,
    );
    let deploy_on_push = existing_config.is_none_or(|cfg| cfg.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));

    let ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    let domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    let email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());

    config::Bones {
        remote_name,
        project_name,
        host,
        port,
        repo_path,
        project_root,
        branch,
        deploy_on_push,
        releases_keep,
        ssl_enabled,
        domain,
        email,
        ..Default::default()
    }
}

pub fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

pub fn resolve_repo_path(
    project_name: &str,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    if let Some(details) = inferred_remote {
        return details.repo_path.clone();
    }

    existing_config.map(|cfg| cfg.repo_path.as_str()).filter(|value| !value.is_empty()).map_or_else(
        || paths::default_repo_path_for(project_name),
        |value| value.replace("<project_name>", project_name),
    )
}

pub fn seed_path_override(
    existing_config: Option<&config::Bones>,
    field: impl Fn(&config::Bones) -> &String,
    current_project_name: &str,
    default_for: fn(&str) -> String,
) -> String {
    let Some(cfg) = existing_config else { return String::new() };
    let value = field(cfg);
    if value.is_empty() {
        return String::new();
    }
    let resolved = value.replace("<project_name>", current_project_name);
    if resolved == default_for(current_project_name) {
        String::new()
    } else {
        resolved
    }
}
