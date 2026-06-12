use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::config;
use crate::ssh;

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

    let host = format!("{}@{}", paths::DEPLOY_USER, cfg.data.host);
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

pub async fn update_remote_from_source(repo_url: &str, _version: &str) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        bail!("No .bones/bones.yaml found. Run from a bonesdeploy project directory.");
    }

    let cfg = config::load(bones_yaml)?;
    let port: u16 = cfg.data.port.parse().with_context(|| format!("Invalid port: {}", cfg.data.port))?;
    let session = ssh::connect_as("root", &cfg.data.host, port).await?;

    let install_root = paths::USR_LOCAL_BIN.trim_end_matches("/bin");
    println!("Building bonesremote from source on remote...");
    ssh::stream_cmd(&session, &format!("cargo install --git {repo_url} bonesremote --force --root {install_root}"))
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
