pub(crate) mod activate;
pub(crate) mod build;
pub(crate) mod checkout;
pub(crate) mod preflight;
pub(crate) mod prepare;
pub(crate) mod stage;
pub(crate) mod wire_shared;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::{config, paths};

#[derive(Clone, Debug)]
pub(crate) struct DeploymentSnapshot {
    pub(crate) site: String,
    pub(crate) config: config::Bones,
    pub(crate) repo_path: PathBuf,
    pub(crate) project_root: PathBuf,
    pub(crate) revision: String,
    pub(crate) deployment_dir: PathBuf,
}

impl DeploymentSnapshot {
    pub(crate) fn new(site: &str, config: &config::Bones, revision: String, deployment_dir: PathBuf) -> Self {
        Self {
            site: site.to_string(),
            config: config.clone(),
            repo_path: PathBuf::from(paths::default_repo_path_for(site)),
            project_root: PathBuf::from(paths::default_project_root_for(site)),
            revision,
            deployment_dir,
        }
    }

    pub(crate) fn with_deployment_dir(mut self, deployment_dir: PathBuf) -> Self {
        self.deployment_dir = deployment_dir;
        self
    }
}

/// Loads the deployed site configuration and verifies that it belongs to the named site.
///
/// Every command that writes or reads site state as root must call this rather
/// than loading config directly, so the confused-deputy check (`project_name` ==
/// site) is applied consistently.
pub(crate) fn load_site_config(site: &str) -> Result<config::Bones> {
    config::validate_site_name(site)?;
    let config_path = PathBuf::from(paths::default_project_root_for(site)).join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load deployed site configuration from {}", config_path.display()))?;
    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bonesdeploy_core::config::Bones;

    use super::DeploymentSnapshot;

    #[test]
    fn snapshot_uses_convention_paths_and_one_revision() {
        let snapshot = DeploymentSnapshot::new("demo", &Bones::default(), "deadbeef".to_string(), PathBuf::new());

        assert_eq!(snapshot.repo_path, PathBuf::from("/home/git/demo.git"));
        assert_eq!(snapshot.project_root, PathBuf::from("/srv/sites/demo"));
        assert_eq!(snapshot.revision, "deadbeef");
        assert_eq!(snapshot.site, "demo");
    }
}
