use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::commands::remote_setup::{PyinfraDeploy, resolve_bootstrap_ssh_user, run_pyinfra_deploy};
use crate::config;

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

    let host = format!("{}@{}", cfg.permissions.defaults.deploy_user, cfg.data.host);
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

    let temp = tempfile::TempDir::new().context("Failed to create temp directory")?;
    let deploy_path = write_update_deploy_file(temp.path(), repo_url, version)?;

    println!("Running remote update deploy...");
    let ssh_user = resolve_bootstrap_ssh_user();
    let data_vars = serde_json::json!({
        "ssh_port": cfg.data.port,
        "bonesremote_install_root": paths::USR_LOCAL_BIN.trim_end_matches("/bin"),
        "bonesremote_binary_path": paths::bonesremote_global_link().display().to_string(),
        "bonesremote_managed_projects_root": paths::DEFAULT_PROJECT_ROOT_PARENT,
    });

    run_pyinfra_deploy(&cfg, &ssh_user, &data_vars, &PyinfraDeploy { extra_args: &[], deploy_file: &deploy_path })?;

    Ok(())
}

fn write_update_deploy_file(dir: &Path, repo_url: &str, _version: &str) -> Result<PathBuf> {
    let path = dir.join("update_bonesremote.py");
    let content = format!(
        r#"from pyinfra.operations import server
from pyinfra import host

server.shell(
    name="Build and install bonesremote from source",
    commands=[
        "cargo install --git {repo_url} bonesremote --force --root {{install_root}}",
    ],
    _env={{
        "install_root": host.data.bonesremote_install_root,
    }},
    _sudo=True,
)

server.shell(
    name="Symlink bonesremote into /usr/local/bin",
    commands=[
        "ln -sf {{binary_path}} /usr/local/bin/bonesremote",
    ],
    _if=host.data.bonesremote_binary_path,
    _sudo=True,
)

server.shell(
    name="Ensure managed projects root exists",
    commands=[
        "mkdir -p {{managed_root}}",
        "chown root:root {{managed_root}}",
        "chmod 711 {{managed_root}}",
    ],
    _sudo=True,
)
"#
    );
    fs::write(&path, content).context("Failed to write update deploy file")?;
    Ok(path)
}
