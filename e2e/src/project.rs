//! A minimal sample repository to run bonesdeploy commands against.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::session::Session;
use crate::{keep_artifacts, scratch_dir, status_ok, unique_suffix};

pub struct SampleProject {
    dir: PathBuf,
    keep: bool,
}

impl SampleProject {
    /// Creates a git repository with one commit on `main`.
    pub fn create(session: &Session) -> Result<Self> {
        let dir = scratch_dir().join(format!("project-{}", unique_suffix()));
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
        fs::write(dir.join("index.html"), "<h1>bones e2e</h1>\n").context("Failed to write sample file")?;

        let project = Self { dir, keep: keep_artifacts() };
        project.git(session, &["init"])?;
        project.git(session, &["add", "-A"])?;
        project.git(session, &["commit", "-m", "initial commit"])?;
        Ok(project)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Runs the given bonesdeploy binary in the project directory with output
    /// streaming to the terminal.
    pub fn bonesdeploy(&self, session: &Session, binary: &Path, args: &[&str]) -> Result<()> {
        let status = session
            .command(binary)
            .current_dir(&self.dir)
            .args(args)
            .status()
            .with_context(|| format!("Failed to run bonesdeploy {}", args.join(" ")))?;
        status_ok(status, &format!("bonesdeploy {}", args.join(" ")))
    }

    fn git(&self, session: &Session, args: &[&str]) -> Result<()> {
        let status = session
            .command("git")
            .current_dir(&self.dir)
            .args(args)
            .status()
            .with_context(|| format!("Failed to run git {}", args.join(" ")))?;
        status_ok(status, &format!("git {}", args.join(" ")))
    }
}

impl Drop for SampleProject {
    fn drop(&mut self) {
        if self.keep {
            eprintln!("{}: keeping sample project {} for inspection", crate::KEEP_ENV, self.dir.display());
            return;
        }
        if let Err(err) = fs::remove_dir_all(&self.dir) {
            eprintln!("Failed to clean up sample project {}: {err}", self.dir.display());
        }
    }
}
