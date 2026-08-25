use anyhow::{Result, anyhow};
use bonesdeploy_core::{
    config::{RuntimeBackend, validate_host},
    paths,
};

use crate::config;
use crate::infra::git;
use crate::ui::{output, prompts};

pub fn collect_fresh_config(args: &super::Args) -> Result<config::Bones> {
    let project_name = config::repo_directory_name()?;

    if args.non_interactive {
        return collect_non_interactive(&project_name, None, args);
    }

    collect_from_existing(&project_name, None, args)
}

fn collect_from_existing(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &super::Args,
) -> Result<config::Bones> {
    let project_name = cli_or_prompt(
        args.project_name.as_ref(),
        existing_config.and_then(|cfg| non_empty(&cfg.project_name)),
        || prompts::prompt_project_name(project_name_hint, existing_config),
    )?;
    let branch = cli_or_prompt(args.branch.as_ref(), None, || prompts::prompt_branch(existing_config))?;
    let remote_name = cli_or_prompt(args.remote.as_ref(), None, || prompts::prompt_remote_name(existing_config))?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host =
        cli_or_prompt(args.host.as_ref(), None, || prompts::prompt_host(existing_config, inferred_remote.as_ref()))?;
    let port =
        cli_or_prompt(args.port.as_ref(), None, || prompts::prompt_port(existing_config, inferred_remote.as_ref()))?;
    let repo_path = resolve_repo_path(&project_name, existing_config, inferred_remote.as_ref());
    let project_root = existing_path_override(
        existing_config,
        |cfg| &cfg.project_root,
        &project_name,
        config::default_project_root_for,
    );

    let mut cfg = config::Bones::default();
    cfg.remote_name = remote_name;
    cfg.project_name = project_name;
    cfg.host = host;
    cfg.port = port;
    cfg.branch = branch;
    cfg.repo_path = repo_path;
    cfg.project_root = project_root;
    cfg.runtime.backend = match (args.runtime_backend.as_deref(), existing_config) {
        (Some(value), _) => parse_runtime_backend(value)?,
        (None, Some(existing)) => existing.runtime.backend,
        (None, None) => parse_runtime_backend(&prompts::prompt_runtime_backend(None)?)?,
    };
    apply_existing_fields(&mut cfg, existing_config);
    Ok(cfg)
}

fn cli_or_prompt(
    cli_value: Option<&String>,
    existing_value: Option<String>,
    prompt: impl FnOnce() -> Result<String>,
) -> Result<String> {
    match cli_value {
        Some(v) if !v.is_empty() => Ok(v.trim().to_string()),
        _ => existing_value.map_or_else(prompt, Ok),
    }
}

pub fn collect_non_interactive(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &super::Args,
) -> Result<config::Bones> {
    let project_name = resolve_project_name(args, existing_config, project_name_hint)?;
    let remote_name = resolve_remote_name(args, existing_config);
    let inferred_remote = infer_remote_details(&remote_name)?;
    let host = resolve_host(args, existing_config, inferred_remote.as_ref())?;
    let branch = resolve_branch(args, existing_config);
    let port = resolve_port(args, existing_config, inferred_remote.as_ref());
    validate_host(&host)?;

    let repo_path = resolve_repo_path(&project_name, existing_config, inferred_remote.as_ref());
    let project_root = existing_path_override(
        existing_config,
        |cfg| &cfg.project_root,
        &project_name,
        config::default_project_root_for,
    );

    let mut cfg = config::Bones::default();
    cfg.remote_name = remote_name;
    cfg.project_name = project_name;
    cfg.host = host;
    cfg.port = port;
    cfg.branch = branch;
    cfg.repo_path = repo_path;
    cfg.project_root = project_root;
    cfg.runtime.backend = resolve_runtime_backend(args, existing_config)?;
    apply_existing_fields(&mut cfg, existing_config);
    Ok(cfg)
}

fn resolve_project_name(
    args: &super::Args,
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
                 Usage: {}",
                console::style("Error:").red().bold(),
                output::green_command("bonesdeploy init --non-interactive --project-name <name> --host <host>")
            )
        })
}

fn resolve_remote_name(args: &super::Args, existing_config: Option<&config::Bones>) -> String {
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
    args: &super::Args,
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
                 Usage: {}",
                console::style("Error:").red().bold(),
                output::green_command("bonesdeploy init --non-interactive --project-name <name> --host <host>")
            )
        })
}

fn resolve_branch(args: &super::Args, existing_config: Option<&config::Bones>) -> String {
    args.branch
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.branch)))
        .unwrap_or_else(|| String::from("main"))
}

fn resolve_port(
    args: &super::Args,
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

fn resolve_runtime_backend(args: &super::Args, existing_config: Option<&config::Bones>) -> Result<RuntimeBackend> {
    let value = args
        .runtime_backend
        .as_deref()
        .or_else(|| {
            existing_config.map(|cfg| match cfg.runtime.backend {
                RuntimeBackend::Native => "native",
                RuntimeBackend::Docker => "docker",
            })
        })
        .unwrap_or("native");

    parse_runtime_backend(value)
}

fn parse_runtime_backend(value: &str) -> Result<RuntimeBackend> {
    match value.trim().to_ascii_lowercase().as_str() {
        "native" => Ok(RuntimeBackend::Native),
        "docker" => Ok(RuntimeBackend::Docker),
        _ => anyhow::bail!("unsupported runtime backend: {value}"),
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

    let configured_repo_path = existing_config.map(|cfg| cfg.repo_path.as_str());

    let repo_path = match configured_repo_path {
        Some(path) if !path.is_empty() => path.replace("<project_name>", project_name),
        _ => paths::default_repo_path_for(project_name),
    };

    repo_path
}

pub fn existing_path_override(
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
    if resolved == default_for(current_project_name) { String::new() } else { resolved }
}

fn apply_existing_fields(config: &mut config::Bones, existing_config: Option<&config::Bones>) {
    config.releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));
    config.ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    config.domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    config.email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());
}
