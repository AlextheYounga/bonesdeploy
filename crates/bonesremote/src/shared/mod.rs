mod archive;
mod environment;
mod filesystem;
mod transaction;

use std::io::Read;

use anyhow::Result;

use crate::release::SiteMutation;

pub use archive::{ArchiveLimits, extract_archive};
pub use filesystem::exchange_directories;
pub use transaction::{ImportTransaction, SiteLifecycle};

pub fn import(mutation: &SiteMutation, reader: &mut dyn Read) -> Result<()> {
    transaction::run(mutation, reader)
}

pub fn install_environment(mutation: &SiteMutation, reader: &mut dyn Read) -> Result<()> {
    transaction::recover(mutation)?;
    environment::install(mutation, reader)
}
