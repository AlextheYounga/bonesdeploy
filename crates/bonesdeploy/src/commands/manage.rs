use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;

pub fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let remote_bones_yaml = format!("{}/{}/bones.yaml", cfg.data.repo_path, config::Constants::REMOTE_BONES_DIR);
    let remote_command = format!("bonesremote manage --config {}", shell_quote_single(&remote_bones_yaml));

    let target = format!("{}@{}", cfg.permissions.defaults.deploy_user, cfg.data.host);

    let status = Command::new("ssh")
        .arg("-t")
        .arg("-p")
        .arg(&cfg.data.port)
        .arg(&target)
        .arg(&remote_command)
        .status()
        .context("Failed to launch ssh for remote manage session")?;

    if !status.success() {
        bail!("Remote manage session failed with status {status}");
    }

    Ok(())
}

fn shell_quote_single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::shell_quote_single;
    use shared::paths;

    // Plain values still require shell quoting to prevent whitespace/token splitting.
    #[test]
    fn shell_quote_single_wraps_plain_value_in_single_quotes() {
        let path = paths::default_project_root_for("acme");
        assert_eq!(shell_quote_single(&path), format!("'{path}'"));
    }

    // Embedded single quotes must be escaped safely for remote shell execution.
    #[test]
    fn shell_quote_single_escapes_embedded_single_quotes() {
        assert_eq!(shell_quote_single("it'works"), "'it'\"'\"'works'");
    }

    // Empty args must remain explicit empty strings, not disappear from command argv.
    #[test]
    fn shell_quote_single_handles_empty_string() {
        assert_eq!(shell_quote_single(""), "''");
    }
}
