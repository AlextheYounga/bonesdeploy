use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;

pub fn run() -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = format!("{}/{}/bones.toml", cfg.data.repo_path, config::Constants::REMOTE_BONES_DIR);
    let remote_command = format!("bonesremote manage --config {}", shell_quote_single(&remote_bones_toml));

    let target = format!("{}@{}", cfg.data.deploy_user, cfg.data.host);

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

    /// Wraps a plain value in single quotes to prevent whitespace and token splitting.
    #[test]
    fn shell_quote_single_wraps_plain_value_in_single_quotes() {
        let path = paths::default_project_root_for("acme");
        assert_eq!(shell_quote_single(&path), format!("'{path}'"));
    }

    /// Escapes embedded single quotes safely for remote shell execution.
    #[test]
    fn shell_quote_single_escapes_embedded_single_quotes() {
        assert_eq!(shell_quote_single("it'works"), "'it'\"'\"'works'");
    }

    /// Returns an explicit empty string for empty input, not a zero-length argument.
    #[test]
    fn shell_quote_single_handles_empty_string() {
        assert_eq!(shell_quote_single(""), "''");
    }
}
