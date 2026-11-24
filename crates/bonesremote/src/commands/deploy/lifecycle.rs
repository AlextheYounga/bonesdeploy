use anyhow::Result;
use bonesdeploy_core::artifact::{ArtifactKind, ArtifactManifest, read_manifest};
use bonesdeploy_core::config::RuntimeBackend;
use std::io;

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;

use super::coordinator;

pub fn run_artifact(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote artifact deploy")?;
    let mutation = SiteMutation::acquire_for_deploy(site)?;
    mutation.ensure_idle()?;
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
