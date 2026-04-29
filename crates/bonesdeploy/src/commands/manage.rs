use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;

pub fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let remote_bones_yaml = format!("{}/{}/bones.yaml", cfg.data.git_dir, config::Constants::REMOTE_BONES_DIR);
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
