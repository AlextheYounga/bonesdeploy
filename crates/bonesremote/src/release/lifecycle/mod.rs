pub mod activate;
pub mod build;
pub mod checkout;
pub mod preflight;
pub mod prepare;
pub mod stage;
pub mod wire_shared;

use std::path::PathBuf;

use crate::release::SiteMutation;
use anyhow::{Context, Result, bail};
use bonesdeploy_core::{config, paths};

#[derive(Clone, Debug)]
pub struct DeploymentSnapshot {
    pub site: String,
    pub config: config::Bones,
    pub repo_path: PathBuf,
    pub project_root: PathBuf,
    pub revision: String,
    pub deployment_dir: PathBuf,
}

impl DeploymentSnapshot {
    pub fn new(mutation: &SiteMutation, revision: String, deployment_dir: PathBuf) -> Self {
        let site = mutation.site();
        let config = mutation.config();
        Self {
            site: site.to_string(),
            config: config.clone(),
            repo_path: PathBuf::from(&config.app.repo_path),
            project_root: PathBuf::from(&config.project_root),
            revision,
            deployment_dir,
        }
    }

    pub fn with_deployment_dir(mut self, deployment_dir: PathBuf) -> Self {
        self.deployment_dir = deployment_dir;
        self
    }
}

/// Loads the deployed site configuration and verifies that it belongs to the named site.
///
/// Every command that writes or reads site state as root must call this rather
/// than loading config directly, so the confused-deputy check (`project_name` ==
/// site) is applied consistently.
pub fn load_site_config(site: &str) -> Result<config::Bones> {
    config::validate_site_name(site)?;
    let config_path = paths::bonesremote_site_config_path(site);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load deployed site configuration from {}", config_path.display()))?;
    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }
    Ok(cfg)
}
