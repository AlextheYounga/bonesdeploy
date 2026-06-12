use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde_json::Value;

use crate::commands::remote_setup;
use crate::config;
use crate::git;
use crate::prompts;
use shared::paths::{self, DeploymentPaths};

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
    if !runtime_yaml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", config::Constants::BONES_RUNTIME_YAML);
    }

    if !prompts::confirm_remote_runtime()? {
        println!("Skipped remote runtime apply.");
        return Ok(());
    }

    let ssh_user = remote_setup::resolve_bootstrap_ssh_user();
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
    vars.insert(String::from("service_group"), Value::String(String::from(paths::DEFAULT_GROUP)));
    vars.insert(String::from("project_root_parent"), Value::String(paths.project_root_parent.clone()));
    vars.insert(String::from("project_root"), Value::String(cfg.data.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(cfg.data.web_root.clone()));
    vars.insert(String::from("project_name"), Value::String(cfg.data.project_name.clone()));
    vars.insert(String::from("repo_path"), Value::String(cfg.data.repo_path.clone()));
    vars.insert(String::from("paths"), serde_json::to_value(paths)?);

    let runtime_data = config::load_runtime(runtime_yaml)?;
    for (key, value) in runtime_data {
        if key == "paths" {
            continue;
        }
        vars.insert(key, value);
    }

    Ok(Value::Object(vars))
}
