use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use bonesdeploy_core::config::{Bones, RuntimeBackend};

pub mod artifact;
pub(crate) mod command;
pub mod compose;
pub mod docker;
pub mod inventory;
mod local_lock;
pub mod native;
mod native_inventory;
mod native_platform;
pub mod source;

pub use artifact::PackagedArtifact;
pub use local_lock::LocalBuildLock;

/// Produces the configured committed revision's uploadable build artifact.
pub fn package(config: &Bones) -> Result<PackagedArtifact> {
    let _lock = LocalBuildLock::acquire(&config.project_name)?;
    let mut context = source::export(config)?;
    match config.runtime.backend {
        RuntimeBackend::Native => {
            if native_build_scripts(config, context.path())?.is_none() {
                return artifact::package(&config.project_name, &context);
            }
            let docker = docker::DockerClient::new()?;
            native::build(config, &mut context, &docker)?;
            artifact::package(&config.project_name, &context)
        }
        RuntimeBackend::Docker => {
            let docker = docker::DockerClient::new()?;
            let images = compose::build(config, &mut context, &docker)?;
            artifact::package_compose(&config.project_name, &context, images)
        }
    }
}

fn native_build_scripts(config: &Bones, source: &Path) -> Result<Option<(PathBuf, Vec<PathBuf>)>> {
    let scripts = native::build_scripts(source)?;
    if scripts.is_none() && !config.runtime.template.is_empty() && config.runtime.template != "custom" {
        bail!(
            "Committed deploy branch '{}' has no native build scripts for the '{}' framework. Run `bonesdeploy update`, commit infra/deployment, and merge it into the deploy branch.",
            config.branch,
            config.runtime.template
        );
    }
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use bonesdeploy_core::config::Bones;

    use super::native_build_scripts;

    #[test]
    fn built_in_native_framework_requires_committed_build_scripts() -> anyhow::Result<()> {
        let source = tempfile::tempdir()?;
        let mut config = Bones::default();
        config.branch = String::from("master");
        config.runtime.template = String::from("laravel");

        let Err(error) = native_build_scripts(&config, source.path()) else {
            anyhow::bail!("missing Laravel build scripts must fail");
        };

        assert!(error.to_string().contains("Committed deploy branch 'master' has no native build scripts"));
        Ok(())
    }

    #[test]
    fn custom_native_framework_may_package_without_build_scripts() -> anyhow::Result<()> {
        let source = tempfile::tempdir()?;
        let mut config = Bones::default();
        config.runtime.template = String::from("custom");

        assert!(native_build_scripts(&config, source.path())?.is_none());
        Ok(())
    }
}
