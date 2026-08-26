pub mod config;
pub mod deploy;
pub mod doctor;
pub mod drop_failed_release;
pub mod release;
pub mod service;
pub mod status;
pub mod version;

pub use crate::cli::args::Cli;
pub use crate::cli::dispatch::run;

use anyhow::{Result, bail};

use crate::release::SiteMutation;

pub fn ensure_site_idle(mutation: &SiteMutation) -> Result<()> {
    let state = mutation.state()?;

    // A pre-cut-over record (or a failed one being aborted) means a deployment
    // is in flight or interrupted. A committed record (`activated` and later)
    // is serving traffic and never blocks the next mutation.
    if let Some(active) = state.active() {
        if !active.phase().is_committed() {
            bail!(
                "Release {} is still active or interrupted. Run 'bonesdeploy site releases' and cancel it before changing site state.",
                active.release()
            );
        }
    }

    // Staging without a committed deployment is an interrupted/failed staging
    // attempt that must be resolved (e.g. `release drop-failed`) first.
    let committed = state.active().is_some_and(|active| active.phase().is_committed());
    if let Some(staged) = state.staged_release() {
        if !committed {
            bail!(
                "Release {staged} is staged without an active deployment. Run 'bonesdeploy site releases' before changing site state."
            );
        }
    }
    Ok(())
}
