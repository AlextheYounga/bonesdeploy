use std::env;
use std::fs::{self, OpenOptions, Permissions};
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config;
use crate::frameworks;
use crate::infra::ssh;
use crate::ui::output;
use bonesdeploy_core::config as shared_config;
use bonesdeploy_core::config::parse_port;
use bonesdeploy_core::paths;

mod gpg;

const LOCAL_ENV_SECRET: &str = ".bones/secrets/.env.gpg";
const DEFAULT_SECRET_MODE: &str = "640";

pub fn init() -> Result<()> {
    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    if !bones_dir.is_dir() {
        bail!("Missing .bones config\n\n{}", output::next_step("bonesdeploy init"));
    }

    let secrets_toml = Path::new(".bones/secrets.toml");
    if secrets_toml.exists() {
        bail!("Missing encrypted secrets\n\n{}", output::next_step("bonesdeploy secrets edit"));
    }

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    initialize_defaults(&cfg)?;

    println!("{} Secrets initialized.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy secrets edit"));
    Ok(())
}

pub fn initialize_defaults(cfg: &config::Bones) -> Result<()> {
    let encrypted_path = Path::new(LOCAL_ENV_SECRET);
    if encrypted_path.is_file() {
        return Ok(());
    }

    gpg::ensure_installed()?;
    let key_fingerprint = gpg::ensure_project_key(&cfg.project_name)?;
    fs::create_dir_all(paths::LOCAL_BONES_SECRETS_DIR)
        .with_context(|| format!("Failed to create {}", paths::LOCAL_BONES_SECRETS_DIR))?;

    let temp_path = create_temp_edit_path()?;
    let mut effective_config = cfg.clone();
    shared_config::apply_derived_defaults(&mut effective_config);
    fs::write(
        &temp_path,
        frameworks::environment_example(
            &effective_config.framework.template,
            &effective_config.project_name,
            &effective_config.domain,
            &effective_config.preview_domain,
        )
        .unwrap_or_default(),
    )
    .with_context(|| format!("Failed to write default secrets to {}", temp_path.display()))?;
    fs::set_permissions(&temp_path, Permissions::from_mode(0o600))?;

    let encrypted_result = gpg::run(&[
        "--batch",
        "--yes",
        "--output",
        encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
        "--encrypt",
        "--recipient",
        &key_fingerprint,
        temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
    ]);
    let cleanup_result = fs::remove_file(&temp_path);
    encrypted_result?;
    cleanup_result.with_context(|| format!("Failed to remove temporary secrets file {}", temp_path.display()))?;
    fs::set_permissions(encrypted_path, Permissions::from_mode(0o640))?;
    Ok(())
}

pub fn edit() -> Result<()> {
    gpg::ensure_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let key_fingerprint = gpg::ensure_project_key(&cfg.project_name)?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);

    if let Some(parent) = encrypted_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let temp_path = create_temp_edit_path()?;

    if encrypted_path.is_file() {
        gpg::run(&[
            "--batch",
            "--yes",
            "--decrypt",
            "--output",
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
        ])?;
    }

    let edit_result = open_editor(&temp_path);
    let encrypt_result = if edit_result.is_ok() {
        gpg::run(&[
            "--batch",
            "--yes",
            "--output",
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
            "--encrypt",
            "--recipient",
            &key_fingerprint,
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
        ])
    } else {
        Ok(())
    };

    let cleanup_result = fs::remove_file(&temp_path);

    edit_result?;
    encrypt_result?;
    if let Err(error) = cleanup_result
        && error.kind() != ErrorKind::NotFound
    {
        eprintln!("Warning: could not remove temporary secret file: {}", temp_path.display());
    }

    println!("{} Secrets updated.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy secrets push"));
    Ok(())
}

pub async fn push() -> Result<()> {
    gpg::ensure_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let runtime_group = shared_config::runtime_group_for(&cfg.project_name);

    let ssh_user = config::bootstrap_ssh_user(&cfg);
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as(&ssh_user, &cfg.host, port).await?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);
    if !encrypted_path.is_file() {
        bail!("Missing encrypted secrets\n\n{}", output::next_step("bonesdeploy secrets edit"));
    }

    let plaintext = gpg::decrypt(encrypted_path)?;
    let shared = Path::new(&cfg.project_root).join(paths::SHARED_DIR);
    let target = shared.join(paths::DOT_ENV);
    let parent = target.parent().ok_or_else(|| anyhow::anyhow!("Remote target has no parent: {}", target.display()))?;
    let parent_s = ssh::shell_quote(&parent.display().to_string());
    let target_s = ssh::shell_quote(&target.display().to_string());
    let group_s = ssh::shell_quote(&runtime_group);
    let cmd = format!(
        "tmp=; trap 'rm -f \"$tmp\"' EXIT; mkdir -p {parent_s} && tmp=$(mktemp {target_s}.XXXXXX) && cat > \"$tmp\" && chown root:{group_s} \"$tmp\" && chmod {DEFAULT_SECRET_MODE} \"$tmp\" && mv \"$tmp\" {target_s} && tmp=",
    );

    ssh::run_cmd_with_stdin(&session, &cmd, &plaintext).await?;
    session.close().await?;
    println!("{} Secrets pushed.", output::success_marker());
    Ok(())
}

fn open_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("$EDITOR is not set. Set it before running `bonesdeploy secrets edit`."))?;

    let status = Command::new("sh")
        .arg("-c")
        .arg("${EDITOR:?EDITOR is not set} \"$1\"")
        .arg("sh")
        .arg(path)
        .env("EDITOR", editor)
        .status()
        .context("Failed to launch editor")?;

    if !status.success() {
        bail!("Editor exited with status {status}");
    }

    Ok(())
}

fn create_temp_edit_path() -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesdeploy-env-{}-{nonce}", process::id()));

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("Failed to create temp file {}", path.display()))?;

    Ok(path)
}
