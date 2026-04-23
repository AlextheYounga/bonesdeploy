use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;

use crate::config;
use crate::ssh;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let git_dir = &cfg.data.git_dir;
    let remote_bones = format!("{git_dir}/{}/", config::Constants::REMOTE_BONES_DIR);

    // rsync .bones/ to remote
    println!("Syncing .bones/ to {remote_bones}...");
    rsync_bones(&cfg)?;

    // Connect via SSH for post-rsync setup
    let session = ssh::connect(&cfg).await?;

    // Delete sample hooks from bare repo
    println!("Cleaning sample hooks from remote...");
    let cmd = format!(
        "find {git_dir}/{}/ -maxdepth 1 -name '*.sample' -delete 2>/dev/null; true",
        config::Constants::REMOTE_HOOKS_DIR
    );
    ssh::run_cmd(&session, &cmd).await?;

    // Symlink bones hooks into bare repo hooks
    println!("Symlinking hooks...");
    let cmd = format!(
        "for hook in {git_dir}/{}/{}/{}; do \
            name=$(basename \"$hook\"); \
            ln -sf \"$hook\" \"{git_dir}/{}/$name\"; \
          done",
        config::Constants::REMOTE_BONES_DIR,
        config::Constants::REMOTE_HOOKS_DIR,
        "*",
        config::Constants::REMOTE_HOOKS_DIR
    );
    ssh::run_cmd(&session, &cmd).await?;

    session.close().await?;

    println!("\n{} .bones/ synced to remote.", style("Done!").green().bold());

    Ok(())
}

fn rsync_bones(cfg: &config::BonesConfig) -> Result<()> {
    let user = &cfg.permissions.defaults.deploy_user;
    let host = &cfg.data.host;
    let port = &cfg.data.port;
    let git_dir = &cfg.data.git_dir;
    let dest = format!("{user}@{host}:{git_dir}/{}/", config::Constants::REMOTE_BONES_DIR);

    let status = Command::new("rsync")
        .args([
            "-av",
            "--delete",
            "-e",
            &format!("ssh -p {port}"),
            &format!("{}/", config::Constants::BONES_DIR),
            &dest,
        ])
        .status()
        .context("Failed to run rsync — is it installed?")?;

    if !status.success() {
        bail!("rsync failed");
    }

    Ok(())
}
