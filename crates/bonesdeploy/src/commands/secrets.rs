use std::env;
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::infra::{bootstrap_ssh, ssh};
use shared::config::parse_port;
use shared::paths;

const LOCAL_SECRETS_TOML: &str = ".bones/secrets.toml";
const LOCAL_SECRETS_DIR: &str = ".bones/secrets";

#[derive(Debug, Serialize, Deserialize)]
struct SecretsConfig {
    recipient: String,
    #[serde(rename = "file")]
    files: Vec<SecretFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SecretFile {
    name: String,
    local: String,
    remote: String,
    #[serde(default)]
    mode: String,
}

pub fn init(recipient: &str) -> Result<()> {
    ensure_gpg_installed()?;

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    if !bones_dir.is_dir() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_DIR);
    }

    let secrets_toml = Path::new(LOCAL_SECRETS_TOML);
    if secrets_toml.exists() {
        bail!("{LOCAL_SECRETS_TOML} already exists");
    }

    fs::create_dir_all(LOCAL_SECRETS_DIR).with_context(|| format!("Failed to create {LOCAL_SECRETS_DIR}"))?;

    let config = SecretsConfig {
        recipient: recipient.to_string(),
        files: vec![SecretFile {
            name: String::from(".env"),
            local: String::from("secrets/.env.gpg"),
            remote: String::from(".env"),
            mode: String::from("640"),
        }],
    };

    let content = toml::to_string(&config).context("Failed to serialize secrets config")?;
    fs::write(secrets_toml, content).with_context(|| format!("Failed to write {LOCAL_SECRETS_TOML}"))?;

    println!("Created {LOCAL_SECRETS_TOML} and {LOCAL_SECRETS_DIR}/");
    Ok(())
}

pub fn edit(name: &str) -> Result<()> {
    ensure_gpg_installed()?;

    let config = load_secrets_config()?;
    let file = config
        .files
        .iter()
        .find(|file| file.name == name)
        .ok_or_else(|| anyhow::anyhow!("Secret not found: {name}"))?;
    let encrypted_path = local_secret_path(&file.local);

    if let Some(parent) = encrypted_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let temp_path = create_temp_edit_path(name)?;

    if encrypted_path.is_file() {
        run_gpg(&[
            "--batch",
            "--yes",
            "--decrypt",
            "--output",
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
        ])?;
    }

    // ponytail: plaintext briefly exists on local disk during edit; upgrade path is stricter temp-file handling or an in-memory editor flow.
    let edit_result = open_editor(&temp_path);
    let encrypt_result = if edit_result.is_ok() {
        run_gpg(&[
            "--batch",
            "--yes",
            "--output",
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
            "--encrypt",
            "--recipient",
            &config.recipient,
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
        ])
    } else {
        Ok(())
    };

    let cleanup_result = fs::remove_file(&temp_path);

    edit_result?;
    encrypt_result?;
    if let Err(error) = cleanup_result {
        if error.kind() != ErrorKind::NotFound {
            eprintln!("Warning: failed to remove temp file {}: {error}", temp_path.display());
        }
    }

    println!("Updated encrypted secret {}", encrypted_path.display());
    Ok(())
}

pub async fn push() -> Result<()> {
    ensure_gpg_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let secrets = load_secrets_config()?;
    let deployment = cfg.deployment_paths(paths::DEFAULT_WEB_ROOT);
    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as(&ssh_user, &cfg.host, port).await?;

    for file in &secrets.files {
        let remote = validate_remote_path(&file.remote)?;
        let encrypted_path = local_secret_path(&file.local);
        if !encrypted_path.is_file() {
            bail!("Encrypted secret does not exist: {}", encrypted_path.display());
        }

        let plaintext = decrypt_secret(&encrypted_path)?;
        let target = Path::new(&deployment.shared).join(remote);
        let parent =
            target.parent().ok_or_else(|| anyhow::anyhow!("Remote target has no parent: {}", target.display()))?;
        let mode = effective_mode(&file.mode);
        let cmd = format!(
            "mkdir -p {parent} && cat > {target} && chmod {mode} {target}",
            parent = shell_quote_single(&parent.display().to_string()),
            target = shell_quote_single(&target.display().to_string()),
            mode = shell_quote_single(mode),
        );

        ssh::run_cmd_with_stdin(&session, &cmd, &plaintext).await?;
        println!("Pushed secret {} to remote shared/{}", file.name, file.remote);
    }

    session.close().await?;
    Ok(())
}

fn load_secrets_config() -> Result<SecretsConfig> {
    let path = Path::new(LOCAL_SECRETS_TOML);
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}

fn validate_remote_path(remote: &str) -> Result<&str> {
    if remote.is_empty() {
        bail!("secret remote path must not be empty");
    }

    for component in Path::new(remote).components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => bail!("secret remote path must not contain ."),
            Component::ParentDir => bail!("secret remote path must not contain .., got: {remote}"),
            Component::RootDir | Component::Prefix(_) => {
                bail!("secret remote path must be relative, got: {remote}")
            }
        }
    }

    Ok(remote)
}

fn ensure_gpg_installed() -> Result<()> {
    let output = Command::new("gpg").arg("--version").output().context("Failed to run gpg — is it installed?")?;
    if !output.status.success() {
        bail!("gpg is required but unavailable")
    }
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

fn create_temp_edit_path(name: &str) -> Result<PathBuf> {
    let sanitized = name.replace('/', "_");
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesdeploy-secret-{}-{nonce}-{sanitized}", std::process::id()));

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("Failed to create temp file {}", path.display()))?;

    Ok(path)
}

fn local_secret_path(relative_path: &str) -> PathBuf {
    Path::new(paths::LOCAL_BONES_DIR).join(relative_path)
}

fn decrypt_secret(path: &Path) -> Result<Vec<u8>> {
    let output = Command::new("gpg")
        .args(["--batch", "--yes", "--decrypt"])
        .arg(path)
        .output()
        .with_context(|| format!("Failed to run gpg for {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to decrypt {}\n{stderr}", path.display());
    }

    Ok(output.stdout)
}

fn run_gpg(args: &[&str]) -> Result<()> {
    let status = Command::new("gpg").args(args).status().context("Failed to run gpg")?;
    if !status.success() {
        bail!("gpg failed with status {status}");
    }
    Ok(())
}

fn effective_mode(mode: &str) -> &str {
    if mode.trim().is_empty() { "640" } else { mode }
}

fn shell_quote_single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{SecretsConfig, validate_remote_path};

    #[test]
    fn validate_remote_path_rejects_empty_absolute_and_dot_paths() {
        assert!(validate_remote_path("").is_err());
        assert!(validate_remote_path("/etc/passwd").is_err());
        assert!(validate_remote_path(".").is_err());
        assert!(validate_remote_path("../.env").is_err());
        assert!(validate_remote_path("storage/../app.key").is_err());
    }

    #[test]
    fn validate_remote_path_accepts_expected_relative_paths() -> Result<()> {
        assert_eq!(validate_remote_path(".env")?, ".env");
        assert_eq!(validate_remote_path("storage/app.key")?, "storage/app.key");
        Ok(())
    }

    #[test]
    fn parse_secrets_toml_example() -> Result<()> {
        let config: SecretsConfig = toml::from_str(
            r#"recipient = "alex@example.com"

[[file]]
name = ".env"
local = "secrets/.env.gpg"
remote = ".env"
mode = "640"
"#,
        )?;

        assert_eq!(config.recipient, "alex@example.com");
        assert_eq!(config.files.len(), 1);
        assert_eq!(config.files[0].name, ".env");
        assert_eq!(config.files[0].local, "secrets/.env.gpg");
        assert_eq!(config.files[0].remote, ".env");
        assert_eq!(config.files[0].mode, "640");
        Ok(())
    }
}
