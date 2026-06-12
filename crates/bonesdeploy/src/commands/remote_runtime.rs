use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;

use crate::commands::remote_setup;
use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;
use shared::config::PathOverride;
use shared::paths::{self, DeploymentPaths};

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let available_templates = embedded::available_templates();
    let template_name =
        prompts::choose_template(&available_templates)?.ok_or_else(|| anyhow!("A runtime template is required."))?;
    let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
    let existing_template = runtime_yaml
        .is_file()
        .then(|| config::load_runtime(runtime_yaml))
        .transpose()?
        .as_ref()
        .and_then(runtime_template_name);

    embedded::scaffold_runtime_base(bones_dir)?;
    embedded::scaffold_runtime_template(&template_name, bones_dir)?;

    if existing_template.as_deref() != Some(template_name.as_str()) || !runtime_yaml.is_file() {
        let mut runtime = serde_json::Map::new();
        runtime.insert(String::from("template"), Value::String(template_name.clone()));
        config::save_runtime(&runtime, runtime_yaml)?;
        println!("Saved runtime config to {}", config::Constants::BONES_RUNTIME_YAML);
    } else {
        println!("Keeping existing runtime config at {}", config::Constants::BONES_RUNTIME_YAML);
    }

    apply_template_defaults(&template_name, existing_template.as_deref())?;
    println!("Applied runtime template: {template_name}");

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let ssh_user = String::from(paths::DEPLOY_USER);
    let deploy_file = PathBuf::from(config::Constants::BONES_REMOTE_RUNTIME_DEPLOY);

    remote_setup::ensure_pyinfra_installed()?;
    let data_vars = build_runtime_data_vars(&cfg, runtime_yaml)?;
    remote_setup::run_pyinfra_deploy(
        &cfg,
        &ssh_user,
        &data_vars,
        &remote_setup::PyinfraDeploy { extra_args: &[], deploy_file: &deploy_file },
    )
}

fn build_runtime_data_vars(cfg: &config::BonesConfig, runtime_yaml: &Path) -> Result<Value> {
    let paths =
        DeploymentPaths::new(&cfg.data.project_name, &cfg.data.repo_path, &cfg.data.project_root, &cfg.data.web_root);
    let mut vars = serde_json::Map::new();

    vars.insert(String::from("ssh_port"), Value::String(cfg.data.port.clone()));
    vars.insert(String::from("deploy_user"), Value::String(String::from(paths::DEPLOY_USER)));
    vars.insert(String::from("service_user"), Value::String(config::service_user(&cfg.data.project_name)));
    vars.insert(String::from("group"), Value::String(String::from(paths::DEFAULT_GROUP)));
    vars.insert(String::from("project_root_parent"), Value::String(paths.project_root_parent.clone()));
    vars.insert(String::from("project_root"), Value::String(cfg.data.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(cfg.data.web_root.clone()));
    vars.insert(String::from("project_name"), Value::String(cfg.data.project_name.clone()));
    vars.insert(String::from("repo_path"), Value::String(cfg.data.repo_path.clone()));
    vars.insert(String::from("paths"), serde_json::to_value(paths)?);

    let runtime_data = config::load_runtime(runtime_yaml)?;
    for (key, value) in runtime_data {
        vars.insert(key, value);
    }

    Ok(Value::Object(vars))
}

fn runtime_template_name(runtime: &serde_json::Map<String, Value>) -> Option<String> {
    runtime.get("template").and_then(Value::as_str).map(str::to_string)
}

fn apply_template_defaults(template_name: &str, previous_template: Option<&str>) -> Result<()> {
    let template_cfg = parse_template_runtime_config(template_name)?;
    let previous_cfg = previous_template.map(parse_template_runtime_config).transpose()?;

    apply_web_root_default(
        template_cfg.web_root.as_deref(),
        previous_cfg.as_ref().and_then(|c| c.web_root.as_deref()),
    )?;
    apply_runtime_config(&template_cfg, previous_cfg.as_ref())?;

    Ok(())
}

fn apply_web_root_default(template_web_root: Option<&str>, previous_web_root: Option<&str>) -> Result<()> {
    let Some(template_web_root) = template_web_root.filter(|v| !v.is_empty()) else {
        return Ok(());
    };

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let mut current = config::load(bones_yaml)?;
    let default_web_root = config::default_web_root();

    if current.data.web_root == default_web_root || previous_web_root.is_some_and(|prev| current.data.web_root == prev)
    {
        current.data.web_root = template_web_root.to_string();
        config::save(&current, bones_yaml)?;
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct RuntimeTemplateConfig {
    #[serde(default)]
    web_root: Option<String>,
    #[serde(default)]
    permissions: Option<RuntimePermissions>,
    #[serde(default)]
    shared: Option<RuntimeShared>,
}

#[derive(serde::Deserialize)]
struct RuntimePermissions {
    #[serde(default)]
    defaults: Option<RuntimePermissionDefaults>,
    #[serde(default)]
    paths: Vec<PathOverride>,
}

#[derive(serde::Deserialize)]
struct RuntimePermissionDefaults {
    #[serde(default)]
    dir_mode: Option<String>,
    #[serde(default)]
    file_mode: Option<String>,
}

#[derive(serde::Deserialize)]
struct RuntimeShared {
    #[serde(default)]
    shared_files: Vec<String>,
    #[serde(default)]
    shared_dirs: Vec<String>,
}

fn parse_template_runtime_config(template_name: &str) -> Result<RuntimeTemplateConfig> {
    let content = embedded::read_template_runtime_config(template_name)?;
    let config: RuntimeTemplateConfig = serde_yml::from_str(&content)
        .with_context(|| format!("Failed to parse runtime config for template: {template_name}"))?;
    Ok(config)
}

fn apply_runtime_config(template: &RuntimeTemplateConfig, previous: Option<&RuntimeTemplateConfig>) -> Result<()> {
    let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
    let mut runtime = if runtime_yaml.is_file() { config::load_runtime(runtime_yaml)? } else { serde_json::Map::new() };

    if let Some(ref perms) = template.permissions {
        if let Some(ref defaults) = perms.defaults {
            if let Some(ref dir_mode) = defaults.dir_mode {
                runtime.entry(String::from("dir_mode")).or_insert(Value::String(dir_mode.clone()));
            }
            if let Some(ref file_mode) = defaults.file_mode {
                runtime.entry(String::from("file_mode")).or_insert(Value::String(file_mode.clone()));
            }
        }

        if !perms.paths.is_empty() {
            let current_paths_empty = runtime.get("paths").and_then(|v| v.as_array()).is_none_or(Vec::is_empty);
            let prev_matches = previous.and_then(|p| p.permissions.as_ref()).is_some_and(|p| p.paths == perms.paths);

            if current_paths_empty || prev_matches {
                runtime.insert(String::from("paths"), serde_json::to_value(&perms.paths)?);
            }
        }
    }

    if let Some(ref shared) = template.shared {
        let current_shared_empty = !runtime.contains_key("shared_files") && !runtime.contains_key("shared_dirs");
        let prev_matches = previous
            .and_then(|p| p.shared.as_ref())
            .is_some_and(|s| s.shared_files == shared.shared_files && s.shared_dirs == shared.shared_dirs);

        if current_shared_empty || prev_matches {
            runtime.insert(
                String::from("shared_files"),
                Value::Array(shared.shared_files.iter().map(|v| Value::String(v.clone())).collect()),
            );
            runtime.insert(
                String::from("shared_dirs"),
                Value::Array(shared.shared_dirs.iter().map(|v| Value::String(v.clone())).collect()),
            );
        }
    }

    config::save_runtime(&runtime, runtime_yaml)?;
    Ok(())
}
