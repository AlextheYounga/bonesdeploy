use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::config;
use crate::infra::ssh;
use bonesdeploy_core::config::{default_deploy_user, parse_port};

pub(super) fn current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub(super) fn current_remote_version() -> String {
    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() {
        return String::from("unknown");
    }

    let Ok(cfg) = config::load(env_file) else {
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

    bonesinfra::run(&[
        "patches",
        "apply",
        "--env-file",
        paths::DOT_ENV,
        "--target-version",
        target_version,
        "--scope",
        "remote",
    ])?;

    Ok(())
}

fn bonesremote_download_command(version: &str, install_root: &str) -> String {
    let artifact = "bonesremote-x86_64-unknown-linux-musl";
    let base_url = format!("https://github.com/AlextheYounga/bonesdeploy/releases/download/v{version}");
    format!(
        "set -eu; case \"$(uname -m)\" in x86_64|amd64) ;; *) echo 'ERROR: bonesremote release binaries only support x86_64 hosts.' >&2; exit 1 ;; esac; tmp=$(mktemp -d); trap 'rm -rf \"$tmp\"' EXIT; curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 '{base_url}/{artifact}' --output \"$tmp/{artifact}\"; curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 '{base_url}/{artifact}.sha256' --output \"$tmp/{artifact}.sha256\"; (cd \"$tmp\" && sha256sum --check \"{artifact}.sha256\"); chmod 0755 \"$tmp/{artifact}\"; test \"$(\"$tmp/{artifact}\" version)\" = 'bonesremote {version}'; install -o root -g root -m 0755 \"$tmp/{artifact}\" '{install_root}/{binary}.tmp'; mv -f '{install_root}/{binary}.tmp' '{install_root}/{binary}'",
        artifact = artifact,
        binary = paths::BONESREMOTE_BINARY,
    )
}

#[cfg(test)]
mod tests {
    use super::bonesremote_download_command;

    #[test]
    fn remote_update_downloads_versioned_release_and_checksum() {
        let command = bonesremote_download_command("0.7.3", "/usr/local/bin");

        assert!(command.contains("releases/download/v0.7.3"));
        assert!(command.contains("bonesremote-x86_64-unknown-linux-musl.sha256"));
        assert!(command.contains("sha256sum --check"));
        assert!(command.contains("uname -m"));
        assert!(command.contains("bonesremote 0.7.3"));
        assert!(command.contains("install -o root -g root -m 0755"));
        assert!(command.contains("'/usr/local/bin/bonesremote.tmp'"));
        assert!(command.contains("'/usr/local/bin/bonesremote'"));
    }
}
