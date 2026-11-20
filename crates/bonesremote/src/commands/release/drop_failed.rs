use anyhow::Result;

use crate::privileges;
use crate::release::{SiteMutation, failed};

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;
    let mutation = SiteMutation::acquire(site)?;
    failed::run_locked(&mutation)
}
