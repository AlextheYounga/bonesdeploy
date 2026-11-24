use anyhow::Result;
use bonesdeploy_core::config::RemoteDeploymentConfig;

use crate::release::SiteMutation;
use crate::{control_plane, privileges};

pub fn sync(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote config sync")?;
    let descriptor = control_plane::read_stdin_descriptor()?;
    sync_descriptor(site, &descriptor)
}

fn sync_descriptor(site: &str, descriptor: &RemoteDeploymentConfig) -> Result<()> {
    let mutation = SiteMutation::acquire(site)?;
    mutation.ensure_idle()?;
    control_plane::store(site, descriptor)
}
