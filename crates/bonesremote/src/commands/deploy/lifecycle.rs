use anyhow::Result;
use bonesdeploy_core::artifact::{ArtifactKind, ArtifactManifest, read_manifest};
use bonesdeploy_core::config::RuntimeBackend;
use std::io;

use crate::commands::ensure_site_idle;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;

use super::coordinator;

pub fn run_artifact(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote artifact deploy")?;
    let mutation = SiteMutation::acquire_for_deploy(site)?;
    ensure_site_idle(&mutation)?;
    let mut input = io::stdin().lock();
    let manifest = read_manifest(&mut input)?;
    if manifest.site != site {
        anyhow::bail!("artifact site does not match deploy site");
    }
    artifact_matches_backend(&manifest, mutation.config().runtime.backend)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, manifest.revision.clone());
    coordinator::run_staged_deployment(&mutation, snapshot, &mut input, &manifest)
}

fn artifact_matches_backend(manifest: &ArtifactManifest, backend: RuntimeBackend) -> Result<()> {
    let matches = matches!(
        (&manifest.kind, backend),
        (ArtifactKind::NativeTree, RuntimeBackend::Native)
            | (ArtifactKind::ComposeImages { .. }, RuntimeBackend::Docker)
    );
    if !matches {
        anyhow::bail!("artifact kind does not match the configured runtime backend");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use bonesdeploy_core::artifact::{ArtifactManifest, ComposeImage};

    use super::*;

    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn artifact_kind_must_match_the_runtime_backend() -> Result<()> {
        let native = ArtifactManifest::new_native_tree("atlas".into(), REVISION.into(), 1, &"a".repeat(64));
        let compose = native.clone().with_compose_images(vec![ComposeImage::new("atlas", "web".into(), REVISION)?]);
        assert!(artifact_matches_backend(&native, RuntimeBackend::Native).is_ok());
        assert!(artifact_matches_backend(&compose, RuntimeBackend::Docker).is_ok());
        assert!(artifact_matches_backend(&native, RuntimeBackend::Docker).is_err());
        assert!(artifact_matches_backend(&compose, RuntimeBackend::Native).is_err());
        Ok(())
    }
}
