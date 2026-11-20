use anyhow::Result;
use bonesdeploy_core::{config::validate_site_name, paths};
use serde::Serialize;

use crate::privileges;
use crate::release::process;
use crate::release::state::{self as release_state, DeploymentPhase, DeploymentRecord};

#[derive(Serialize)]
struct Report {
    releases: Vec<Release>,
}

#[derive(Serialize)]
struct Release {
    name: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<DeploymentPhase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    started_at: Option<String>,
}

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release list")?;
    validate_site_name(site)?;
    // Listing must remain available while deploy owns the mutation lock, so it
    // intentionally uses lock-free inspection with site-derived paths.
    let project_root = paths::default_project_root_for(site);
    let current = release_state::current_release_name(&project_root).ok();
    let active = release_state::read_active_deployment(site)?;
    let staged = release_state::read_staged_release(site).ok();
    let mut releases = release_state::list_releases_sorted(&project_root)?
        .into_iter()
        .map(|name| release(&name, current.as_deref(), active.as_ref(), staged.as_deref()))
        .collect::<Vec<_>>();
    releases.sort_by(|left, right| right.name.cmp(&left.name));

    println!("{}", serde_json::to_string(&Report { releases })?);
    Ok(())
}

fn release(name: &str, current: Option<&str>, active: Option<&DeploymentRecord>, staged: Option<&str>) -> Release {
    if current == Some(name) {
        return Release { name: name.to_string(), status: String::from("active"), phase: None, started_at: None };
    }

    if let Some(active) = active.filter(|active| active.release() == name) {
        let running = process::matches(active);
        return Release {
            name: name.to_string(),
            status: if running { phase_status(active.phase()) } else { String::from("interrupted") },
            phase: Some(active.phase().clone()),
            started_at: Some(active.started_at().to_string()),
        };
    }

    if staged == Some(name) {
        return Release { name: name.to_string(), status: String::from("interrupted"), phase: None, started_at: None };
    }

    Release { name: name.to_string(), status: String::from("previous"), phase: None, started_at: None }
}

fn phase_status(phase: &DeploymentPhase) -> String {
    match phase {
        DeploymentPhase::Created => String::from("created"),
        DeploymentPhase::Received => String::from("received"),
        DeploymentPhase::Materialized => String::from("materialized"),
        DeploymentPhase::Promoted => String::from("promoted"),
        DeploymentPhase::Prepared => String::from("prepared"),
        DeploymentPhase::Sealed => String::from("sealed"),
        DeploymentPhase::Activated => String::from("activated"),
        DeploymentPhase::Verified => String::from("verified"),
        DeploymentPhase::Completed => String::from("completed"),
        DeploymentPhase::CleanupPending => String::from("cleanup_pending"),
        DeploymentPhase::Failed => String::from("failed"),
    }
}
