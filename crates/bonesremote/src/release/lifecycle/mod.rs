pub mod activate;
pub mod build;
pub mod checkout;
pub mod preflight;
pub mod prepare;
pub mod stage;
pub mod wire_shared;

use std::path::PathBuf;

use crate::release::SiteMutation;
use bonesdeploy_core::config;

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
