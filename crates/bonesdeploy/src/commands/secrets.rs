use std::env;
use std::fs::{self, OpenOptions, Permissions};
use std::io::{ErrorKind, Write as IoWrite};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config;
use crate::infra::{bootstrap_ssh, ssh};
use shared::config as shared_config;
use shared::config::parse_port;
use shared::paths;

const LOCAL_SECRETS_DIR: &str = ".bones/secrets";
const LOCAL_ENV_SECRET: &str = ".bones/secrets/.env.gpg";
const REMOTE_ENV_SECRET: &str = ".env";
const DEFAULT_SECRET_MODE: &str = "640";

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
        bail!("Missing .bones config\n\nNext: run bonesdeploy init.");
    }

    let secrets_toml = Path::new(".bones/secrets.toml");
    if secrets_toml.exists() {
        bail!("Missing encrypted secrets\n\nNext: run bonesdeploy secrets edit.");
    }

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let _key_fingerprint = ensure_project_key(&cfg.project_name)?;

    fs::create_dir_all(LOCAL_SECRETS_DIR).with_context(|| format!("Failed to create {LOCAL_SECRETS_DIR}"))?;

    println!("Secrets initialized.");
    println!();
    println!("Next: run bonesdeploy secrets edit.");
    Ok(())
}

pub fn edit() -> Result<()> {
    ensure_gpg_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let key_fingerprint = ensure_project_key(&cfg.project_name)?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);

    if let Some(parent) = encrypted_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let temp_path = create_temp_edit_path()?;

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

    println!("Secrets updated.");
    println!();
    println!("Next: run bonesdeploy secrets push.");
    Ok(())
}

pub async fn push() -> Result<()> {
    ensure_gpg_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;
    let runtime_group = if runtime.runtime_group.is_empty() {
        shared_config::runtime_group_for(&cfg.project_name)
    } else {
        runtime.runtime_group
    };

    let deployment = cfg.deployment_paths(paths::DEFAULT_WEB_ROOT);
    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as(&ssh_user, &cfg.host, port).await?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);
    if !encrypted_path.is_file() {
        bail!("Missing encrypted secrets\n\nNext: run bonesdeploy secrets edit.");
    }

    let plaintext = decrypt_secret(encrypted_path)?;
    let target = Path::new(&deployment.shared).join(REMOTE_ENV_SECRET);
    let parent = target.parent().ok_or_else(|| anyhow::anyhow!("Remote target has no parent: {}", target.display()))?;
    let parent_s = shell_quote_single(&parent.display().to_string());
    let target_s = shell_quote_single(&target.display().to_string());
    let group_s = shell_quote_single(&runtime_group);
    let cmd = format!(
        "mkdir -p {parent_s} && tmp=$(mktemp {target_s}.XXXXXX) && cat > \"$tmp\" && chown root:{group_s} \"$tmp\" && chmod {DEFAULT_SECRET_MODE} \"$tmp\" && mv \"$tmp\" {target_s}",
    );

    ssh::run_cmd_with_stdin(&session, &cmd, &plaintext).await?;
    session.close().await?;
    println!("Secrets pushed.");
    Ok(())
}

fn ensure_gpg_installed() -> Result<()> {
    let output = Command::new("gpg").arg("--version").output().context("gpg is required.")?;
    if !output.status.success() {
        bail!("gpg is required.")
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

    let uid = format!("BonesDeploy secrets: {project_name}");

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
        .context("Failed to spawn gpg --generate-key")?;

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

fn shell_quote_single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::{extract_fingerprint, gpg_home};
    use shared::paths;

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
