pub(crate) mod deploy;
pub(crate) mod doctor;
pub(crate) mod drop_failed_release;
pub(crate) mod hook;
pub(crate) mod release;
pub(crate) mod service;
pub(crate) mod site;
pub(crate) mod status;
pub(crate) mod version;

pub use crate::cli::args::Cli;
pub use crate::cli::dispatch::run;

use anyhow::{Result, bail};

use crate::release::state as release_state;

fn ensure_site_idle(site: &str) -> Result<()> {
    if let Some(active) = release_state::read_active_deployment(site)? {
        bail!(
            "Release {} is still active or interrupted. Run 'bonesdeploy releases' and cancel it before changing site state.",
            active.release
        );
    }

    let staged_path = release_state::staged_release_path(site);
    if staged_path.exists() {
        let staged = release_state::read_staged_release(site)?;
        bail!(
            "Release {staged} is staged without an active deployment. Run 'bonesdeploy releases' before changing site state."
        );
    }
    Ok(())
}
