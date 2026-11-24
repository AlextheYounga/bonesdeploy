use anyhow::Result;
use std::io;

use crate::privileges;
use crate::release::lifecycle;

use super::coordinator;

pub fn run_artifact(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote artifact deploy")?;
    let mut input = io::stdin().lock();
    let (mutation, manifest) = lifecycle::request::receive(site, &mut input)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, manifest.revision.clone());
    coordinator::run_staged_deployment(&mutation, snapshot, &mut input, &manifest)
}
