use std::env;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result};
use console::style;

use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    // Extract scaffold to .bones/
    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if bones_dir.exists() {
        println!(".bones/ already exists, skipping scaffold extraction.");
    } else {
        let available_templates = embedded::available_templates();
        let selected_template = prompts::choose_template(&available_templates)?;

        println!("Creating .bones/ scaffold...");
        embedded::scaffold(bones_dir)?;

        if let Some(template_name) = selected_template {
            embedded::scaffold_template(&template_name, bones_dir)?;
            println!("Applied template: {template_name}");
        } else {
            println!("Using build-from-scratch scaffold.");
        }
    }

    // Update .gitignore
    update_gitignore()?;

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = load_or_collect_config(bones_yaml)?;

    // Save config
    config::save(&cfg, bones_yaml)?;
    println!("Saved config to {}", config::Constants::BONES_YAML);
    ensure_local_remote(&cfg)?;

    // Symlink pre-push hook
    symlink_pre_push()?;

    println!(
        "\n{} Run {} before your first deploy.",
        style("Next:").cyan().bold(),
        style("bonesdeploy server setup").cyan()
    );
    println!(
        "{} Run {} after setup to sync .bones/ to the remote.",
        style("Done!").green().bold(),
        style("bonesdeploy push").cyan()
    );

    Ok(())
}

fn collect(project_name_hint: &str) -> Result<config::BonesConfig> {
    collect_from_seed(project_name_hint, None)
}

fn collect_from_seed(project_name_hint: &str, seed: Option<&config::BonesConfig>) -> Result<config::BonesConfig> {
    let project_name = prompts::prompt_project_name(project_name_hint, seed)?;
    let branch = prompts::prompt_branch(seed)?;
    let remote_name = prompts::prompt_remote_name(seed)?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host = prompts::prompt_host(seed, inferred_remote.as_ref())?;
    let port = prompts::prompt_port(seed, inferred_remote.as_ref())?;
    let git_dir = resolve_git_dir(&project_name, seed, inferred_remote.as_ref());
    // live_root and deploy_root are intentionally not collected here. They are
    // hidden from the init flow and resolved to project-derived defaults at
    // load time. If a previous bones.yaml carried a user override, pass it
    // through verbatim so it survives re-init.
    let live_root = seed_path_override(seed, |cfg| &cfg.data.live_root, &project_name, config::default_live_root_for);
    let deploy_root =
        seed_path_override(seed, |cfg| &cfg.data.deploy_root, &project_name, config::default_deploy_root_for);
    let deploy_on_push = seed.is_none_or(|cfg| cfg.data.deploy_on_push);
    let deploy_user = seed_string(seed, |cfg| &cfg.permissions.defaults.deploy_user, "git");
    let service_user = seed_string(seed, |cfg| &cfg.permissions.defaults.service_user, &project_name);
    let group = seed_string(seed, |cfg| &cfg.permissions.defaults.group, "www-data");
    let dir_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.dir_mode, "750");
    let file_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.file_mode, "640");
    let releases_keep = seed.map_or(5, |cfg| cfg.releases.keep.max(1));
    let shared_paths = seed
        .map(|cfg| cfg.releases.shared_paths.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from(".env"), String::from("storage")]);
    let path_overrides = seed.map_or_else(Vec::new, |cfg| cfg.permissions.paths.clone());

    Ok(config::BonesConfig {
        data: config::Data {
            remote_name,
            project_name,
            host,
            port,
            git_dir,
            live_root,
            deploy_root,
            branch,
            deploy_on_push,
        },
        permissions: config::Permissions {
            defaults: config::PermissionDefaults { deploy_user, service_user, group, dir_mode, file_mode },
            paths: path_overrides,
        },
        releases: config::Releases { keep: releases_keep, shared_paths },
        runtime: seed.map_or_else(config::Runtime::default, |cfg| cfg.runtime.clone()),
        ssl: seed.map_or_else(config::Ssl::default, |cfg| cfg.ssl.clone()),
    })
}

fn seed_string(
    seed: Option<&config::BonesConfig>,
    field: impl Fn(&config::BonesConfig) -> &String,
    fallback: &str,
) -> String {
    seed.map(field).filter(|value| !value.is_empty()).map_or_else(|| fallback.to_string(), Clone::clone)
}

fn resolve_git_dir(
    project_name: &str,
    seed: Option<&config::BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    if let Some(details) = inferred_remote {
        return details.git_dir.clone();
    }

    seed.map(|cfg| cfg.data.git_dir.as_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("/home/git/{project_name}.git"), |value| value.replace("<project_name>", project_name))
}

// Returns the seed's path override only when it differs from the project-derived
// default at the time the seed was loaded. Empty result means "no override" —
// save() will then omit the field from bones.yaml.
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

fn load_or_collect_config(bones_yaml: &Path) -> Result<config::BonesConfig> {
    if bones_yaml.exists() {
        let existing = config::load(bones_yaml)?;
        if config::is_configured(&existing) {
            println!("Loading existing config from {}...", config::Constants::BONES_YAML);
            return Ok(existing);
        }
        println!("Config is incomplete, running prompts...");
        let project_name = repo_directory_name()?;
        return collect_from_seed(&project_name, Some(&existing));
    }
    let project_name = repo_directory_name()?;
    collect(&project_name)
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

fn symlink_pre_push() -> Result<()> {
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

    let remote_url = format!("{}@{}:{}", cfg.permissions.defaults.deploy_user, cfg.data.host, cfg.data.git_dir);
    git::add_remote(&cfg.data.remote_name, &remote_url)?;
    println!("Added git remote {} -> {}", cfg.data.remote_name, remote_url);
    Ok(())
}
