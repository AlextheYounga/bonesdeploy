use std::io::{Read, stdin};

use anyhow::{Context, Result};
use bonesdeploy_core::config::validate_site_name;

use crate::privileges;
use crate::release::state::{self, DeletionPlan, DeploymentLock};

pub fn begin(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote decommission begin")?;
    validate_site_name(site)?;
    let plan = read_plan()?;
    let _lock = DeploymentLock::acquire(site)?;
    let plan = state::begin_decommission(site, plan)?;
    println!("{}", serde_json::to_string(plan.value()).context("Failed to serialize deletion plan")?);
    Ok(())
}

pub fn complete(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote decommission complete")?;
    transition(site, state::complete_decommission)
}

pub fn verify(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote decommission verify")?;
    transition(site, state::verify_decommission)
}

pub fn reactivate(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote decommission reactivate")?;
    transition(site, state::reactivate_decommission)
}

fn transition(site: &str, operation: impl FnOnce(&str) -> Result<()>) -> Result<()> {
    validate_site_name(site)?;
    let _lock = DeploymentLock::acquire(site)?;
    operation(site)
}

fn read_plan() -> Result<DeletionPlan> {
    let mut input = String::new();
    stdin().read_to_string(&mut input).context("Failed to read deletion plan from stdin")?;
    let plan = serde_json::from_str(&input).context("Failed to parse deletion plan from stdin as JSON")?;
    Ok(DeletionPlan::new(plan))
}
