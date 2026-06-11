use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};
use serde_json::{Map, Value};

use crate::commands::remote_setup;
use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;

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

    let runtime_dir = Path::new(config::Constants::BONES_RUNTIME_DIR);
    if runtime_dir.exists() {
        fs::remove_dir_all(runtime_dir)?;
    }

    embedded::scaffold_runtime_base(bones_dir)?;
    embedded::scaffold_runtime_template(&template_name, bones_dir)?;

    if existing_template.as_deref() != Some(template_name.as_str()) || !runtime_yaml.is_file() {
        let mut runtime = parse_runtime_defaults(&template_name)?;
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

    let ssh_user = cfg.permissions.defaults.deploy_user.clone();
    let playbook = PathBuf::from(config::Constants::BONES_REMOTE_RUNTIME_PLAYBOOK);
    let roles_dirs = [PathBuf::from(config::Constants::BONES_REMOTE_RUNTIME_ROLES_DIR)];

    remote_setup::ensure_ansible_playbook_installed()?;
    remote_setup::run_ansible_playbook(
        &cfg,
        &ssh_user,
        Value::Null,
        &remote_setup::AnsiblePlaybook { extra_args: &[], playbook: &playbook, roles_dirs: &roles_dirs },
    )
}

fn parse_runtime_defaults(template_name: &str) -> Result<Map<String, Value>> {
    let content = embedded::read_template_runtime_vars(template_name)?;
    let value: Value = serde_yml::from_str(&content)?;
    match value {
        Value::Object(map) => Ok(map),
        _ => bail!("Template runtime vars for {template_name} must contain a YAML object"),
    }
}

fn runtime_template_name(runtime: &Map<String, Value>) -> Option<String> {
    runtime.get("template").and_then(Value::as_str).map(str::to_string)
}

fn apply_template_defaults(template_name: &str, previous_template: Option<&str>) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let mut current = config::load(bones_yaml)?;
    let template = parse_template_config(template_name)?;
    let previous = previous_template.map(parse_template_config).transpose()?;

    let default_web_root = config::default_web_root();
    let template_web_root = template.data.web_root;
    if !template_web_root.is_empty()
        && (current.data.web_root == default_web_root
            || previous.as_ref().is_some_and(|cfg| current.data.web_root == cfg.data.web_root))
    {
        current.data.web_root = template_web_root;
    }

    if !template.permissions.paths.is_empty()
        && (current.permissions.paths.is_empty()
            || previous.as_ref().is_some_and(|cfg| current.permissions.paths == cfg.permissions.paths))
    {
        current.permissions.paths = template.permissions.paths;
    }

    config::save(&current, bones_yaml)?;
    Ok(())
}

fn parse_template_config(template_name: &str) -> Result<config::BonesConfig> {
    let content = embedded::read_template_bones_config(template_name)?;
    let mut config: config::BonesConfig = serde_yml::from_str(&content)?;
    if config.data.project_name.is_empty() {
        config.data.project_name = String::from("template");
    }
    Ok(config)
}
