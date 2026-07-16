use std::path::Path;

use anyhow::{Result, anyhow};
use shared::{config::validate_host, paths};

use crate::config;
use crate::infra::git;
use crate::ui::{output, prompts};

pub(super) fn collect_fresh_config(args: &super::Args) -> Result<config::Bones> {
    let project_name = config::repo_directory_name()?;

    if args.non_interactive {
        return collect_non_interactive(&project_name, None, args);
    }

    collect_from_existing(&project_name, None, args)
}

pub(super) fn load_or_collect_config(bones_toml: &Path, args: &super::Args) -> Result<config::Bones> {
    if bones_toml.exists() {
        let existing = config::load(bones_toml)?;
        if config::is_configured(&existing) {
            return Ok(existing);
        }

        let project_name = config::repo_directory_name()?;
        if args.non_interactive {
            return collect_non_interactive(&project_name, Some(&existing), args);
        }

        return collect_from_existing(&project_name, Some(&existing), args);
    }

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
    let deploy_on_push = existing_config.map_or(false, |cfg| cfg.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));

    let ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    let domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    let email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());

    let mut config = config::Bones::default();
    config.remote_name = remote_name;
    config.project_name = project_name;
    config.host = host;
    config.port = port;
    config.repo_path = repo_path;
    config.project_root = project_root;
    config.branch = branch;
    config.deploy_on_push = deploy_on_push;
    config.releases_keep = releases_keep;
    config.ssl_enabled = ssl_enabled;
    config.domain = domain;
    config.email = email;
    Ok(config)
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

pub(super) fn collect_non_interactive(
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
    let deploy_on_push = existing_config.map_or(false, |cfg| cfg.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));

    let ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    let domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    let email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());

    let mut config = config::Bones::default();
    config.remote_name = remote_name;
    config.project_name = project_name;
    config.host = host;
    config.port = port;
    config.repo_path = repo_path;
    config.project_root = project_root;
    config.branch = branch;
    config.deploy_on_push = deploy_on_push;
    config.releases_keep = releases_keep;
    config.ssl_enabled = ssl_enabled;
    config.domain = domain;
    config.email = email;
    Ok(config)
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

#[cfg(test)]
mod tests {
    use anyhow::{Result, bail};
    use shared::paths;

    use super::collect_non_interactive;
    use crate::config::Bones;

    fn incomplete_existing(project_name: &str) -> Bones {
        let mut config = Bones::default();
        config.remote_name = String::from("production");
        config.project_name = String::from(project_name);
        config.port = String::from("22");
        config.branch = String::from("main");
        config.deploy_on_push = true;
        config
    }

    #[test]
    fn uses_existing_and_cli_values_without_prompting() -> Result<()> {
        let existing = incomplete_existing("atlas");
        let args = super::super::Args {
            non_interactive: true,
            project_name: None,
            branch: None,
            remote: None,
            host: Some(String::from("deploy.example.com")),
            port: None,
        };

        let cfg = collect_non_interactive("workspace", Some(&existing), &args)?;

        assert_eq!(cfg.project_name, "atlas");
        assert_eq!(cfg.host, "deploy.example.com");
        assert_eq!(cfg.branch, "main");
        assert_eq!(cfg.remote_name, "production");
        assert_eq!(cfg.repo_path, paths::default_repo_path_for("atlas"));

        Ok(())
    }

    #[test]
    fn requires_host_when_existing_and_cli_are_missing_it() -> Result<()> {
        let existing = incomplete_existing("atlas");
        let args = super::super::Args {
            non_interactive: true,
            project_name: None,
            branch: None,
            remote: None,
            host: None,
            port: None,
        };

        let result = collect_non_interactive("workspace", Some(&existing), &args);
        let Err(err) = result else {
            bail!("missing host should fail");
        };
        assert!(err.to_string().contains("--host is required"));

        Ok(())
    }
}
