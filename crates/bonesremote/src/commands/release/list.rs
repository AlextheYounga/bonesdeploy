use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use shared::config;
use shared::paths;

use crate::privileges;
use crate::release::state::{self as release_state, ActiveDeployment, DeploymentPhase};

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
    let cfg =
        config::load(&paths::bonesremote_bones_toml_path(site)).context("Failed to load release site configuration")?;
    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{site}'", cfg.project_name);
    }
    let current = release_state::current_release_name(&cfg.project_root).ok();
    let active = release_state::read_active_deployment(site)?;
    let staged = release_state::read_staged_release(site).ok();
    let mut releases = release_state::list_releases_sorted(&cfg.project_root)?
        .into_iter()
        .map(|name| release(&name, current.as_deref(), active.as_ref(), staged.as_deref()))
        .collect::<Vec<_>>();
    releases.sort_by(|left, right| right.name.cmp(&left.name));

    println!("{}", serde_json::to_string(&Report { releases })?);
    Ok(())
}

fn release(name: &str, current: Option<&str>, active: Option<&ActiveDeployment>, staged: Option<&str>) -> Release {
    if current == Some(name) {
        return Release { name: name.to_string(), status: String::from("active"), phase: None, started_at: None };
    }

    if let Some(active) = active.filter(|active| active.release == name) {
        let running = process_matches(active);
        return Release {
            name: name.to_string(),
            status: if running { phase_status(&active.phase) } else { String::from("interrupted") },
            phase: Some(active.phase.clone()),
            started_at: Some(active.started_at.clone()),
        };
    }

    if staged == Some(name) {
        return Release { name: name.to_string(), status: String::from("interrupted"), phase: None, started_at: None };
    }

    Release { name: name.to_string(), status: String::from("previous"), phase: None, started_at: None }
}

fn phase_status(phase: &DeploymentPhase) -> String {
    match phase {
        DeploymentPhase::Building => String::from("building"),
        DeploymentPhase::Preparing => String::from("preparing"),
    }
}

pub(crate) fn process_matches(active: &ActiveDeployment) -> bool {
    let stat = Path::new("/proc").join(active.pid.to_string()).join("stat");
    fs::read_to_string(stat)
        .ok()
        .and_then(|content| process_start_ticks(&content))
        .is_some_and(|ticks| ticks == active.process_start_ticks)
}

pub(crate) fn process_start_ticks(stat: &str) -> Option<u64> {
    let (_, fields) = stat.rsplit_once(") ")?;
    fields.split_whitespace().nth(19)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::process_start_ticks;

    #[test]
    fn parses_process_start_ticks_after_parenthesized_name() {
        let stat = "123 (bonesremote) R 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 99 20";
        assert_eq!(process_start_ticks(stat), Some(99));
    }
}
