use std::io::Read;

use anyhow::{Result, bail};
use bonesdeploy_core::artifact::{ArtifactKind, ArtifactManifest, read_manifest};
use bonesdeploy_core::config::RuntimeBackend;

use crate::release::SiteMutation;

/// Locks the site and validates the incoming artifact's identity and runtime.
/// The returned guard must be retained for the deployment. Callers must
/// authorize the deployment before receiving the request.
pub fn receive<R: Read>(site: &str, input: &mut R) -> Result<(SiteMutation, ArtifactManifest)> {
    let mutation = SiteMutation::acquire_for_deploy(site)?;
    mutation.ensure_idle()?;
    let manifest = read_manifest(input)?;
    if manifest.site != site {
        bail!("artifact site does not match deploy site");
    }
    if !matches!(
        (&manifest.kind, mutation.config().runtime.backend),
        (ArtifactKind::NativeTree, RuntimeBackend::Native)
            | (ArtifactKind::ComposeImages { .. }, RuntimeBackend::Docker)
    ) {
        bail!("artifact kind does not match the configured runtime backend");
    }
    Ok((mutation, manifest))
}
