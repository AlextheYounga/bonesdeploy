use std::future::Future;
use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::artifact::encode_manifest;
use bonesdeploy_core::paths;
use console::style;
use tokio::fs::File;

use crate::build::{self, PackagedArtifact};
use crate::commands::secrets;
use crate::config::{self, Bones};
use crate::infra::{self, ssh};
use crate::ui::{output, progress::ProgressReader};

pub trait DeployOperations {
    fn package(&mut self, config: &Bones) -> Result<PackagedArtifact>;

    fn handle_production_secrets(&mut self, config: &Bones) -> impl Future<Output = Result<()>> + Send;

    fn sync_control_plane(&mut self, config: &Bones) -> impl Future<Output = Result<()>> + Send;

    fn upload_artifact(
        &mut self,
        config: &Bones,
        artifact: &PackagedArtifact,
    ) -> impl Future<Output = Result<()>> + Send;
}

pub struct DeployWorkflow<O> {
    operations: O,
}

impl<O> DeployWorkflow<O> {
    pub fn new(operations: O) -> Self {
        Self { operations }
    }
}

impl<O: DeployOperations> DeployWorkflow<O> {
    pub async fn run(&mut self, config: &Bones) -> Result<()> {
        let artifact = self.operations.package(config)?;
        self.operations.handle_production_secrets(config).await?;
        self.operations.sync_control_plane(config).await?;
        self.operations.upload_artifact(config, &artifact).await
    }
}

struct ProductionDeployOperations {
    session: Option<ssh::SshTransport>,
}

impl ProductionDeployOperations {
    fn new() -> Self {
        Self { session: None }
    }
}

impl DeployOperations for ProductionDeployOperations {
    fn package(&mut self, config: &Bones) -> Result<PackagedArtifact> {
        println!("Building the committed {} branch locally...", config.branch);
        let artifact = build::package(config)?;
        println!("Artifact size: {} bytes.", artifact.manifest.artifact_length);
        Ok(artifact)
    }

    fn handle_production_secrets(&mut self, config: &Bones) -> impl Future<Output = Result<()>> + Send {
        async move {
            if !secrets::production_secrets_exist(config).await? {
                println!("Production secrets are missing; pushing them now...");
                secrets::push().await?;
            }
            Ok(())
        }
    }

    fn sync_control_plane(&mut self, config: &Bones) -> impl Future<Output = Result<()>> + Send {
        async move {
            let session = ssh::SshTransport::connect(config).await?;
            infra::sync_control_plane(&session, config).await?;
            self.session = Some(session);
            Ok(())
        }
    }

    fn upload_artifact(
        &mut self,
        config: &Bones,
        artifact: &PackagedArtifact,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            let frame = encode_manifest(&artifact.manifest)?;
            let session = self.session.take().context("Deployment SSH session is not available for artifact upload")?;
            let file = File::open(artifact.path()).await.context("Failed to open local artifact for upload")?;
            let reader = ProgressReader::new(file, artifact.manifest.artifact_length, |percentage| {
                output::upload_progress("artifact", percentage);
            });
            let upload = session
                .stream_cmd_with_reader(&infra::artifact_deploy_command(&config.project_name), &frame, reader)
                .await;
            if upload.is_err() {
                println!();
            }
            upload?;
            session.close().await?;
            Ok(())
        }
    }
}

pub fn local_bones_load_error() -> String {
    format!("Failed to load root {}", paths::DOT_ENV)
}

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::DOT_ENV);
    let config = config::load(bones_toml).context(local_bones_load_error())?;

    println!(
        "{} {} {} {}",
        style("Deploying").cyan().bold(),
        style(&config.project_name).bold(),
        style("to").dim(),
        style(&config.host).dim(),
    );

    DeployWorkflow::new(ProductionDeployOperations::new()).run(&config).await?;

    println!("{} Deployment complete.", output::success_marker());
    Ok(())
}
