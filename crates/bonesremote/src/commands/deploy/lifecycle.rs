use std::io;
use std::path::PathBuf;

use anyhow::Result;
use bonesdeploy_core::config::RuntimeBackend;

use crate::commands::ensure_site_idle;
use crate::control_plane;
use crate::git;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;

use super::coordinator::DeploymentLifecycleCoordinator;

pub fn run_source(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote Compose deploy")?;
    let bones = control_plane::load(site)?.into_site_config(site);
    source_deploy_allowed(bones.runtime.backend)?;
    let mutation = SiteMutation::acquire_with_config(site, bones)?;
    ensure_site_idle(&mutation)?;
    let repo_path = PathBuf::from(&mutation.config().repo_path);
    let revision = git::resolve_revision_commit(&repo_path, &mutation.config().branch)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, revision, PathBuf::new());
    DeploymentLifecycleCoordinator::new(&mutation, snapshot).run_with_source()
}

pub fn run_artifact(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote artifact deploy")?;
    let bones = control_plane::load(site)?.into_site_config(site);
    artifact_deploy_allowed(bones.runtime.backend)?;
    let mutation = SiteMutation::acquire_with_config(site, bones)?;
    ensure_site_idle(&mutation)?;
    let repo_path = PathBuf::from(&mutation.config().repo_path);
    let revision = git::resolve_revision_commit(&repo_path, &mutation.config().branch)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, revision, PathBuf::new());
    DeploymentLifecycleCoordinator::new(&mutation, snapshot).run_with_artifact(&mut io::stdin().lock())
}

fn source_deploy_allowed(backend: RuntimeBackend) -> Result<()> {
    if backend != RuntimeBackend::Docker {
        anyhow::bail!(
            "ordinary deploy is only supported for the Docker Compose runtime; native sites require --artifact-stdin"
        );
    }
    Ok(())
}

fn artifact_deploy_allowed(backend: RuntimeBackend) -> Result<()> {
    if backend != RuntimeBackend::Native {
        anyhow::bail!("artifact deploy requires the native runtime backend");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{artifact_deploy_allowed, source_deploy_allowed};
    use bonesdeploy_core::config::RuntimeBackend;

    #[test]
    fn ordinary_deploy_is_rejected_for_native_sites() {
        assert!(source_deploy_allowed(RuntimeBackend::Native).is_err());
        assert!(source_deploy_allowed(RuntimeBackend::Docker).is_ok());
    }

    #[test]
    fn artifact_deploy_is_rejected_for_compose_sites() {
        assert!(artifact_deploy_allowed(RuntimeBackend::Docker).is_err());
        assert!(artifact_deploy_allowed(RuntimeBackend::Native).is_ok());
    }
}
