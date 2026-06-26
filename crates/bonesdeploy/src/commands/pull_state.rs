use std::fs;
use std::io::Write as _;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config;
use crate::infra::git;
use anyhow::{Context, Result, bail};
use shared::paths;

use crate::commands::init_project;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let target = resolve_pull_target()?;

    println!("Pulling .bones...");

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

    let archive = fetch_remote_archive(&target)?;
    clear_managed_bones_entries(bones_dir)?;
    extract_bones_archive(bones_dir, &archive)?;
    init_project::symlink_pre_push()?;

    println!(".bones pulled.");
    println!();
    println!("Next: run bonesdeploy doctor.");
    Ok(())
}

fn resolve_pull_target() -> Result<git::RemoteConnectionDetails> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if bones_toml.exists() {
        let cfg = config::load(bones_toml)?;
        return Ok(git::RemoteConnectionDetails {
            user: cfg.ssh_user,
            host: cfg.host,
            port: cfg.port,
            repo_path: cfg.repo_path,
        });
    }

    let remote_name = resolve_remote_name()?;
    let details = git::infer_remote_connection_details(&remote_name)?
        .with_context(|| format!("Remote '{remote_name}' must use an SSH-style URL ending in .git"))?;

    Ok(git::RemoteConnectionDetails { user: String::from("root"), ..details })
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
            bail!(
                "Multiple git remotes configured. Keep {} or name the deployment remote 'production'.",
                paths::LOCAL_BONES_TOML
            )
        }
    }
}

fn fetch_remote_archive(target: &git::RemoteConnectionDetails) -> Result<Vec<u8>> {
    let site = site_name_from_repo_path(&target.repo_path)?;
    let output = Command::new("ssh")
        .args([
            "-p",
            &target.port,
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "UserKnownHostsFile=/dev/null",
            &format!("{}@{}", target.user, target.host),
            &format!("bonesremote site export --site '{site}'"),
        ])
        .output()
        .context("Failed to export remote site state")?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to export remote site state\n{stderr}")
}

fn clear_managed_bones_entries(bones_dir: &Path) -> Result<()> {
    for name in [paths::BONES_TOML, paths::RUNTIME_TOML, paths::DEPLOYMENT_DIR, paths::HOOKS_DIR] {
        let path = bones_dir.join(name);
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
        } else {
            fs::remove_file(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn extract_bones_archive(bones_dir: &Path, archive: &[u8]) -> Result<()> {
    let mut child = Command::new("tar")
        .args(["-xzf", "-", "-C"])
        .arg(bones_dir)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run tar for pull")?;

    let mut stdin = child.stdin.take().context("tar stdin was not piped")?;
    stdin.write_all(archive).context("Failed to write pulled archive to tar")?;
    drop(stdin);

    let output = child.wait_with_output().context("Failed to finish extracting .bones")?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to extract .bones\n{stderr}")
}

fn site_name_from_repo_path(repo_path: &str) -> Result<String> {
    let repo_name = Path::new(repo_path)
        .file_name()
        .and_then(|name| name.to_str())
        .context("Remote repo path must end in a site name")?;
    repo_name.strip_suffix(".git").map(ToOwned::to_owned).context("Remote repo path must end in .git")
}
