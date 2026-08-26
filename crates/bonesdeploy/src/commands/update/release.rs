use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::config;
use crate::infra;
use crate::infra::ssh;
use bonesdeploy_core::config::parse_port;

pub(super) fn current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub(super) async fn current_remote_version() -> String {
    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() {
        return String::from("unknown");
    }

    let Ok(cfg) = config::load(env_file) else {
        return String::from("unknown");
    };

    let Ok(session) = ssh::connect(&cfg).await else {
        return String::from("unknown");
    };
    let version = ssh::run_cmd(&session, "bonesremote version").await.ok();
    let _ = session.close().await;

    version
        .as_deref()
        .map(str::trim)
        .and_then(|output| output.strip_prefix("bonesremote "))
        .unwrap_or("unknown")
        .to_string()
}

pub(super) fn update_local_from_crates_io(version: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(["install", "--locked", paths::BONESDEPLOY_BINARY, "--version", version, "--force"])
        .status()
        .context("Failed to run cargo install for bonesdeploy from crates.io")?;

    if !status.success() {
        bail!("Failed to install bonesdeploy {version} from crates.io");
    }

    Ok(())
}

pub(super) async fn update_remote_from_release(current_version: &str, target_version: &str) -> Result<()> {
    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() {
        bail!("No {} found. Run from a bonesdeploy project directory.", paths::DOT_ENV);
    }

    let cfg = config::load(env_file)?;
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as("root", &cfg.host, port).await?;

    let install_root = paths::USR_LOCAL_BIN;
    if current_version != target_version {
        ssh::stream_cmd(&session, &bonesremote_download_command(target_version, install_root)).await?;
    }

    ssh::stream_cmd(
        &session,
        &format!(
            "mkdir -p {root} && chown root:root {root} && chmod 711 {root}",
            root = paths::DEFAULT_PROJECT_ROOT_PARENT
        ),
    )
    .await?;

    session.close().await?;

    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(
        &["patches", "apply", "--request-stdin", "--target-version", target_version, "--scope", "remote"],
        &request,
    )?;

    Ok(())
}

pub fn bonesremote_download_command(version: &str, install_root: &str) -> String {
    let artifact = "bonesremote-x86_64-unknown-linux-musl";
    let base_url = format!("https://github.com/AlextheYounga/bonesdeploy/releases/download/v{version}");
    format!(
        "set -eu; case \"$(uname -m)\" in x86_64|amd64) ;; *) echo 'ERROR: bonesremote release binaries only support x86_64 hosts.' >&2; exit 1 ;; esac; tmp=$(mktemp -d); trap 'rm -rf \"$tmp\"' EXIT; curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 '{base_url}/{artifact}' --output \"$tmp/{artifact}\"; curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 '{base_url}/{artifact}.sha256' --output \"$tmp/{artifact}.sha256\"; (cd \"$tmp\" && sha256sum --check \"{artifact}.sha256\"); chmod 0755 \"$tmp/{artifact}\"; test \"$(\"$tmp/{artifact}\" version)\" = 'bonesremote {version}'; install -o root -g root -m 0755 \"$tmp/{artifact}\" '{install_root}/{binary}.tmp'; mv -f '{install_root}/{binary}.tmp' '{install_root}/{binary}'",
        artifact = artifact,
        binary = paths::BONESREMOTE_BINARY,
    )
}
