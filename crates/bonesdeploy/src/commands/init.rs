use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;

use crate::commands::init_config;
pub use crate::commands::init_config::InitArgs;
use crate::commands::remote_setup;
use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;
use crate::python;

pub struct InitOutcome {
    pub remote_setup_ran: bool,
}

pub fn run(args: &InitArgs) -> Result<InitOutcome> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    let is_fresh = !bones_dir.exists();

    let mut initial_project_name: Option<String> = None;

    if is_fresh {
        let project_name = resolve_project_name(args)?;
        let config_dir = config::bones_config_dir(&project_name);

        println!("Creating {}...", config_dir.display());
        fs::create_dir_all(&config_dir)?;
        embedded::scaffold(&config_dir)?;

        unix_fs::symlink(&config_dir, bones_dir)?;
        println!("Symlinked .bones -> {}", config_dir.display());

        let seed = config::BonesConfig {
            data: config::Data { project_name: project_name.clone(), ..Default::default() },
            releases: config::Releases::default(),
            ssl: config::Ssl::default(),
        };
        config::save(&seed, Path::new(config::Constants::BONES_YAML))?;

        initial_project_name = Some(project_name);
    } else {
        println!(".bones/ already exists, skipping scaffold extraction.");
    }

    update_gitignore()?;

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = load_or_collect_config(bones_yaml, args)?;

    if let Some(ref initial) = initial_project_name
        && cfg.data.project_name != *initial
    {
        let old_dir = config::bones_config_dir(initial);
        let new_dir = config::bones_config_dir(&cfg.data.project_name);
        fs::rename(&old_dir, &new_dir)?;
        fs::remove_file(bones_dir)?;
        unix_fs::symlink(&new_dir, bones_dir)?;
        println!("Renamed centralized folder to {}.bones", cfg.data.project_name);
    }

    config::save(&cfg, bones_yaml)?;
    println!("Saved config to {}", config::Constants::BONES_YAML);

    if is_fresh {
        let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
        seed_runtime_config(args, bones_dir, runtime_yaml)?;
    }
    ensure_local_remote(&cfg)?;

    symlink_pre_push()?;

    let remote_setup_ran = args.setup_remote || (!args.non_interactive && prompts::confirm_remote_setup()?);
    if remote_setup_ran {
        remote_setup::run()?;
    } else {
        print_follow_up_hint();
    }

    Ok(InitOutcome { remote_setup_ran })
}

fn print_follow_up_hint() {
    println!();
    println!("{}", style("Next:").cyan().bold());
    println!("Run {} to sync {} to the remote.", style("bonesdeploy push").cyan(), style(".bones/").cyan());
}

fn seed_runtime_config(args: &InitArgs, _bones_dir: &Path, runtime_yaml: &Path) -> Result<()> {
    let available = python::list_runtimes()?;
    let template = if args.non_interactive { None } else { prompts::choose_template(&available)? };

    if let Some(ref template_name) = template {
        let defaults = python::runtime_defaults(template_name)?;
        let yaml = serde_yml::to_string(&defaults).context("Failed to serialize runtime defaults")?;
        fs::write(runtime_yaml, yaml)?;
        println!("Applied runtime template: {template_name}");
        println!("Saved runtime config to {}", config::Constants::BONES_RUNTIME_YAML);
    } else {
        let empty = serde_json::Map::new();
        config::save_runtime(&empty, runtime_yaml)?;
        println!("Seeded {} from kit defaults", config::Constants::BONES_RUNTIME_YAML);
    }

    Ok(())
}

fn collect(project_name_hint: &str, args: &InitArgs) -> Result<config::BonesConfig> {
    collect_from_seed(project_name_hint, None, args)
}

fn collect_from_seed(
    project_name_hint: &str,
    existing_config: Option<&config::BonesConfig>,
    args: &InitArgs,
) -> Result<config::BonesConfig> {
    let project_name = cli_existing_or_prompt(
        args.project_name.as_ref(),
        existing_config.and_then(|cfg| init_config::non_empty(&cfg.data.project_name)),
        || prompts::prompt_project_name(project_name_hint, existing_config),
    )?;
    let branch = cli_or_prompt(args.branch.as_ref(), || prompts::prompt_branch(existing_config))?;
    let remote_name = cli_or_prompt(args.remote.as_ref(), || prompts::prompt_remote_name(existing_config))?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host = cli_or_prompt(args.host.as_ref(), || prompts::prompt_host(existing_config, inferred_remote.as_ref()))?;
    let port = cli_or_prompt(args.port.as_ref(), || prompts::prompt_port(existing_config, inferred_remote.as_ref()))?;
    let repo_path = init_config::resolve_repo_path(&project_name, existing_config, inferred_remote.as_ref());
    let project_root = init_config::seed_path_override(
        existing_config,
        |cfg| &cfg.data.project_root,
        &project_name,
        config::default_project_root_for,
    );
    let web_root =
        init_config::seed_string(existing_config, |cfg| &cfg.data.web_root, config::default_web_root().as_str());
    let deploy_on_push = existing_config.is_none_or(|cfg| cfg.data.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases.keep.max(1));

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
            ..Default::default()
        },
        releases: config::Releases { keep: releases_keep },
        ssl: existing_config.map_or_else(config::Ssl::default, |cfg| cfg.ssl.clone()),
    })
}

#[cfg(test)]
fn collect_non_interactive(
    project_name_hint: &str,
    existing_config: Option<&config::BonesConfig>,
    args: &InitArgs,
) -> Result<config::BonesConfig> {
    init_config::collect_non_interactive(project_name_hint, existing_config, args)
}

fn resolve_project_name(args: &InitArgs) -> Result<String> {
    if let Some(name) = args.project_name.as_ref().filter(|v| !v.is_empty()) {
        return Ok(name.trim().to_string());
    }
    if args.non_interactive {
        bail!(
            "--project-name is required in non-interactive mode.\nUsage: bonesdeploy init --non-interactive --project-name <name> --host <host>"
        );
    }
    let hint = config::repo_directory_name()?;
    prompts::prompt_project_name(&hint, None)
}

fn cli_or_prompt(cli_value: Option<&String>, prompt: impl FnOnce() -> Result<String>) -> Result<String> {
    match cli_value {
        Some(v) if !v.is_empty() => Ok(v.trim().to_string()),
        _ => prompt(),
    }
}

fn cli_existing_or_prompt(
    cli_value: Option<&String>,
    existing_value: Option<String>,
    prompt: impl FnOnce() -> Result<String>,
) -> Result<String> {
    match cli_value {
        Some(v) if !v.is_empty() => Ok(v.trim().to_string()),
        _ => existing_value.map_or_else(prompt, Ok),
    }
}

fn load_or_collect_config(bones_yaml: &Path, args: &InitArgs) -> Result<config::BonesConfig> {
    if bones_yaml.exists() {
        let existing = config::load(bones_yaml)?;
        if config::is_configured(&existing) {
            println!("Loading existing config from {}...", config::Constants::BONES_YAML);
            return Ok(existing);
        }
        let project_name = config::repo_directory_name()?;
        if args.non_interactive {
            return init_config::collect_non_interactive(&project_name, Some(&existing), args);
        }
        return collect_from_seed(&project_name, Some(&existing), args);
    }

    let project_name = config::repo_directory_name()?;

    if args.non_interactive {
        return init_config::collect_non_interactive(&project_name, None, args);
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

fn ensure_local_remote(cfg: &config::BonesConfig) -> Result<()> {
    if git::remote_exists(&cfg.data.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", cfg.data.deploy_user, cfg.data.host, cfg.data.repo_path);
    git::add_remote(&cfg.data.remote_name, &remote_url)?;
    println!("Configured local git remote {} -> {}", cfg.data.remote_name, remote_url);
    Ok(())
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
