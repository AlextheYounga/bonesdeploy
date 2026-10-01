use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use console::style;
use serde::Deserialize;

use crate::config;
use crate::infra::{self, ssh};
use crate::ui::{output, prompts};

use super::manifest::{PathArtifact, render_path_tree};

#[derive(Deserialize)]
struct DeletionPlan {
    artifacts: Vec<DeletionArtifact>,
    services: Vec<DeletionService>,
}

#[derive(Deserialize)]
struct DeletionArtifact {
    path: String,
    kind: String,
}

#[derive(Deserialize)]
struct DeletionService {
    unit: String,
}

pub async fn run(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    let preflight = bonesinfra::run_with_request_output(&["site", "preflight", "--request-stdin"], &request)
        .context("Site deletion preflight failed; no remote resources were changed")?;

    println!("Remote site to delete: {}", cfg.project_name);
    print!("{}", render_preflight(&preflight)?);
    if !yes && !prompts::confirm_site_delete(&cfg.project_name)? {
        println!("Skipped.");
        return Ok(());
    }

    let session = ssh::SshTransport::connect_privileged(&cfg).await?;
    let begin = infra::decommission_command("begin", &cfg.project_name);
    let plan = session.run_cmd_with_stdin_output(&begin, preflight.as_bytes()).await?;
    let plan = plan.trim();
    if plan.is_empty() {
        anyhow::bail!("BonesRemote returned an empty deletion plan");
    }

    let delete_args = ["site", "delete", "--request-stdin", "--plan-json", plan];
    bonesinfra::run_with_request(&delete_args, &request)
        .context("Site deletion stopped before completion; rerun the command to resume")?;

    let complete = infra::decommission_command("complete", &cfg.project_name);
    session.run_cmd(&complete).await?;
    let verify = infra::decommission_command("verify", &cfg.project_name);
    session.run_cmd(&verify).await?;
    session.close().await?;

    println!("{} Site deletion complete.", output::success_marker());
    Ok(())
}

fn render_preflight(preflight: &str) -> Result<String> {
    let plan: DeletionPlan =
        serde_json::from_str(preflight).context("BonesInfra returned an invalid deletion inventory")?;
    let mut lines = vec![String::new(), style("Resources to remove").cyan().bold().to_string()];
    lines.extend(render_path_tree(
        plan.artifacts
            .into_iter()
            .map(|artifact| PathArtifact { path: artifact.path, kind: artifact.kind, state: None })
            .collect(),
    )?);
    lines.extend([String::new(), style("Services to stop").cyan().bold().to_string()]);
    lines.extend(plan.services.into_iter().map(|service| format!("{} {}", style("•").cyan(), service.unit)));
    Ok(format!("{}\n\n", lines.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::output::strip_ansi;

    #[test]
    fn deletion_preflight_renders_resources_as_a_path_tree() -> Result<()> {
        let output = strip_ansi(&render_preflight(
            r#"{
                "artifacts": [
                    {"kind":"directory","name":"site root","owner":"setup","path":"/srv/sites/example"},
                    {"kind":"socket","name":"application socket","owner":"framework","path":"/run/example/app.sock"}
                ],
                "services": [
                    {"name":"application","owner":"framework","unit":"example-app.service"}
                ]
            }"#,
        )?);

        assert!(output.contains("Resources to remove\n/\n├── run/\n│   └── example/\n│       └── app.sock"));
        assert!(output.contains("└── srv/\n    └── sites/\n        └── example/"));
        assert!(output.contains("Services to stop\n• example-app.service"));
        assert!(!output.contains("owner"));
        Ok(())
    }
}
