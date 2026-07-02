use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, thread_local};

use anyhow::{Context, Result, bail};
use shared::config;
use shared::paths;

use crate::privileges;

thread_local! {
    static SITES_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

fn resolved_sites_root() -> PathBuf {
    SITES_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_sites_root)
}

fn resolved_tmp_root(site: &str) -> Result<PathBuf> {
    let bones_path = resolved_sites_root().join(site).join(paths::BONES_TOML);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;
    Ok(Path::new(&cfg.project_root).join(paths::TMP_BUILDS_DIR))
}

pub fn run(site: &str, revision: &str, context_dir: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release checkout")?;

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    let archive_output = Command::new("git")
        .args(["--git-dir", &cfg.repo_path, "archive", "--format=tar", revision])
        .output()
        .with_context(|| format!("Failed to run git archive for revision {revision} in {}", cfg.repo_path))?;
    let git_stderr = String::from_utf8_lossy(&archive_output.stderr).into_owned();

    if !archive_output.status.success() {
        bail!("Failed to export source revision '{revision}' from {}\n{git_stderr}", cfg.repo_path);
    }

    if !git_stderr.is_empty() {
        println!("[bonesdeploy] git archive reported: {git_stderr}");
    }

    let mut archive = archive_output.stdout.as_slice();

    let mut tar = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(&context_dir)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start tar extraction into {}", context_dir.display()))?;

    let mut tar_stdin = tar.stdin.take().context("tar stdin was not piped")?;
    io::copy(&mut archive, &mut tar_stdin).context("Failed to stream git archive into tar")?;
    drop(tar_stdin);

    let tar_output = tar.wait_with_output().context("Failed to finish tar extraction")?;

    if !tar_output.status.success() {
        bail!(
            "Failed to extract source archive into build context {}\n{}",
            context_dir.display(),
            String::from_utf8_lossy(&tar_output.stderr)
        );
    }

    println!("Exported source for {revision} into {}", context_dir.display());
    Ok(())
}

pub(crate) fn ensure_build_context(site: &str) -> Result<PathBuf> {
    let root = resolved_tmp_root(site)?;
    fs::create_dir_all(&root).with_context(|| format!("Failed to create tmp builds root: {}", root.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let context = root.join(format!("build-{site}-{nanos}"));
    fs::create_dir_all(&context).with_context(|| format!("Failed to create build context {}", context.display()))?;
    Ok(context)
}

pub fn cleanup_build_context(site: &str, context: &Path) -> Result<()> {
    if context.exists() {
        fs::remove_dir_all(context).with_context(|| format!("Failed to remove build context {}", context.display()))?;
    }
    let root = if let Some(parent) = context.parent() { parent.to_path_buf() } else { resolved_tmp_root(site)? };
    if root.exists() && fs::read_dir(&root)?.next().is_none() {
        fs::remove_dir(&root).ok();
    }
    Ok(())
}
