use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::release::SiteMutation;
use crate::release::lifecycle;
use crate::release::state as release_state;

pub struct DeploymentLifecycleCoordinator<'a> {
    mutation: &'a SiteMutation,
    snapshot: lifecycle::DeploymentSnapshot,
}

impl<'a> DeploymentLifecycleCoordinator<'a> {
    pub fn new(mutation: &'a SiteMutation, snapshot: lifecycle::DeploymentSnapshot) -> Self {
        Self { mutation, snapshot }
    }

    pub fn run(self) -> Result<()> {
        let release = begin(self.mutation.site(), &self.snapshot.revision)?;
        self.mutation.set_staged_release(&release)?;

        let identity =
            release_state::ProcessIdentity::new(process::id(), process_start_ticks()?, deployment_started_at()?);
        let mut deployment = release_state::DeploymentRecord::new(
            release.clone(),
            self.snapshot.revision.clone(),
            release_state::DeploymentPhase::Created,
            identity,
        );
        self.mutation.set_active(&deployment)?;

        let context = match lifecycle::checkout::ensure_build_context(&self.snapshot, &release) {
            Ok(context) => context,
            Err(error) => return abort(self.mutation, &release, error),
        };
        deployment.set_context(context.display().to_string());
        self.mutation.set_active(&deployment)?;
        let snapshot =
            self.snapshot.with_deployment_dir(context.join(paths::LOCAL_INFRA_DIR).join(paths::DEPLOYMENT_DIR));

        if let Err(error) = lifecycle::checkout::run(&snapshot, &context) {
            return abort(self.mutation, &release, error);
        }
        deployment.set_phase(release_state::DeploymentPhase::SourceExported);
        self.mutation.set_active(&deployment)?;

        if let Err(error) = transition("prepare", self.mutation.site(), &release) {
            return abort(self.mutation, &release, error);
        }
        deployment.set_phase(release_state::DeploymentPhase::Sealed);
        self.mutation.set_active(&deployment)?;

        if let Err(error) = transition("commit", self.mutation.site(), &release) {
            return abort(self.mutation, &release, error);
        }
        deployment.set_phase(release_state::DeploymentPhase::Verified);
        self.mutation.set_active(&deployment)?;

        if let Err(error) = transition("complete", self.mutation.site(), &release) {
            deployment.set_phase(release_state::DeploymentPhase::CleanupPending);
            deployment.set_error(error.to_string());
            self.mutation.set_active(&deployment)?;
            eprintln!("Warning: release {release} is active, but post-deploy maintenance is pending: {error:#}");
            return Ok(());
        }

        self.mutation.clear_staged_release()?;
        self.mutation.clear_active()
    }
}

fn begin(site: &str, revision: &str) -> Result<String> {
    let output = Command::new("/usr/bin/sudo")
        .args([
            "-n",
            "/usr/local/bin/bonesremote",
            "deploy-transition",
            "begin",
            "--site",
            site,
            "--revision",
            revision,
        ])
        .stderr(Stdio::inherit())
        .output()
        .context("Failed to start privileged deployment begin transition")?;
    if !output.status.success() {
        bail!("Privileged deployment begin transition failed with status {}", output.status);
    }
    let release = String::from_utf8(output.stdout).context("Privileged deployment begin returned invalid UTF-8")?;
    let release = release.trim();
    crate::commands::deploy::transitions::validate_release(release)?;
    Ok(release.to_owned())
}

fn transition(action: &str, site: &str, release: &str) -> Result<()> {
    let status = Command::new("/usr/bin/sudo")
        .args(["-n", "/usr/local/bin/bonesremote", "deploy-transition", action, "--site", site, "--release", release])
        .status()
        .with_context(|| format!("Failed to start privileged deployment {action} transition"))?;
    if !status.success() {
        bail!("Privileged deployment {action} transition failed with status {status}");
    }
    Ok(())
}

fn abort(mutation: &SiteMutation, release: &str, error: anyhow::Error) -> Result<()> {
    let abort_error = transition("abort", mutation.site(), release).err();
    mutation.clear_staged_release()?;
    mutation.clear_active()?;
    match abort_error {
        Some(abort_error) => Err(error.context(format!("Failed to clean up the candidate: {abort_error:#}"))),
        None => Err(error),
    }
}

pub fn restore_previous_release(project_root: &Path, previous_release: &Path) -> Result<()> {
    let current_link = PathBuf::from(project_root).join(paths::CURRENT_LINK);
    release_state::point_symlink_atomically(&current_link, previous_release)
}

fn process_start_ticks() -> Result<u64> {
    crate::commands::release::list::current_process_start_ticks()
        .context("Failed to read deployment process start time")
}

fn deployment_started_at() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc().format(TIMESTAMP_FORMAT).context("Failed to format deployment start time")
}
