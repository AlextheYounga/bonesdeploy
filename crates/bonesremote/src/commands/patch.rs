use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, id};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::validate_site_name;
use bonesdeploy_core::paths;

use crate::privileges;

const CONFIG_REPO_HOOK: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/hooks/config-pre-receive"));
const PATCH_ROOT: &str = "/var/lib/bonesdeploy/patches";

pub fn apply(site: &str, patch: &str) -> Result<()> {
    privileges::ensure_root("bonesremote patch apply")?;
    validate_site_name(site)?;

    if !matches!(patch, "0001-config-repo" | "0002-root-config-repo") {
        bail!("Unknown remote patch {patch}");
    }

    let marker_dir = Path::new(PATCH_ROOT).join(site);
    let marker = marker_dir.join(patch);
    if marker.exists() {
        return Ok(());
    }

    migrate_config_repo(site)?;
    write_marker(&marker_dir, &marker)
}

fn migrate_config_repo(site: &str) -> Result<()> {
    let paths = ConfigRepositoryPaths {
        repository: PathBuf::from(paths::default_bones_repo_path_for(site)),
        previous_repository: Path::new(paths::DEFAULT_REPO_PARENT).join(format!("{site}.bones.git")),
    };
    migrate_config_repo_at(&paths, true)
}

struct ConfigRepositoryPaths {
    repository: PathBuf,
    previous_repository: PathBuf,
}

fn migrate_config_repo_at(paths: &ConfigRepositoryPaths, set_root_ownership: bool) -> Result<()> {
    let parent = paths.repository.parent().context("config repository has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;

    if !paths.repository.exists() && paths.previous_repository.exists() {
        fs::rename(&paths.previous_repository, &paths.repository).with_context(|| {
            format!("Failed to migrate {} to {}", paths.previous_repository.display(), paths.repository.display())
        })?;
    }
    let repository = paths.repository.to_str().context("config repository path is not UTF-8")?;
    if !paths.repository.is_dir() {
        run_git(["init", "--bare", repository])?;
    }
    if set_root_ownership {
        run_command("chown", ["-R", "root:root", repository])?;
    }
    run_git(["--git-dir", repository, "symbolic-ref", paths::GIT_HEAD, "refs/heads/master"])?;
    write_hook(&paths.repository.join(paths::HOOKS_DIR).join("pre-receive"))
}

fn write_hook(target: &Path) -> Result<()> {
    fs::write(target, CONFIG_REPO_HOOK).with_context(|| format!("Failed to write {}", target.display()))?;
    let mut permissions =
        fs::metadata(target).with_context(|| format!("Failed to stat {}", target.display()))?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(target, permissions).with_context(|| format!("Failed to chmod {}", target.display()))
}

fn run_git<const N: usize>(args: [&str; N]) -> Result<()> {
    run_command("git", args)
}

fn run_command<const N: usize>(program: &str, args: [&str; N]) -> Result<()> {
    let output = Command::new(program).args(args).output().with_context(|| format!("Failed to run {program}"))?;
    if output.status.success() {
        return Ok(());
    }
    bail!("{program} failed: {}", String::from_utf8_lossy(&output.stderr).trim())
}

fn write_marker(marker_dir: &Path, marker: &Path) -> Result<()> {
    fs::create_dir_all(marker_dir)
        .with_context(|| format!("Failed to create patch marker directory {}", marker_dir.display()))?;
    let temporary = marker.with_extension(format!("tmp-{}", id()));
    fs::write(&temporary, b"completed\n")
        .with_context(|| format!("Failed to write patch marker {}", temporary.display()))?;
    fs::rename(&temporary, marker).with_context(|| format!("Failed to activate patch marker {}", marker.display()))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::{self, Command};
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{ConfigRepositoryPaths, migrate_config_repo_at, write_hook};

    #[test]
    fn config_repo_hook_is_executable_and_receives_master_pushes() -> Result<()> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        let directory = env::temp_dir().join(format!("bonesremote_patch_{}_{}", process::id(), stamp));
        fs::create_dir(&directory)?;
        let hook = directory.join("pre-receive");

        write_hook(&hook)?;

        let content = fs::read_to_string(&hook)?;
        assert!(content.contains("bonesremote site receive --site \"$SITE\" --revision \"$newrev\""));
        assert_eq!(fs::metadata(&hook)?.permissions().mode() & 0o111, 0o111);
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn migration_moves_previous_repository_and_installs_the_config_hook() -> Result<()> {
        let directory = temp_directory("migration")?;
        let previous_repository = directory.join("atlas.bones.git");
        let repository = directory.join("repos").join("atlas.bones.git");
        let status = Command::new("git").args(["init", "--bare"]).arg(&previous_repository).status()?;
        assert!(status.success());
        let paths =
            ConfigRepositoryPaths { repository: repository.clone(), previous_repository: previous_repository.clone() };

        migrate_config_repo_at(&paths, false)?;

        assert!(repository.is_dir());
        assert!(!previous_repository.exists());
        assert!(repository.join("hooks/pre-receive").is_file());
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    fn temp_directory(name: &str) -> Result<PathBuf> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        let directory = env::temp_dir().join(format!("bonesremote_patch_{}_{}_{}", process::id(), stamp, name));
        fs::create_dir(&directory)?;
        Ok(directory)
    }
}
