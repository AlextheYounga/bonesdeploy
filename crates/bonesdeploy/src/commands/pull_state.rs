use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;

use crate::config;
use crate::git;
use crate::infra::rsync;
use shared::config::default_deploy_user;
use shared::paths;

use crate::commands::init_project;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let target = resolve_pull_target()?;
    let remote_bones = format!("{}@{}:{}/{}/", target.user, target.host, target.repo_path, paths::BONES_DIR);

    println!("Pulling .bones/ from {remote_bones}...");

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    if !bones_dir.exists() {
        let project_name = config::repo_directory_name()?;
        let config_dir = config::bones_config_dir(&project_name);
        if config_dir.exists() && !config_dir.is_dir() {
            fs::remove_file(&config_dir)
                .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
        }
        fs::create_dir_all(&config_dir)?;
        unix_fs::symlink(&config_dir, bones_dir)?;
    }

    rsync_bones(&target)?;
    init_project::symlink_pre_push()?;

    println!("\n{} .bones/ pulled from remote.", style("Done!").green().bold());
    Ok(())
}

fn resolve_pull_target() -> Result<git::RemoteConnectionDetails> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if bones_toml.exists() {
        let cfg = config::load(bones_toml)?;
        return Ok(git::RemoteConnectionDetails {
            user: default_deploy_user(),
            host: cfg.host,
            port: cfg.port,
            repo_path: cfg.repo_path,
        });
    }

    let remote_name = resolve_remote_name()?;
    let details = git::infer_remote_connection_details(&remote_name)?
        .with_context(|| format!("Remote '{remote_name}' must use an SSH-style URL ending in .git"))?;

    Ok(details)
}

fn resolve_remote_name() -> Result<String> {
    if git::remote_exists("production")? {
        return Ok(String::from("production"));
    }

    let remotes = git::list_remotes()?;
    match remotes.as_slice() {
        [] => bail!("No git remotes configured. Add a deployment remote before running bonesdeploy pull."),
        [remote] => Ok(remote.clone()),
        _ => {
            bail!("Multiple git remotes configured. Keep .bones/bones.toml or name the deployment remote 'production'.")
        }
    }
}

fn rsync_bones(target: &git::RemoteConnectionDetails) -> Result<()> {
    let source = format!("{}@{}:{}/{}/", target.user, target.host, target.repo_path, paths::BONES_DIR);
    let ssh_arg = format!("ssh -p {} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null", target.port);
    let dest = format!("{}/", paths::LOCAL_BONES_DIR);
    let status = rsync::status(&["-av", "--delete", "-e", &ssh_arg, &source, &dest])?;

    if !status.success() {
        bail!("rsync failed");
    }

    Ok(())
}
