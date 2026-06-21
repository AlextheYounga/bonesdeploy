use std::path::Path;

use anyhow::{Result, bail};
use console::style;

use crate::config;
use crate::infra::rsync;
use crate::infra::ssh;
use shared::config::default_deploy_user;
use shared::paths;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let repo_path = &cfg.repo_path;
    let remote_bones = format!("{repo_path}/{}/", paths::BONES_DIR);

    // rsync .bones/ to remote
    println!("Syncing .bones/ to {remote_bones}...");
    sync_bones_directory(&cfg)?;

    // Connect via SSH for post-rsync setup
    let session = ssh::connect(&cfg).await?;

    println!("Cleaning sample hooks from remote...");
    let cmd = format!("find {repo_path}/{}/ -maxdepth 1 -name '*.sample' -delete 2>/dev/null; true", paths::HOOKS_DIR);
    ssh::run_cmd(&session, &cmd).await?;

    println!("Symlinking hooks...");
    let cmd = format!(
        "for hook in {repo_path}/{}/{}/{}; do \
            name=$(basename \"$hook\"); \
            ln -sf \"$hook\" \"{repo_path}/{}/$name\"; \
          done",
        paths::BONES_DIR,
        paths::HOOKS_DIR,
        "*",
        paths::HOOKS_DIR
    );
    ssh::run_cmd(&session, &cmd).await?;

    session.close().await?;

    println!("\n{} .bones/ synced to remote.", style("Done!").green().bold());

    Ok(())
}

pub(crate) fn sync_bones_directory(cfg: &config::Bones) -> Result<()> {
    let args = rsync_args(cfg);
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();

    let status = rsync::status(&arg_refs)?;

    if !status.success() {
        bail!("rsync failed");
    }

    Ok(())
}

pub(crate) fn rsync_args(cfg: &config::Bones) -> Vec<String> {
    let user = default_deploy_user();
    let host = &cfg.host;
    let port = &cfg.port;
    let repo_path = &cfg.repo_path;
    let dest = format!("{user}@{host}:{repo_path}/{}/", paths::BONES_DIR);

    let ssh_arg = format!("ssh -p {port} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null");
    let source = format!("{}/", paths::LOCAL_BONES_DIR);
    vec![
        String::from("-av"),
        String::from("--delete"),
        String::from("--exclude"),
        String::from("secrets/"),
        String::from("-e"),
        ssh_arg,
        source,
        dest,
    ]
}

#[cfg(test)]
mod tests {
    use super::rsync_args;
    use crate::config::Bones;

    #[test]
    fn rsync_args_exclude_local_secrets_directory_only() {
        let cfg = Bones {
            host: String::from("deploy.example.com"),
            port: String::from("22"),
            repo_path: String::from("/home/git/acme.git"),
            ..Default::default()
        };

        let args = rsync_args(&cfg);
        let excludes =
            args.windows(2).filter(|pair| pair[0] == "--exclude").map(|pair| pair[1].as_str()).collect::<Vec<_>>();

        assert!(excludes.contains(&"secrets/"));
        assert!(!excludes.contains(&"secrets.toml"));
    }
}
