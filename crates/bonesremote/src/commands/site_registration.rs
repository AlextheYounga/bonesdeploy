use anyhow::Result;
use bonesdeploy_core::config::validate_site_name;

use crate::privileges;
use crate::release::state::{self, DeploymentLock};

pub fn remove(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote remove-site")?;
    validate_site_name(site)?;
    let _lock = DeploymentLock::acquire(site)?;
    state::remove_site_registration(site)
}
