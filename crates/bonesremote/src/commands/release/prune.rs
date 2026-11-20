use anyhow::Result;

use crate::privileges;
use crate::release::{SiteMutation, prune};

pub fn run(site: &str, keep: usize) -> Result<()> {
    privileges::ensure_root("bonesremote release prune")?;
    let mutation = SiteMutation::acquire(site)?;
    prune::run_locked(&mutation, keep)
}
