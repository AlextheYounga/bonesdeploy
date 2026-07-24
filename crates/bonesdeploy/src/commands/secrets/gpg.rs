use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use shared::paths;

pub(super) fn home() -> PathBuf {
    let current = paths::bones_data_root().join("gnupg");
    if current.exists() {
        return current;
    }

    // TODO: remove after existing projects have migrated their GPG keyrings.
    let previous = paths::bones_config_root().join("_lib/gnupg");
    if previous.exists() {
        return previous;
    }

    current
}

pub(super) fn command() -> Command {
    let mut cmd = Command::new("gpg");
    cmd.arg("--homedir").arg(home().as_os_str());
    cmd
}

pub(super) fn ensure_installed() -> Result<()> {
    let output = Command::new("gpg").arg("--version").output().context("gpg is required.")?;
    if !output.status.success() {
        bail!("gpg is required.")
    }
    Ok(())
}

fn ensure_home() -> Result<()> {
    let gpg_home = home();
    fs::create_dir_all(&gpg_home).with_context(|| format!("Failed to create {}", gpg_home.display()))?;
    fs::set_permissions(&gpg_home, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to chmod 0700 {}", gpg_home.display()))?;
    Ok(())
}

pub(super) fn ensure_project_key(project_name: &str) -> Result<String> {
    ensure_home()?;

    let uid = format!("BonesDeploy secrets: {project_name}");

    if let Some(fingerprint) = find_fingerprint(&uid)? {
        return Ok(fingerprint);
    }

    generate_key(project_name, &uid)
}

fn find_fingerprint(uid: &str) -> Result<Option<String>> {
    let mut cmd = command();
    cmd.args(["--list-keys", "--with-colons", "--with-fingerprint", uid]);
    let output = cmd.output().context("Failed to run gpg --list-keys")?;

    if !output.status.success() {
        return Ok(None);
    }

    Ok(extract_fingerprint(&String::from_utf8_lossy(&output.stdout)))
}

fn generate_key(project_name: &str, uid: &str) -> Result<String> {
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

    let mut child = command()
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

    find_fingerprint(uid)?.ok_or_else(|| anyhow::anyhow!("Key was generated but fingerprint could not be found"))
}

pub(super) fn run(args: &[&str]) -> Result<()> {
    let mut cmd = command();
    cmd.args(args);
    let status = cmd.status().context("Failed to run gpg")?;
    if !status.success() {
        bail!("gpg failed with status {status}");
    }
    Ok(())
}

pub(super) fn decrypt(path: &Path) -> Result<Vec<u8>> {
    let mut cmd = command();
    cmd.args(["--batch", "--yes", "--decrypt"]).arg(path);
    let output = cmd.output().with_context(|| format!("Failed to run gpg for {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to decrypt {}\n{stderr}", path.display());
    }

    Ok(output.stdout)
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

#[cfg(test)]
mod tests {
    use super::extract_fingerprint;

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
