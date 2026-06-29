use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, thread_local};

use anyhow::{Context, Result, bail};
use shared::paths;
use shared::registry;

use crate::privileges;

thread_local! {
    static SITES_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
struct ScopedRoot(Option<PathBuf>);

#[cfg(test)]
fn set_sites_root_for_tests(root: PathBuf) -> ScopedRoot {
    let prev = SITES_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root)));
    ScopedRoot(prev)
}

#[cfg(test)]
impl Drop for ScopedRoot {
    fn drop(&mut self) {
        let previous = self.0.take();
        SITES_ROOT_OVERRIDE.with(|slot| {
            slot.replace(previous);
        });
    }
}

fn resolved_sites_root() -> PathBuf {
    SITES_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_sites_root)
}

fn resolved_tmp_root(site: &str) -> PathBuf {
    resolved_sites_root().join(site).join(paths::TMP_BUILDS_DIR)
}

pub fn run(site: &str, revision: &str, context_dir: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release checkout")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let archive_output = Command::new("git")
        .args(["--git-dir", &cfg.repo_path, "archive", "--format=tar", revision])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run git archive for revision {revision} in {}", cfg.repo_path))?;

    let mut archive = archive_output.stdout.context("git archive stdout was not piped")?;
    let stderr = archive_output.stderr.context("git archive stderr was not piped")?;

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
    let git_stderr = String::from_utf8_lossy(&archive_stderr_handle(stderr)?).into_owned();

    if !tar_output.status.success() {
        bail!(
            "Failed to extract source archive into build context {}\n{}",
            context_dir.display(),
            String::from_utf8_lossy(&tar_output.stderr)
        );
    }

    if !git_stderr.is_empty() {
        println!("[bonesdeploy] git archive reported: {git_stderr}");
    }

    println!("Exported source for {revision} into {}", context_dir.display());
    Ok(())
}

fn archive_stderr_handle<R: Read>(mut reader: R) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

pub(crate) fn ensure_build_context(site: &str) -> Result<PathBuf> {
    let root = resolved_tmp_root(site);
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
    let root = resolved_tmp_root(site);
    if root.exists() && fs::read_dir(&root)?.next().is_none() {
        fs::remove_dir(&root).ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use super::ensure_build_context;
    use super::run;
    use super::set_sites_root_for_tests;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_release_checkout_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    #[test]
    fn build_context_lives_under_bonesremote_tmp_root() -> Result<()> {
        let root = temp_dir_path("tmp_root");
        let _guard = set_sites_root_for_tests(root.clone());

        let context = ensure_build_context("unitapp")?;
        let expected_root = root.join("unitapp").join("tmp");
        assert!(context.starts_with(&expected_root));

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn run_reuses_supplied_context_path() -> Result<()> {
        let root = temp_dir_path("reuse_context");
        let _guard = set_sites_root_for_tests(root.clone());
        let context = ensure_build_context("unitapp")?;

        let site_root = root.join("unitapp");
        fs::create_dir_all(&site_root)?;
        fs::write(
            site_root.join(paths::REGISTRY_TOML),
            "site = \"unitapp\"\nrepo_path = \"/nope.git\"\nsite_root = \"/srv/sites/unitapp\"\nshared_root = \"/srv/sites/unitapp/shared\"\nreleases_root = \"/srv/sites/unitapp/releases\"\ncurrent_path = \"/srv/sites/unitapp/current\"\nruntime_user = \"unitapp\"\nruntime_group = \"unitapp\"\nbranch = \"main\"\ndeploy_on_push = true\nreleases_keep = 5\n",
        )?;

        let err = match run("unitapp", "main", &context) {
            Ok(()) => anyhow::bail!("missing repo should fail"),
            Err(error) => error,
        };
        let message = err.to_string();
        assert!(
            message.contains("git archive") || message.contains("/nope.git") || message.contains("must be run as root"),
            "unexpected error: {message}"
        );
        assert!(context.exists(), "caller-owned context should remain for cleanup");

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
