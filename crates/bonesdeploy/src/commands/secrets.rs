use std::env;
use std::fs::{self, OpenOptions, Permissions};
use std::io::{ErrorKind, Write as IoWrite};
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
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
    key_fingerprint: String,
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

fn gpg_home() -> PathBuf {
    paths::bones_config_root().join("gnupg")
}

fn gpg_command() -> Command {
    let mut cmd = Command::new("gpg");
    cmd.arg("--homedir").arg(gpg_home().as_os_str());
    cmd
}

pub fn init() -> Result<()> {
    ensure_gpg_installed()?;

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    if !bones_dir.is_dir() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_DIR);
    }

    let secrets_toml = Path::new(LOCAL_SECRETS_TOML);
    if secrets_toml.exists() {
        let content =
            fs::read_to_string(secrets_toml).with_context(|| format!("Failed to read {LOCAL_SECRETS_TOML}"))?;
        reject_old_recipient_config(&content)?;
        bail!("{LOCAL_SECRETS_TOML} already exists");
    }

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let key_fingerprint = ensure_project_key(&cfg.project_name)?;

    fs::create_dir_all(LOCAL_SECRETS_DIR).with_context(|| format!("Failed to create {LOCAL_SECRETS_DIR}"))?;

    let config = SecretsConfig {
        key_fingerprint,
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
            &config.key_fingerprint,
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
    reject_old_recipient_config(&content)?;
    toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
}

fn reject_old_recipient_config(content: &str) -> Result<()> {
    let raw: toml::Value = toml::from_str(content).context("Failed to parse secrets config")?;
    if raw.as_table().map_or(false, |t| t.contains_key("recipient")) {
        bail!(
            "Old recipient-based secrets config is no longer supported. \
             Delete .bones/secrets.toml and run `bonesdeploy secrets init` again."
        );
    }
    Ok(())
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

fn ensure_gpg_home() -> Result<()> {
    let home = gpg_home();
    fs::create_dir_all(&home).with_context(|| format!("Failed to create {}", home.display()))?;
    fs::set_permissions(&home, Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to chmod 0700 {}", home.display()))?;
    Ok(())
}

fn ensure_project_key(project_name: &str) -> Result<String> {
    ensure_gpg_home()?;

    let uid = format!("BonesDeploy secrets: {}", project_name);

    if let Some(fingerprint) = find_key_fingerprint(&uid)? {
        return Ok(fingerprint);
    }

    generate_project_key(project_name, &uid)
}

fn find_key_fingerprint(uid: &str) -> Result<Option<String>> {
    let mut cmd = gpg_command();
    cmd.args(["--list-keys", "--with-colons", "--with-fingerprint", uid]);
    let output = cmd.output().context("Failed to run gpg --list-keys")?;

    if !output.status.success() {
        return Ok(None);
    }

    Ok(extract_fingerprint(&String::from_utf8_lossy(&output.stdout)))
}

fn extract_fingerprint(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("fpr:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 10 {
                return Some(parts[9].to_string());
            }
        }
    }
    None
}

// ponytail: MVP uses an unprotected local project key inside the private
// BonesDeploy GPG home; upgrade path is optional passphrase / OS keychain
// integration.
fn generate_project_key(project_name: &str, uid: &str) -> Result<String> {
    let email = format!("{project_name}@bonesdeploy.local");
    let params = format!(
        "Key-Type: RSA\n\
         Key-Length: 4096\n\
         Key-Usage: cert\n\
         Subkey-Type: RSA\n\
         Subkey-Length: 4096\n\
         Subkey-Usage: encrypt\n\
         Name-Real: {uid}\n\
         Name-Email: {email}\n\
         %no-protection\n\
         %commit\n"
    );

    let mut child = gpg_command()
        .args(["--batch", "--generate-key"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawngpg --generate-key")?;

    {
        let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?;
        stdin.write_all(params.as_bytes()).context("Failed to write batch key params to gpg")?;
    }

    let output = child.wait_with_output().context("Failed to wait for gpg --generate-key")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to generate GPG key: {stderr}");
    }

    find_key_fingerprint(uid)?.ok_or_else(|| anyhow::anyhow!("Key was generated but fingerprint could not be found"))
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
    let mut cmd = gpg_command();
    cmd.args(["--batch", "--yes", "--decrypt"]).arg(path);
    let output = cmd.output().with_context(|| format!("Failed to run gpg for {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to decrypt {}\n{stderr}", path.display());
    }

    Ok(output.stdout)
}

fn run_gpg(args: &[&str]) -> Result<()> {
    let mut cmd = gpg_command();
    cmd.args(args);
    let status = cmd.status().context("Failed to run gpg")?;
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

    use super::{SecretsConfig, extract_fingerprint, gpg_home, reject_old_recipient_config, validate_remote_path};
    use shared::paths;

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
            r#"key_fingerprint = "ABCDEF123456..."

[[file]]
name = ".env"
local = "secrets/.env.gpg"
remote = ".env"
mode = "640"
"#,
        )?;

        assert_eq!(config.key_fingerprint, "ABCDEF123456...");
        assert_eq!(config.files.len(), 1);
        assert_eq!(config.files[0].name, ".env");
        assert_eq!(config.files[0].local, "secrets/.env.gpg");
        assert_eq!(config.files[0].remote, ".env");
        assert_eq!(config.files[0].mode, "640");
        Ok(())
    }

    #[test]
    fn old_recipient_config_is_rejected() {
        let result = reject_old_recipient_config(
            r#"recipient = "alex@example.com"

[[file]]
name = ".env"
local = "secrets/.env.gpg"
remote = ".env"
mode = "640"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn gpg_home_resolves_under_bones_config_root() {
        assert_eq!(gpg_home(), paths::bones_config_root().join("gnupg"));
    }

    #[test]
    fn extract_fingerprint_parses_fpr_line() {
        let output = "tru::1:1754651437:0:3:1:3\nfpr:::::::::ABCDEF1234567890ABCDEF1234567890ABCDEF:\nuid:::::::::Test <test@example.com>:\n";
        assert_eq!(extract_fingerprint(output).as_deref(), Some("ABCDEF1234567890ABCDEF1234567890ABCDEF"));
    }

    #[test]
    fn extract_fingerprint_returns_none_without_fpr_line() {
        let output = "tru::1:1754651437:0:3:1:3\nuid:::::::::Test <test@example.com>:\n";
        assert_eq!(extract_fingerprint(output), None);
    }
}
