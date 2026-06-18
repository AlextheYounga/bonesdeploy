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
use shared::config::{default_deploy_user, runtime_group_for, runtime_user_for, release_group_for};

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
        if config_dir.exists() && !config_dir.is_dir() {
            fs::remove_file(&config_dir)
                .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
        }
        fs::create_dir_all(&config_dir)?;
        embedded::scaffold(&config_dir)?;

        unix_fs::symlink(&config_dir, bones_dir)?;
        println!("Symlinked .bones -> {}", config_dir.display());

        let seed = config::Bones { project_name: project_name.clone(), ..Default::default() };
        config::save(&seed, Path::new(config::Constants::BONES_TOML))?;

        initial_project_name = Some(project_name);
    } else {
        println!(".bones/ already exists, keeping existing local bones state.");
    }

    update_gitignore()?;

    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = load_or_collect_config(bones_toml, args)?;

    if let Some(ref initial) = initial_project_name
        && cfg.project_name != *initial
    {
        let old_dir = config::bones_config_dir(initial);
        let new_dir = config::bones_config_dir(&cfg.project_name);
        fs::rename(&old_dir, &new_dir)?;
        fs::remove_file(bones_dir)?;
        unix_fs::symlink(&new_dir, bones_dir)?;
        println!("Renamed centralized folder to {}.bones", cfg.project_name);
    }

    config::save(&cfg, bones_toml)?;
    println!("Saved config to {}", config::Constants::BONES_TOML);

    if is_fresh {
        let runtime_toml = Path::new(config::Constants::BONES_RUNTIME_TOML);
        seed_runtime_config(args, &cfg.project_name, bones_dir, runtime_toml)?;
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

fn seed_runtime_config(args: &InitArgs, project_name: &str, _bones_dir: &Path, runtime_toml: &Path) -> Result<()> {
    let template = if args.non_interactive {
        None
    } else {
        let available = python::list_runtimes()?;
        prompts::choose_template(&available)?
    };

    if let Some(ref template_name) = template {
        let defaults = python::runtime_defaults(template_name)?;
        let answers = if args.non_interactive {
            defaults
        } else {
            let questions = python::runtime_questions(template_name)?;
            prompts::prompt_runtime_questions(&questions, &defaults)?
        };
        let mut map = answers.as_object().cloned().unwrap_or_default();
        inject_runtime_identity(&mut map, project_name);
        let toml_str = toml::to_string(&map).context("Failed to serialize runtime config")?;
        fs::write(runtime_toml, toml_str)?;
        println!("Applied runtime template: {template_name}");
        println!("Saved runtime config to {}", config::Constants::BONES_RUNTIME_TOML);
    } else {
        let mut vars = serde_json::Map::new();
        inject_runtime_identity(&mut vars, project_name);
        config::save_runtime(&vars, runtime_toml)?;
        println!("Seeded {} from kit defaults", config::Constants::BONES_RUNTIME_TOML);
    }

    Ok(())
}

fn inject_runtime_identity(vars: &mut serde_json::Map<String, serde_json::Value>, project_name: &str) {
    vars.insert(
        "runtime_user".into(),
        serde_json::Value::String(runtime_user_for(project_name)),
    );
    vars.insert(
        "runtime_group".into(),
        serde_json::Value::String(runtime_group_for(project_name)),
    );
    vars.insert(
        "release_group".into(),
        serde_json::Value::String(release_group_for(project_name)),
    );
}

fn collect(project_name_hint: &str, args: &InitArgs) -> Result<config::Bones> {
    collect_from_seed(project_name_hint, None, args)
}

fn collect_from_seed(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
    let project_name = cli_existing_or_prompt(
        args.project_name.as_ref(),
        existing_config.and_then(|cfg| init_config::non_empty(&cfg.project_name)),
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
        |cfg| &cfg.project_root,
        &project_name,
        config::default_project_root_for,
    );
    let deploy_on_push = existing_config.is_none_or(|cfg| cfg.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));

    let ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    let domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    let email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());

    Ok(config::Bones {
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
    })
}

#[cfg(test)]
fn collect_non_interactive(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
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

fn load_or_collect_config(bones_toml: &Path, args: &InitArgs) -> Result<config::Bones> {
    if bones_toml.exists() {
        let existing = config::load(bones_toml)?;
        if config::is_configured(&existing) {
            println!("Loading existing config from {}...", config::Constants::BONES_TOML);
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

    if fs::symlink_metadata(&link).is_ok() {
        fs::remove_file(&link).with_context(|| format!("Failed to remove existing {}", link.display()))?;
    }

    unix_fs::symlink(target, &link).with_context(|| format!("Failed to symlink {}", link.display()))?;

    println!("Symlinked {} -> {}", config::Constants::GIT_PRE_PUSH_HOOK_PATH, config::Constants::PRE_PUSH_HOOK_TARGET);
    Ok(())
}

fn ensure_local_remote(cfg: &config::Bones) -> Result<()> {
    if git::remote_exists(&cfg.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", default_deploy_user(), cfg.host, cfg.repo_path);
    git::add_remote(&cfg.remote_name, &remote_url)?;
    println!("Configured local git remote {} -> {}", cfg.remote_name, remote_url);
    Ok(())
}

#[cfg(test)]
#[path = "tests/test_init_project.rs"]
mod test_init_project;


