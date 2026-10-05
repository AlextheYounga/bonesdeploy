use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use bonesdeploy_core::paths;

use crate::platform;

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
    let output = Command::new("gpg").arg("--version").output().context(gpg_prerequisite())?;
    if !output.status.success() {
        bail!(gpg_prerequisite())
    }
    Ok(())
}

const fn gpg_prerequisite() -> &'static str {
    "GnuPG is required; install Gpg4win and ensure gpg.exe is on PATH"
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};

    use super::ensure_installed;
    use crate::test_support::with_env;

    #[test]
    fn missing_gnupg_names_the_windows_prerequisite_and_executable() -> Result<()> {
        let result = with_env("PATH", Some("missing-gpg-path"), ensure_installed);
        let error = result.err().context("missing GnuPG should fail")?;
        let message = format!("{error:#}");
        assert!(message.contains("GnuPG is required"), "{message}");
        assert!(message.contains("gpg.exe"), "{message}");
        Ok(())
    }
}

fn ensure_home() -> Result<()> {
    let gpg_home = home();
    fs::create_dir_all(&gpg_home).with_context(|| format!("Failed to create {}", gpg_home.display()))?;
    #[cfg(unix)]
    platform::set_mode(&gpg_home, 0o700)?;
    #[cfg(not(unix))]
    platform::set_mode(&gpg_home, 0o700);
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

pub fn extract_fingerprint(output: &str) -> Option<String> {
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
