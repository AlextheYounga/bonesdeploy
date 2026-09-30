use anyhow::Result;
use bonesdeploy_core::config::{Bones, RuntimeBackend};

pub mod artifact;
pub mod compose;
mod local_lock;
pub mod native;
pub mod source;

pub use artifact::PackagedArtifact;
pub use local_lock::LocalBuildLock;

/// Produces the configured committed revision's uploadable build artifact.
pub fn package(config: &Bones) -> Result<PackagedArtifact> {
    let _lock = LocalBuildLock::acquire(&config.project_name)?;
    let context = source::export(config)?;
    match config.runtime.backend {
        RuntimeBackend::Native => {
            native::build(config, &context)?;
            artifact::package(&config.project_name, &context)
        }
        RuntimeBackend::Docker => {
            let images = compose::build(config, &context)?;
            artifact::package_compose(&config.project_name, &context, images)
        }
    }
}
