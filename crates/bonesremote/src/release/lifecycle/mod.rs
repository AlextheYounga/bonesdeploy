pub(crate) mod activate;
pub(crate) mod build;
pub(crate) mod checkout;
pub(crate) mod preflight;
pub(crate) mod prepare;
pub(crate) mod stage;
pub(crate) mod wire_shared;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::{config, paths};

/// Loads the site configuration and verifies that it belongs to the named site.
///
/// Every command that writes or reads site state as root must call this rather
/// than loading config directly, so the confused-deputy check (`project_name` ==
/// site) is applied consistently.
pub(crate) fn load_site_config(site: &str) -> Result<config::Bones> {
    config::validate_site_name(site)?;
    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;
    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }
    Ok(cfg)
}
