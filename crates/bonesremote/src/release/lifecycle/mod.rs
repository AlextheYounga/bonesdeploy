pub mod activate;
pub mod artifact;
pub mod build;
pub mod context;
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
    pub project_root: PathBuf,
    pub revision: String,
    pub deployment_dir: PathBuf,
}

impl DeploymentSnapshot {
    pub fn new(mutation: &SiteMutation, revision: String) -> Self {
        let site = mutation.site();
        let config = mutation.config();
        Self {
            site: site.to_string(),
            config: config.clone(),
            project_root: PathBuf::from(&config.project_root),
            revision,
            deployment_dir: PathBuf::new(),
        }
    }

    pub fn with_deployment_dir(mut self, deployment_dir: PathBuf) -> Self {
        self.deployment_dir = deployment_dir;
        self
    }
}
