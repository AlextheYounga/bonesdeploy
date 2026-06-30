use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use crate::commands::init_config;
pub use crate::commands::init_config::InitArgs;
use crate::config;
use crate::infra::bonesinfra;
use crate::infra::embedded;
use crate::infra::git;
use crate::ui::prompts;
use anyhow::{Context, Result};
use shared::config::{bonesinfra_input, default_deploy_user, release_group_for, runtime_group_for, runtime_user_for};
use shared::paths;

struct RuntimeSelection {
    template: Option<String>,
    config: serde_json::Map<String, serde_json::Value>,
}

pub fn run(args: &InitArgs) -> Result<()> {
    run_with_prefetch(args, bonesinfra::prefetch)
}

fn run_with_prefetch(args: &InitArgs, prefetch_bonesinfra: impl FnOnce() -> Result<()>) -> Result<()> {
    git::ensure_git_repository()?;

    println!("Initializing bonesdeploy...");
    prefetch_bonesinfra()?; // Allows us to skip this step in tests. 

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    let had_bones_entry = fs::symlink_metadata(bones_dir).is_ok();
    let is_fresh = !bones_dir.exists();
    if !is_fresh {
        println!("Using existing .bones config.");
    }

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = if is_fresh { collect_fresh_config(args)? } else { load_or_collect_config(bones_toml, args)? };
    let runtime_selection = if is_fresh { Some(collect_runtime_config(args, &cfg.project_name)?) } else { None };

    if let Some(runtime) = runtime_selection {
        materialize_fresh_bones(bones_dir, had_bones_entry, &cfg, runtime)?;
    }

    update_gitignore()?;
    ensure_config_gitignore(&cfg.project_name)?;
    config::save(&cfg, bones_toml)?;

    if is_fresh {
        println!("bonesdeploy initialized.");
    } else {
        println!("bonesdeploy config updated.");
    }

    ensure_local_remote(&cfg)?;

    symlink_pre_push()?;

    print_follow_up_hint();

    Ok(())
}

fn print_follow_up_hint() {
    println!();
    println!("Next: run `bonesdeploy setup` to setup the remote server.");
}

fn collect_fresh_config(args: &InitArgs) -> Result<config::Bones> {
    let project_name = config::repo_directory_name()?;

    if args.non_interactive {
        return init_config::collect_non_interactive(&project_name, None, args);
    }

    collect_from_existing(&project_name, None, args)
}

fn collect_runtime_config(args: &InitArgs, project_name: &str) -> Result<RuntimeSelection> {
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
            let questions = bonesinfra::runtime_questions(template_name)?;
            prompts::prompt_runtime_questions(&questions, &serde_json::Value::Object(defaults.clone()))?
        };
        let mut map = answers.as_object().cloned().unwrap_or(defaults);
        inject_runtime_identity(&mut map, project_name);
        Ok(RuntimeSelection { template: Some(template_name.clone()), config: map })
    } else {
        let mut vars = embedded::base_runtime_defaults()?;
        inject_runtime_identity(&mut vars, project_name);
        Ok(RuntimeSelection { template: None, config: vars })
    }
}

fn materialize_fresh_bones(
    bones_dir: &Path,
    had_bones_entry: bool,
    cfg: &config::Bones,
    runtime: RuntimeSelection,
) -> Result<()> {
    let config_dir = config::bones_config_dir(&cfg.project_name);

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

    let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
    config::save_runtime(&runtime.config, runtime_toml)?;

    if let Some(template_name) = runtime.template {
        embedded::scaffold_runtime_deployment(&template_name, bones_dir)?;
        embedded::scaffold_runtime_secrets(&template_name, bones_dir)?;
        println!("Runtime template: {template_name}");
    } else {
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
    let deploy_on_push = existing_config.map_or(false, |cfg| cfg.deploy_on_push);
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
