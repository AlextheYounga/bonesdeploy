use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use bonesinfra::ProvisioningContext;
use bonesinfra::manifest::{ArtifactKind, DeletionPlan};
use bonesinfra::provisioning::apply_site_delete;
use console::style;

use crate::config;
use crate::infra::{self, ssh};
use crate::ui::{output, prompts};

use crate::ui::manifest::{PathArtifact, render_path_tree};

pub async fn run(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let context = ProvisioningContext::from_bones(&cfg)
        .context("Site deletion preflight failed; no remote resources were changed")?;
    let plan =
        DeletionPlan::resolve(&context).context("Site deletion preflight failed; no remote resources were changed")?;

    println!("Remote site to delete: {}", cfg.project_name);
    print!("{}", render_preflight(&plan)?);
    if !yes && !prompts::confirm_site_delete(&cfg.project_name)? {
        println!("Skipped.");
        return Ok(());
    }

    let persisted_plan = plan.to_json()?;
    apply_site_delete(&cfg, Path::new("."), &persisted_plan)
        .await
        .context("Site deletion stopped before completion; rerun the command to resume")?;

    let session = ssh::SshTransport::connect_privileged(&cfg).await.context(
        "Site resources were removed, but BonesRemote registration cleanup could not connect; rerun the command",
    )?;
    session
        .run_cmd(&infra::remove_site_command(&cfg.project_name))
        .await
        .context("Site resources were removed, but BonesRemote registration cleanup failed; rerun the command")?;
    session.close().await.context(
        "Site resources and registration were removed, but the SSH session did not close cleanly; rerunning is safe",
    )?;

    println!("{} Site deletion complete.", output::success_marker());
    Ok(())
}

fn render_preflight(plan: &DeletionPlan) -> Result<String> {
    let mut lines = vec![String::new(), style("Resources to remove").cyan().bold().to_string()];
    lines.extend(render_path_tree(
        plan.artifacts
            .iter()
            .map(|artifact| PathArtifact {
                path: artifact.path.clone(),
                kind: artifact_kind(artifact.kind).to_owned(),
                state: None,
            })
            .collect(),
    )?);
    lines.extend([String::new(), style("Services to stop").cyan().bold().to_string()]);
    lines.extend(plan.services.iter().map(|service| format!("{} {}", style("•").cyan(), service.unit)));
    Ok(format!("{}\n\n", lines.join("\n")))
}

fn artifact_kind(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::File => "file",
        ArtifactKind::Directory => "directory",
        ArtifactKind::Link => "link",
        ArtifactKind::Socket => "socket",
    }
}
