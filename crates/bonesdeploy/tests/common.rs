//! Command-level test support for the `bonesdeploy` binary.
//!
//! Runs the compiled executable as a child process with a throwaway git
//! repository and an isolated HOME and XDG roots, so no test depends on the
//! developer environment or leaks state between tests.

// Each integration-test target builds this module standalone, so helpers that
// some targets do not use would otherwise look like dead code.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};

static BONESINFRA_READY: OnceLock<Result<(), String>> = OnceLock::new();

/// Absolute path to the compiled `bonesdeploy` binary.
pub fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bonesdeploy")
}

/// A throwaway project workspace: a git repository paired with isolated HOME,
/// configuration, data, and state roots. The disposable Cargo cache is shared
/// so command tests bootstrap the embedded bonesinfra runtime only once.
pub struct TestEnv {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    home: PathBuf,
}

impl TestEnv {
    /// Creates a temporary git repository and an isolated HOME.
    pub fn new() -> Result<Self> {
        ensure_bonesinfra_ready()?;
        let temp = tempfile::TempDir::new().context("failed to create temp workspace")?;
        let repo = temp.path().join("repo");
        let home = temp.path().join("home");
        fs::create_dir_all(&repo).context("failed to create repo dir")?;
        fs::create_dir_all(&home).context("failed to create home dir")?;
        init_git(&repo)?;
        Ok(Self { _temp: temp, repo, home })
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Runs `bonesdeploy` with isolated user state and a shared disposable
    /// cache, returning its raw output.
    pub fn run(&self, args: &[&str]) -> Result<Output> {
        Command::new(binary())
            .args(args)
            .current_dir(&self.repo)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("XDG_DATA_HOME", self.home.join(".local/share"))
            .env("XDG_CACHE_HOME", shared_cache_home())
            .env("XDG_STATE_HOME", self.home.join(".local/state"))
            .output()
            .context("failed to run bonesdeploy")
    }
}

fn shared_cache_home() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("bonesdeploy-command-cache")
}

fn ensure_bonesinfra_ready() -> Result<()> {
    BONESINFRA_READY
        .get_or_init(|| {
            bonesinfra::prepare_in(&shared_cache_home().join("bonesdeploy")).map_err(|error| format!("{error:#}"))
        })
        .as_ref()
        .map(|_| ())
        .map_err(|error| anyhow!(error.clone()))
}

/// Initializes a fresh git repository on a `master` branch at `path`.
pub fn init_git(path: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["init", "--quiet", "--initial-branch", "master"])
        .current_dir(path)
        .status()
        .context("failed to run git init")?;
    if !status.success() {
        bail!("git init failed with status {status}");
    }
    Ok(())
}

/// Creates an initial empty commit at `path`, so the configured branch exists.
pub fn commit_initial(path: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["-C"])
        .arg(path)
        .args(["-c", "user.name=BonesDeploy", "-c", "user.email=bonesdeploy@local"])
        .args(["commit", "--allow-empty", "-m", "initial"])
        .status()
        .context("failed to create initial commit")?;
    if !status.success() {
        bail!("initial commit failed with status {status}");
    }
    Ok(())
}
