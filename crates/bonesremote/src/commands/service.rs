use anyhow::Result;

use crate::release::SiteMutation;
use crate::runtime::service;

pub fn restart(site: &str) -> Result<()> {
    let mutation = SiteMutation::acquire(site)?;
    service::run(&mutation)
}
