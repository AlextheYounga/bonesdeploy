use anyhow::Result;
use bonesdeploy_core::config::{Bones, RuntimeBackend};

pub mod artifact;
pub mod compose;
pub mod docker;
pub mod native;
pub mod source;

pub use artifact::PackagedArtifact;

/// Produces the configured committed revision's uploadable build artifact.
pub fn package(config: &Bones) -> Result<PackagedArtifact> {
    let context = source::export(config)?;
    let docker = docker::DockerClient::new()?;
    match config.runtime.backend {
        RuntimeBackend::Native => {
            native::build(config, &context, &docker)?;
            artifact::package(&config.project_name, &context)
        }
        RuntimeBackend::Docker => {
            let images = compose::build(config, &context, &docker)?;
            artifact::package_compose(&config.project_name, &context, images)
        }
    }
}
