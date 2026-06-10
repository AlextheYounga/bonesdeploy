use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::commands::remote_setup::resolve_bootstrap_ssh_user;
use crate::config;
use crate::update_assets;

pub fn current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn current_remote_version() -> String {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        return String::from("unknown");
    }

    let Ok(cfg) = config::load(bones_yaml) else {
        return String::from("unknown");
    };

    let host = format!("{}@{}", cfg.permissions.defaults.deploy_user, config::resolve_host(&cfg).unwrap_or_default());
    let output = Command::new("ssh").args(["-p", &cfg.data.port]).args([&host, "bonesremote", "version"]).output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().strip_prefix("bonesremote ").unwrap_or("unknown").to_string()
        }
        _ => String::from("unknown"),
    }
}

pub fn update_local_from_source(repo_url: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(["install", "--git", repo_url, paths::BONESDEPLOY_BINARY, "--force"])
        .status()
        .context("Failed to run cargo install for bonesdeploy")?;

    if !status.success() {
        bail!("Failed to install bonesdeploy from {repo_url}");
    }

    Ok(())
}

pub fn update_remote_from_source(repo_url: &str, version: &str) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        bail!("No .bones/bones.yaml found. Run from a bonesdeploy project directory.");
    }

    let cfg = config::load(bones_yaml)?;

    let ansible_temp = tempfile::TempDir::new().context("Failed to create Ansible temp directory")?;
    let playbook_path = update_assets::materialize_playbook(ansible_temp.path())?;

    println!("Running remote update playbook...");
    run_update_playbook(&cfg, &playbook_path, repo_url, version)?;

    Ok(())
}

pub fn run_update_playbook(cfg: &config::BonesConfig, playbook: &Path, repo_url: &str, version: &str) -> Result<()> {
    let roles_dir = playbook
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("roles"))
        .ok_or_else(|| anyhow::anyhow!("Invalid playbook path structure"))?;

    let inventory = format!("{},", config::resolve_host(cfg)?);
    let ssh_user = resolve_bootstrap_ssh_user();

    let ansible_playbook = resolve_ansible_playbook()?;

    let mut command = Command::new(&ansible_playbook);
    command
        .env("ANSIBLE_ROLES_PATH", roles_dir.display().to_string())
        .arg("-i")
        .arg(&inventory)
        .arg("-u")
        .arg(&ssh_user)
        .arg("-e")
        .arg(format!("ansible_port={}", cfg.data.port))
        .arg("-e")
        .arg(format!("bonesremote_repo_url={repo_url}"))
        .arg("-e")
        .arg(format!("bonesremote_target_version={version}"))
        .arg("-e")
        .arg(format!("bonesremote_install_root={}", paths::USR_LOCAL_BIN.trim_end_matches("/bin")))
        .arg("-e")
        .arg(format!("bonesremote_binary_path={}", paths::bonesremote_global_link().display()))
        .arg("-e")
        .arg(format!("bonesremote_managed_projects_root={}", paths::DEFAULT_PROJECT_ROOT_PARENT))
        .arg(playbook);

    println!("Running: {command:?}");

    let status = command.status().context("Failed to run ansible-playbook")?;

    if !status.success() {
        bail!("Remote update playbook failed with status {status}");
    }

    Ok(())
}

fn resolve_ansible_playbook() -> Result<PathBuf> {
    if ansible_playbook_available(Path::new("ansible-playbook")) {
        return Ok(PathBuf::from("ansible-playbook"));
    }

    let home = env::var("HOME").context("HOME is not set")?;
    let local_ansible = Path::new(&home).join(".local/bin/ansible-playbook");

    if ansible_playbook_available(&local_ansible) {
        return Ok(local_ansible);
    }

    bail!("ansible-playbook not found. Install Ansible first.");
}

fn ansible_playbook_available(binary: &Path) -> bool {
    Command::new(binary).arg("--version").status().is_ok_and(|s| s.success())
}
