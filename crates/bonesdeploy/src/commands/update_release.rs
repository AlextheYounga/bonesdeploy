use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::config;
use crate::infra::ssh;
use shared::config::{default_deploy_user, parse_port};

pub fn current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn current_remote_version() -> String {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        return String::from("unknown");
    }

    let Ok(cfg) = config::load(bones_toml) else {
        return String::from("unknown");
    };

    let host = format!("{}@{}", default_deploy_user(), cfg.host);
    let output = Command::new("ssh").args(["-p", &cfg.port]).args([&host, "bonesremote", "version"]).output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().strip_prefix("bonesremote ").unwrap_or("unknown").to_string()
        }
        _ => String::from("unknown"),
    }
}

pub fn update_local_from_source(repo_url: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(["install", "--locked", "--git", repo_url, paths::BONESDEPLOY_BINARY, "--force"])
        .status()
        .context("Failed to run cargo install for bonesdeploy")?;

    if !status.success() {
        bail!("Failed to install bonesdeploy from {repo_url}");
    }

    Ok(())
}

pub async fn update_remote_from_source(repo_url: &str, _version: &str) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        bail!("No .bones/bones.toml found. Run from a bonesdeploy project directory.");
    }

    let cfg = config::load(bones_toml)?;
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as("root", &cfg.host, port).await?;

    let install_root = paths::USR_LOCAL_BIN.trim_end_matches("/bin");
    println!("Building bonesremote from source on remote...");
    ssh::stream_cmd(
        &session,
        &format!("cargo install --locked --git {repo_url} bonesremote --force --root {install_root}"),
    )
    .await?;

    ssh::stream_cmd(
        &session,
        &format!(
            "mkdir -p {root} && chown root:root {root} && chmod 711 {root}",
            root = paths::DEFAULT_PROJECT_ROOT_PARENT
        ),
    )
    .await?;

    session.close().await?;

    Ok(())
}
