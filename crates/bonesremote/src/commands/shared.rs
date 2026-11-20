use std::io;

use anyhow::Result;

use crate::privileges;
use crate::release::SiteMutation;
use crate::shared;

pub fn import(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote shared import")?;
    let mutation = SiteMutation::acquire(site)?;
    mutation.ensure_idle()?;
    shared::import(&mutation, &mut io::stdin().lock())
}

pub fn install_environment(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote shared install-environment")?;
    let mutation = SiteMutation::acquire(site)?;
    mutation.ensure_idle()?;
    shared::install_environment(&mutation, &mut io::stdin().lock())
}
