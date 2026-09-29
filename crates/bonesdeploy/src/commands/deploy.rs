use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use tokio::fs::File;

use crate::config;
use crate::infra::{self, ssh};
use crate::ui::output;
use crate::{artifact, compose_build, local_build};
use bonesdeploy_core::artifact::encode_manifest;
use bonesdeploy_core::config::RuntimeBackend;
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

    println!("Building the committed {} branch locally...", cfg.branch);
    let build = local_build::export(&cfg)?;
    let artifact = match cfg.runtime.backend {
        RuntimeBackend::Native => {
            local_build::build_native(&cfg, &build)?;
            artifact::package(&cfg.project_name, &build)?
        }
        RuntimeBackend::Docker => {
            let images = compose_build::build(&cfg, &build)?;
            artifact::package_compose(&cfg.project_name, &build, images)?
        }
    };
    deploy_artifact(&cfg, &artifact).await?;

    println!("{} Deployment complete.", output::success_marker());
    Ok(())
}

async fn deploy_artifact(cfg: &config::Bones, artifact: &artifact::PackagedArtifact) -> Result<()> {
    let frame = encode_manifest(&artifact.manifest)?;

    let session = ssh::connect(cfg).await?;
    infra::sync_control_plane(&session, cfg).await?;
    let file = File::open(artifact.path()).await.context("Failed to open local artifact for upload")?;
    ssh::stream_cmd_with_reader(&session, &infra::artifact_deploy_command(&cfg.project_name), &frame, file).await?;
    session.close().await?;
    Ok(())
}
