use std::env;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use bonesdeploy::build::{self, PackagedArtifact};
use bonesdeploy::commands::deploy::{DeployOperations, DeployWorkflow};
use bonesdeploy_core::config::Bones;

const HELPER: &str = "BONESDEPLOY_DEPLOY_WORKFLOW_HELPER";

#[tokio::test]
async fn package_failure_prevents_all_remote_deploy_operations() -> Result<()> {
    if env::var_os(HELPER).is_none() {
        let project = tempfile::tempdir()?;
        initialize_project(project.path())?;
        return run_helper("package_failure_prevents_all_remote_deploy_operations", project.path());
    }

    let events = Arc::new(Mutex::new(Vec::new()));
    let mut workflow = DeployWorkflow::new(RecordingDeployOperations::new(Arc::clone(&events)));
    let mut config = Bones::for_site("deploy-workflow-test");
    config.branch = "missing".into();

    let error = match workflow.run(&config).await {
        Ok(()) => anyhow::bail!("missing branch must prevent packaging"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("missing"));
    assert_eq!(
        events.lock().map_err(|error| anyhow::anyhow!("event log lock poisoned: {error}"))?.as_slice(),
        ["package"]
    );
    Ok(())
}

#[tokio::test]
async fn successful_package_precedes_and_survives_remote_deploy_operations() -> Result<()> {
    if env::var_os(HELPER).is_none() {
        let project = tempfile::tempdir()?;
        initialize_project(project.path())?;
        return run_helper("successful_package_precedes_and_survives_remote_deploy_operations", project.path());
    }

    let events = Arc::new(Mutex::new(Vec::new()));
    let mut workflow = DeployWorkflow::new(RecordingDeployOperations::new(Arc::clone(&events)));
    let mut config = Bones::for_site("deploy-workflow-test");
    config.branch = "main".into();

    workflow.run(&config).await?;

    assert_eq!(
        events.lock().map_err(|error| anyhow::anyhow!("event log lock poisoned: {error}"))?.as_slice(),
        ["package", "handle_production_secrets", "sync_control_plane", "upload_artifact"]
    );
    Ok(())
}

struct RecordingDeployOperations {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl RecordingDeployOperations {
    fn new(events: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self { events }
    }

    fn record(&self, event: &'static str) -> Result<()> {
        self.events.lock().map_err(|error| anyhow::anyhow!("event log lock poisoned: {error}"))?.push(event);
        Ok(())
    }
}

impl DeployOperations for RecordingDeployOperations {
    fn package(&mut self, config: &Bones) -> Result<PackagedArtifact> {
        self.record("package")?;
        build::package(config)
    }

    fn handle_production_secrets(&mut self, _config: &Bones) -> impl Future<Output = Result<()>> + Send {
        async move { self.record("handle_production_secrets") }
    }

    fn sync_control_plane(&mut self, _config: &Bones) -> impl Future<Output = Result<()>> + Send {
        async move { self.record("sync_control_plane") }
    }

    fn upload_artifact(
        &mut self,
        _config: &Bones,
        artifact: &PackagedArtifact,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            anyhow::ensure!(artifact.path().is_file(), "upload must receive the packaged temporary artifact");
            self.record("upload_artifact")
        }
    }
}

fn run_helper(test_name: &str, project: &Path) -> Result<()> {
    let output = Command::new(env::current_exe()?)
        .args(["--exact", test_name, "--nocapture"])
        .current_dir(project)
        .env(HELPER, "1")
        .output()
        .context("failed to run deploy workflow helper")?;
    anyhow::ensure!(
        output.status.success(),
        "helper failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--initial-branch=main"])?;
    run_git(project, ["config", "user.email", "test@example.com"])?;
    run_git(project, ["config", "user.name", "Test"])?;
    fs::write(project.join("application.txt"), "application")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["commit", "-m", "initial"])
}

fn run_git<const N: usize>(project: &Path, arguments: [&str; N]) -> Result<()> {
    let status = Command::new("git").arg("-C").arg(project).args(arguments).status().context("failed to run git")?;
    anyhow::ensure!(status.success(), "git command failed with status {status}");
    Ok(())
}
