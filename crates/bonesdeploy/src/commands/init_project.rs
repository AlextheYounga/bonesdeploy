use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use crate::commands::init_config;
pub use crate::commands::init_config::InitArgs;
use crate::commands::remote_setup;
use crate::config;
use crate::infra::bonesinfra;
use crate::infra::bonesinfra_cli;
use crate::infra::embedded;
use crate::infra::git;
use crate::ui::prompts;
use anyhow::{Context, Result, bail};
use shared::config::{bonesinfra_input, default_deploy_user, release_group_for, runtime_group_for, runtime_user_for};
use shared::paths;

pub fn run(args: &InitArgs) -> Result<bool> {
    run_with_prefetch(args, bonesinfra::prefetch)
}

fn run_with_prefetch(args: &InitArgs, prefetch_bonesinfra: impl FnOnce() -> Result<()>) -> Result<bool> {
    git::ensure_git_repository()?;

    println!("Initializing bonesdeploy...");
    prefetch_bonesinfra()?;

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    let had_bones_entry = fs::symlink_metadata(bones_dir).is_ok();
    let has_live_bones_dir = bones_dir.exists();
    let is_fresh = !has_live_bones_dir;

    let mut initial_project_name: Option<String> = None;

    if is_fresh {
        let project_name = resolve_project_name(args)?;
        let config_dir = config::bones_config_dir(&project_name);

        if config_dir.exists() && !config_dir.is_dir() {
            fs::remove_file(&config_dir)
                .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
        }
        fs::create_dir_all(&config_dir)?;
        embedded::scaffold(&config_dir)?;

        if had_bones_entry {
            fs::remove_file(bones_dir)
                .with_context(|| format!("Failed to remove stale {} symlink", bones_dir.display()))?;
        }
        unix_fs::symlink(&config_dir, bones_dir)?;

        let existing = config::Bones { project_name: project_name.clone(), ..Default::default() };
        config::save(&existing, Path::new(paths::LOCAL_BONES_TOML))?;

        initial_project_name = Some(project_name);
    } else {
        println!("Using existing .bones config.");
    }

    update_gitignore()?;

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = load_or_collect_config(bones_toml, args)?;
    ensure_config_gitignore(&cfg.project_name)?;

    if let Some(ref initial) = initial_project_name
        && cfg.project_name != *initial
    {
        let old_dir = config::bones_config_dir(initial);
        let new_dir = config::bones_config_dir(&cfg.project_name);
        fs::rename(&old_dir, &new_dir)?;
        fs::remove_file(bones_dir)?;
        unix_fs::symlink(&new_dir, bones_dir)?;
    }

    config::save(&cfg, bones_toml)?;

    if is_fresh {
        println!("bonesdeploy initialized.");
    } else {
        println!("bonesdeploy config updated.");
    }

    if is_fresh {
        let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
        existing_runtime_config(args, &cfg.project_name, bones_dir, runtime_toml)?;
    }
    ensure_local_remote(&cfg)?;

    symlink_pre_push()?;

    let remote_setup_ran = args.setup_remote || (!args.non_interactive && prompts::confirm_remote_setup()?);
    if remote_setup_ran {
        remote_setup::run()?;
    } else {
        print_follow_up_hint();
    }

    Ok(remote_setup_ran)
}

fn print_follow_up_hint() {
    println!();
    println!("Next: run bonesdeploy setup.");
}

fn existing_runtime_config(args: &InitArgs, project_name: &str, bones_dir: &Path, runtime_toml: &Path) -> Result<()> {
    let template = if args.non_interactive {
        None
    } else {
        let available = embedded::runtime_names();
        prompts::choose_template(&available)?
    };

    if let Some(ref template_name) = template {
        let defaults = embedded::runtime_defaults(template_name)?;
        let answers = if args.non_interactive {
            serde_json::Value::Object(defaults.clone())
        } else {
            let questions = bonesinfra_cli::runtime_questions(template_name)?;
            prompts::prompt_runtime_questions(&questions, &serde_json::Value::Object(defaults.clone()))?
        };
        let mut map = answers.as_object().cloned().unwrap_or(defaults);
        inject_runtime_identity(&mut map, project_name);
        config::save_runtime(&map, runtime_toml)?;
        embedded::scaffold_runtime_deployment(template_name, bones_dir)?;
        embedded::scaffold_runtime_secrets(template_name, bones_dir)?;
        println!("Runtime template: {template_name}");
    } else {
        let mut vars = embedded::base_runtime_defaults()?;
        inject_runtime_identity(&mut vars, project_name);
        config::save_runtime(&vars, runtime_toml)?;
        println!("Runtime template: custom");
    }

    Ok(())
}

fn inject_runtime_identity(vars: &mut serde_json::Map<String, serde_json::Value>, project_name: &str) {
    vars.insert(bonesinfra_input::RUNTIME_USER.into(), serde_json::Value::String(runtime_user_for(project_name)));
    vars.insert(bonesinfra_input::RUNTIME_GROUP.into(), serde_json::Value::String(runtime_group_for(project_name)));
    vars.insert(bonesinfra_input::RELEASE_GROUP.into(), serde_json::Value::String(release_group_for(project_name)));
}

fn collect_from_existing(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
    let project_name = cli_or_prompt(
        args.project_name.as_ref(),
        existing_config.and_then(|cfg| init_config::non_empty(&cfg.project_name)),
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
    let repo_path = init_config::resolve_repo_path(&project_name, existing_config, inferred_remote.as_ref());
    let project_root = init_config::existing_path_override(
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

fn load_or_collect_config(bones_toml: &Path, args: &InitArgs) -> Result<config::Bones> {
    if bones_toml.exists() {
        let existing = config::load(bones_toml)?;
        if config::is_configured(&existing) {
            return Ok(existing);
        }
        let project_name = config::repo_directory_name()?;
        if args.non_interactive {
            return init_config::collect_non_interactive(&project_name, Some(&existing), args);
        }
        return collect_from_existing(&project_name, Some(&existing), args);
    }

    let project_name = config::repo_directory_name()?;

    if args.non_interactive {
        return init_config::collect_non_interactive(&project_name, None, args);
    }

    collect_from_existing(&project_name, None, args)
}

fn ensure_config_gitignore(project_name: &str) -> Result<()> {
    let gitignore = paths::bones_config_root().join(".gitignore");
    let project_entry = format!("{project_name}.bones");

    if gitignore.exists() {
        let content = fs::read_to_string(&gitignore)?;
        let mut missing = Vec::new();
        for entry in [paths::BONES_CONFIG_LIB_DIR, &project_entry] {
            if !content.lines().any(|line| line.trim() == entry) {
                missing.push(entry);
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        let mut append = String::new();
        for entry in &missing {
            append.push_str(entry);
            append.push('\n');
        }
        fs::write(&gitignore, format!("{content}{separator}{append}"))?;
    } else {
        let mut content = String::new();
        for entry in [paths::BONES_CONFIG_LIB_DIR, &project_entry] {
            content.push_str(entry);
            content.push('\n');
        }
        fs::write(&gitignore, content)?;
    }

    Ok(())
}

fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(".gitignore");
    let entry = paths::LOCAL_BONES_DIR;

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

    Ok(())
}

pub(crate) fn symlink_pre_push() -> Result<()> {
    let hooks_dir = Path::new(paths::GIT_HOOKS_DIR);
    fs::create_dir_all(hooks_dir)?;

    let link = hooks_dir.join(paths::PRE_PUSH_HOOK_NAME);
    let target = Path::new(paths::PRE_PUSH_HOOK_TARGET);

    if fs::symlink_metadata(&link).is_ok() {
        fs::remove_file(&link).with_context(|| format!("Failed to remove existing {}", link.display()))?;
    }

    unix_fs::symlink(target, &link).with_context(|| format!("Failed to symlink {}", link.display()))?;

    Ok(())
}

fn ensure_local_remote(cfg: &config::Bones) -> Result<()> {
    if git::remote_exists(&cfg.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", default_deploy_user(), cfg.host, cfg.repo_path);
    git::add_remote(&cfg.remote_name, &remote_url)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/test_init_project.rs"]
mod test_init_project;
