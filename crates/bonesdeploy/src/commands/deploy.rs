use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use tokio::fs::File;

use crate::build;
use crate::commands::secrets;
use crate::config;
use crate::infra::{self, ssh};
use crate::ui::output;
use bonesdeploy_core::artifact::encode_manifest;
use bonesdeploy_core::paths;

pub fn local_bones_load_error() -> String {
    format!("Failed to load root {}", paths::DOT_ENV)
}

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::DOT_ENV);
    let cfg = config::load(bones_toml).context(local_bones_load_error())?;

    println!(
        "{} {} {} {}",
        style("Deploying").cyan().bold(),
        style(&cfg.project_name).bold(),
        style("to").dim(),
        style(&cfg.host).dim(),
    );

    if !secrets::production_secrets_exist(&cfg).await? {
        println!("Production secrets are missing; pushing them now...");
        secrets::push().await?;
    }

    println!("Building the committed {} branch locally...", cfg.branch);
    let artifact = build::package(&cfg)?;
    deploy_artifact(&cfg, &artifact).await?;

    println!("{} Deployment complete.", output::success_marker());
    Ok(())
}

async fn deploy_artifact(cfg: &config::Bones, artifact: &build::PackagedArtifact) -> Result<()> {
    let frame = encode_manifest(&artifact.manifest)?;

    let session = ssh::connect(cfg).await?;
    infra::sync_control_plane(&session, cfg).await?;
    let file = File::open(artifact.path()).await.context("Failed to open local artifact for upload")?;
    ssh::stream_cmd_with_reader(&session, &infra::artifact_deploy_command(&cfg.project_name), &frame, file).await?;
    session.close().await?;
    Ok(())
}
