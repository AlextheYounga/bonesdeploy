use std::env;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use console::style;

use crate::commands::remote_setup;
use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;

pub struct InitArgs {
    pub non_interactive: bool,
    pub setup_remote: bool,
    pub project_name: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub template: Option<String>,
}

pub fn run(args: &InitArgs) -> Result<()> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if bones_dir.exists() {
        println!(".bones/ already exists, skipping scaffold extraction.");
    } else {
        let available_templates = embedded::available_templates();
        let selected_template = resolve_template(args.template.as_deref(), &available_templates, args.non_interactive)?;

        println!("Creating .bones/ scaffold...");
        embedded::scaffold(bones_dir)?;

        if let Some(ref template_name) = selected_template {
            embedded::scaffold_template(template_name, bones_dir)?;
            println!("Applied template: {template_name}");
        } else {
            println!("Using build-from-scratch scaffold.");
        }
    }

    update_gitignore()?;

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = load_or_collect_config(bones_yaml, args)?;

    config::save(&cfg, bones_yaml)?;
    println!("Saved config to {}", config::Constants::BONES_YAML);
    ensure_local_remote(&cfg)?;

    symlink_pre_push()?;

    if args.setup_remote || (!args.non_interactive && prompts::confirm_remote_setup()?) {
        remote_setup::run()?;
    } else {
        print_follow_up_hint();
    }

    Ok(())
}

fn print_follow_up_hint() {
    println!();
    println!("{}", style("Next:").cyan().bold());
    println!("Run {} to sync {} to the remote.", style("bonesdeploy push").cyan(), style(".bones/").cyan());
}

fn resolve_template(cli_value: Option<&str>, available: &[String], non_interactive: bool) -> Result<Option<String>> {
    if let Some(value) = cli_value.filter(|v| !v.is_empty()) {
        if !available.iter().any(|t| t == value) {
            bail!("Template '{value}' not found. Available templates: {}", available.join(", "));
        }
        return Ok(Some(value.to_string()));
    }
    if non_interactive {
        return Ok(None);
    }
    prompts::choose_template(available)
}

fn collect_non_interactive(
    project_name_hint: &str,
    seed: Option<&config::BonesConfig>,
    args: &InitArgs,
) -> Result<config::BonesConfig> {
    let project_name = args
        .project_name
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| seed.and_then(|cfg| non_empty(&cfg.data.project_name)))
        .or_else(|| {
            let name = project_name_hint.to_string();
            (!name.is_empty()).then_some(name)
        })
        .ok_or_else(|| {
            anyhow!(
                "{} --project-name is required in non-interactive mode.\n\
                 Usage: bonesdeploy init --non-interactive --project-name <name> --host <host>",
                style("Error:").red().bold(),
            )
        })?;

    let remote_name = args
        .remote
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| seed.and_then(|cfg| non_empty(&cfg.data.remote_name)))
        .unwrap_or_else(|| String::from("production"));

    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };

    let host = args
        .host
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| seed.and_then(|cfg| non_empty(&cfg.data.host)))
        .or_else(|| inferred_remote.as_ref().map(|details| details.host.clone()))
        .ok_or_else(|| {
            anyhow!(
                "{} --host is required in non-interactive mode.\n\
                 Usage: bonesdeploy init --non-interactive --project-name <name> --host <host>",
                style("Error:").red().bold(),
            )
        })?;

    let branch = args
        .branch
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| seed.and_then(|cfg| non_empty(&cfg.data.branch)))
        .unwrap_or_else(|| String::from("main"));

    let port = args
        .port
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| seed.and_then(|cfg| non_empty(&cfg.data.port)))
        .or_else(|| inferred_remote.as_ref().map(|details| details.port.clone()))
        .unwrap_or_else(|| String::from("22"));

    let resolved = NonInteractiveValues { project_name, remote_name, branch, host, port };
    Ok(build_non_interactive_config(resolved, seed, inferred_remote.as_ref()))
}

struct NonInteractiveValues {
    project_name: String,
    remote_name: String,
    branch: String,
    host: String,
    port: String,
}

fn build_non_interactive_config(
    values: NonInteractiveValues,
    seed: Option<&config::BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> config::BonesConfig {
    let NonInteractiveValues { project_name, remote_name, branch, host, port } = values;

    let repo_path = resolve_repo_path(&project_name, seed, inferred_remote);
    let project_root =
        seed_path_override(seed, |cfg| &cfg.data.project_root, &project_name, config::default_project_root_for);
    let web_root = seed_string(seed, |cfg| &cfg.data.web_root, config::default_web_root().as_str());
    let deploy_on_push = seed.is_none_or(|cfg| cfg.data.deploy_on_push);
    let deploy_user = seed_string(seed, |cfg| &cfg.permissions.defaults.deploy_user, "git");
    let service_user = seed_string(seed, |cfg| &cfg.permissions.defaults.service_user, &project_name);
    let group = seed_string(seed, |cfg| &cfg.permissions.defaults.group, "www-data");
    let dir_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.dir_mode, "750");
    let file_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.file_mode, "640");
    let releases_keep = seed.map_or(5, |cfg| cfg.releases.keep.max(1));
    let shared_files = seed
        .map(|cfg| cfg.releases.shared_files.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from(".env")]);
    let shared_dirs = seed
        .map(|cfg| cfg.releases.shared_dirs.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from("storage")]);
    let path_overrides = seed.map_or_else(Vec::new, |cfg| cfg.permissions.paths.clone());

    config::BonesConfig {
        data: config::Data {
            remote_name,
            project_name,
            host,
            port,
            repo_path,
            project_root,
            web_root,
            branch,
            deploy_on_push,
        },
        permissions: config::Permissions {
            defaults: config::PermissionDefaults { deploy_user, service_user, group, dir_mode, file_mode },
            paths: path_overrides,
        },
        releases: config::Releases { keep: releases_keep, shared_files, shared_dirs },
        ssl: seed.map_or_else(config::Ssl::default, |cfg| cfg.ssl.clone()),
    }
}

fn collect(project_name_hint: &str, args: &InitArgs) -> Result<config::BonesConfig> {
    collect_from_seed(project_name_hint, None, args)
}

fn collect_from_seed(
    project_name_hint: &str,
    seed: Option<&config::BonesConfig>,
    args: &InitArgs,
) -> Result<config::BonesConfig> {
    let project_name =
        cli_or_prompt(args.project_name.as_ref(), || prompts::prompt_project_name(project_name_hint, seed))?;
    let branch = cli_or_prompt(args.branch.as_ref(), || prompts::prompt_branch(seed))?;
    let remote_name = cli_or_prompt(args.remote.as_ref(), || prompts::prompt_remote_name(seed))?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host = cli_or_prompt(args.host.as_ref(), || prompts::prompt_host(seed, inferred_remote.as_ref()))?;
    let port = cli_or_prompt(args.port.as_ref(), || prompts::prompt_port(seed, inferred_remote.as_ref()))?;
    let repo_path = resolve_repo_path(&project_name, seed, inferred_remote.as_ref());
    let project_root =
        seed_path_override(seed, |cfg| &cfg.data.project_root, &project_name, config::default_project_root_for);
    let web_root = seed_string(seed, |cfg| &cfg.data.web_root, config::default_web_root().as_str());
    let deploy_on_push = seed.is_none_or(|cfg| cfg.data.deploy_on_push);
    let deploy_user = seed_string(seed, |cfg| &cfg.permissions.defaults.deploy_user, "git");
    let service_user = seed_string(seed, |cfg| &cfg.permissions.defaults.service_user, &project_name);
    let group = seed_string(seed, |cfg| &cfg.permissions.defaults.group, "www-data");
    let dir_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.dir_mode, "750");
    let file_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.file_mode, "640");
    let releases_keep = seed.map_or(5, |cfg| cfg.releases.keep.max(1));
    let shared_files = seed
        .map(|cfg| cfg.releases.shared_files.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from(".env")]);
    let shared_dirs = seed
        .map(|cfg| cfg.releases.shared_dirs.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from("storage")]);
    let path_overrides = seed.map_or_else(Vec::new, |cfg| cfg.permissions.paths.clone());

    Ok(config::BonesConfig {
        data: config::Data {
            remote_name,
            project_name,
            host,
            port,
            repo_path,
            project_root,
            web_root,
            branch,
            deploy_on_push,
        },
        permissions: config::Permissions {
            defaults: config::PermissionDefaults { deploy_user, service_user, group, dir_mode, file_mode },
            paths: path_overrides,
        },
        releases: config::Releases { keep: releases_keep, shared_files, shared_dirs },
        ssl: seed.map_or_else(config::Ssl::default, |cfg| cfg.ssl.clone()),
    })
}

fn cli_or_prompt(cli_value: Option<&String>, prompt: impl FnOnce() -> Result<String>) -> Result<String> {
    match cli_value {
        Some(v) if !v.is_empty() => Ok(v.clone()),
        _ => prompt(),
    }
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

fn seed_string(
    seed: Option<&config::BonesConfig>,
    field: impl Fn(&config::BonesConfig) -> &String,
    fallback: &str,
) -> String {
    seed.map(field).filter(|value| !value.is_empty()).map_or_else(|| fallback.to_string(), Clone::clone)
}

fn resolve_repo_path(
    project_name: &str,
    seed: Option<&config::BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    if let Some(details) = inferred_remote {
        return details.repo_path.clone();
    }

    seed.map(|cfg| cfg.data.repo_path.as_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("/home/git/{project_name}.git"), |value| value.replace("<project_name>", project_name))
}

fn seed_path_override(
    seed: Option<&config::BonesConfig>,
    field: impl Fn(&config::BonesConfig) -> &String,
    current_project_name: &str,
    default_for: fn(&str) -> String,
) -> String {
    let Some(cfg) = seed else { return String::new() };
    let value = field(cfg);
    if value.is_empty() {
        return String::new();
    }

    let resolved = value.replace("<project_name>", current_project_name);
    if resolved == default_for(&cfg.data.project_name) || resolved == default_for(current_project_name) {
        return String::new();
    }
    resolved
}

fn load_or_collect_config(bones_yaml: &Path, args: &InitArgs) -> Result<config::BonesConfig> {
    if bones_yaml.exists() {
        let existing = config::load(bones_yaml)?;
        if config::is_configured(&existing) {
            println!("Loading existing config from {}...", config::Constants::BONES_YAML);
            return Ok(existing);
        }
        println!("Config is incomplete, running prompts...");
        let project_name = repo_directory_name()?;
        if args.non_interactive {
            return collect_non_interactive(&project_name, Some(&existing), args);
        }
        return collect_from_seed(&project_name, Some(&existing), args);
    }

    let project_name = repo_directory_name()?;

    if args.non_interactive {
        return collect_non_interactive(&project_name, None, args);
    }

    collect(&project_name, args)
}

fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(".gitignore");
    let entry = config::Constants::BONES_DIR;

    if gitignore.exists() {
        let content = fs::read_to_string(gitignore)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        fs::write(gitignore, format!("{content}{separator}{entry}\n"))?;
    } else {
        fs::write(gitignore, format!("{entry}\n"))?;
    }

    println!("Added .bones to .gitignore");
    Ok(())
}

pub(crate) fn symlink_pre_push() -> Result<()> {
    let hooks_dir = Path::new(config::Constants::GIT_HOOKS_DIR);
    fs::create_dir_all(hooks_dir)?;

    let link = hooks_dir.join(config::Constants::PRE_PUSH_HOOK);
    let target = Path::new(config::Constants::PRE_PUSH_HOOK_TARGET);

    if link.exists() || link.symlink_metadata().is_ok() {
        fs::remove_file(&link).with_context(|| format!("Failed to remove existing {}", link.display()))?;
    }

    unix_fs::symlink(target, &link).with_context(|| format!("Failed to symlink {}", link.display()))?;

    println!("Symlinked {} -> {}", config::Constants::GIT_PRE_PUSH_HOOK_PATH, config::Constants::PRE_PUSH_HOOK_TARGET);
    Ok(())
}

fn repo_directory_name() -> Result<String> {
    let cwd = env::current_dir()?;
    let name = cwd.file_name().map_or_else(|| "project".into(), |n| n.to_string_lossy().to_string());
    Ok(name)
}

fn ensure_local_remote(cfg: &config::BonesConfig) -> Result<()> {
    if git::remote_exists(&cfg.data.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", cfg.permissions.defaults.deploy_user, cfg.data.host, cfg.data.repo_path);
    git::add_remote(&cfg.data.remote_name, &remote_url)?;
    println!("Configured local git remote {} -> {}", cfg.data.remote_name, remote_url);
    Ok(())
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
