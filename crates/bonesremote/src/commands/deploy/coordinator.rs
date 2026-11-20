use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Error, Result};
use bonesdeploy_core::artifact::{ArtifactKind, ArtifactManifest};
use bonesdeploy_core::paths;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::release::SiteMutation;
use crate::release::failed;
use crate::release::lifecycle;
use crate::release::lifecycle::activate::{ActivationFailure, ReleaseActivation};
use crate::release::lifecycle::preflight;
use crate::release::state as release_state;
use crate::release::{process as release_process, prune};
use crate::runtime::docker;
use crate::runtime::service;

struct PreparedDeployment {
    snapshot: lifecycle::DeploymentSnapshot,
    deployment: release_state::DeploymentRecord,
    context_dir: PathBuf,
    previous_release: PathBuf,
    manifest_image_tags: Vec<String>,
}

pub(crate) fn run_staged_deployment(
    mutation: &SiteMutation,
    snapshot: lifecycle::DeploymentSnapshot,
    reader: &mut dyn Read,
    manifest: &ArtifactManifest,
) -> Result<()> {
    let prepared = prepare_deployment(mutation, snapshot, reader, manifest)?;
    activate_deployment(mutation, &prepared)?;
    complete_deployment(mutation, &prepared);
    Ok(())
}

fn prepare_deployment(
    mutation: &SiteMutation,
    snapshot: lifecycle::DeploymentSnapshot,
    reader: &mut dyn Read,
    manifest: &ArtifactManifest,
) -> Result<PreparedDeployment> {
    let (mut deployment, snapshot, context_dir) = start_release(mutation, snapshot)?;
    let manifest_image_tags = build_and_prepare(mutation, &snapshot, &context_dir, &mut deployment, reader, manifest)?;

    let site = mutation.site();
    stage("Verifying before cut-over");
    if let Err(error) = preflight::validate_ready(mutation, deployment.release(), || preflight::run_nginx_test(site)) {
        return finish_abort(mutation, Some(&context_dir), &manifest_image_tags, error);
    }

    let previous_release = match release_state::current_release_dir(&mutation.config().project_root) {
        Ok(release) => release,
        Err(error) => return finish_abort(mutation, Some(&context_dir), &manifest_image_tags, error),
    };
    deployment.set_previous_release(previous_release.file_name().map(|name| name.to_string_lossy().into_owned()));

    Ok(PreparedDeployment { snapshot, deployment, context_dir, previous_release, manifest_image_tags })
}

fn start_release(
    mutation: &SiteMutation,
    snapshot: lifecycle::DeploymentSnapshot,
) -> Result<(release_state::DeploymentRecord, lifecycle::DeploymentSnapshot, PathBuf)> {
    let identity = release_state::ProcessIdentity::new(process::id(), process_start_ticks()?, deployment_started_at()?);
    stage("Staging release");
    if let Err(error) = lifecycle::stage::run(mutation, &snapshot) {
        return finish_abort(mutation, None, &[], error);
    }

    let release_name = match mutation.required_staged_release() {
        Ok(release_name) => release_name,
        Err(error) => return finish_abort(mutation, None, &[], error),
    };
    let mut deployment = release_state::DeploymentRecord::new(
        release_name,
        snapshot.revision.clone(),
        release_state::DeploymentPhase::Created,
        identity,
    );
    if let Err(error) = mutation.set_active(&deployment) {
        return finish_abort(mutation, None, &[], error);
    }

    stage("Creating candidate");
    let context_dir = match lifecycle::context::create(&snapshot) {
        Ok(context) => context,
        Err(error) => return finish_abort(mutation, None, &[], error),
    };
    deployment.set_context(context_dir.display().to_string());
    if let Err(error) = advance_phase(mutation, &deployment, None, Some(&context_dir)) {
        return finish_abort(mutation, Some(&context_dir), &[], error);
    }
    let snapshot = snapshot.with_deployment_dir(context_dir.join(paths::LOCAL_INFRA_DIR).join(paths::DEPLOYMENT_DIR));
    Ok((deployment, snapshot, context_dir))
}

#[expect(
    clippy::too_many_arguments,
    reason = "candidate context, persisted record and framed artifact meet at the preparation boundary"
)]
fn build_and_prepare(
    mutation: &SiteMutation,
    snapshot: &lifecycle::DeploymentSnapshot,
    context_dir: &Path,
    deployment: &mut release_state::DeploymentRecord,
    reader: &mut dyn Read,
    manifest: &ArtifactManifest,
) -> Result<Vec<String>> {
    let mut manifest_image_tags = Vec::new();
    if let Err(error) = lifecycle::artifact::receive_payload(reader, manifest, &snapshot.site, context_dir) {
        return finish_abort(mutation, Some(context_dir), &[], error);
    }
    if let ArtifactKind::ComposeImages { images } = &manifest.kind {
        manifest_image_tags = images.iter().map(|image| image.tag.clone()).collect();
    }
    if let Err(error) =
        advance_phase(mutation, deployment, Some(release_state::DeploymentPhase::Received), Some(context_dir))
    {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }

    if let ArtifactKind::ComposeImages { images } = &manifest.kind {
        stage("Loading Compose images");
        if let Err(error) = docker::command::validate_artifact_images(context_dir, images) {
            return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
        }
        if let Err(error) = docker::command::load_artifact_images(context_dir, images) {
            return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
        }
    }

    if let Err(error) =
        advance_phase(mutation, deployment, Some(release_state::DeploymentPhase::Materialized), Some(context_dir))
    {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }

    stage("Preparing release");
    if let Err(error) = lifecycle::build::promote(mutation, snapshot, context_dir) {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let ArtifactKind::ComposeImages { images } = &manifest.kind
        && let Err(error) =
            docker::command::validate_release_override(mutation.release_dir(deployment.release()).as_path(), images)
    {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let Err(error) =
        advance_phase(mutation, deployment, Some(release_state::DeploymentPhase::Promoted), Some(context_dir))
    {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let Err(error) = lifecycle::wire_shared::run(mutation, snapshot) {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let Err(error) = lifecycle::prepare::run(mutation, snapshot) {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let Err(error) =
        advance_phase(mutation, deployment, Some(release_state::DeploymentPhase::Prepared), Some(context_dir))
    {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let Err(error) = lifecycle::build::finalize(mutation, snapshot) {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    if let Err(error) =
        advance_phase(mutation, deployment, Some(release_state::DeploymentPhase::Sealed), Some(context_dir))
    {
        return finish_abort(mutation, Some(context_dir), &manifest_image_tags, error);
    }
    Ok(manifest_image_tags)
}

fn activate_deployment(mutation: &SiteMutation, prepared: &PreparedDeployment) -> Result<()> {
    // Only durable service verification commits the new release.
    stage("Activating release");
    let candidate = mutation.release_dir(prepared.deployment.release());
    let result = ReleaseActivation::new(&prepared.snapshot.project_root, &candidate, &prepared.previous_release).run(
        |phase| advance_phase(mutation, &prepared.deployment, Some(phase), None),
        || service::run_for_release(mutation),
    );
    match result {
        Ok(()) => Ok(()),
        Err(ActivationFailure::Aborted(error)) => {
            finish_abort(mutation, Some(&prepared.context_dir), &prepared.manifest_image_tags, error)
        }
        Err(ActivationFailure::RecoveryRequired(error)) => {
            let mut record = prepared.deployment.clone();
            record.set_phase(release_state::DeploymentPhase::Failed);
            record.set_error(format!("{error:#}"));
            let error = match mutation.set_active(&record) {
                Ok(()) => error,
                Err(state_error) => error.context(format!("Failed to retain recovery state: {state_error:#}")),
            };
            Err(error.context("Release recovery is incomplete; candidate and deployment state were retained"))
        }
    }
}

fn complete_deployment(mutation: &SiteMutation, prepared: &PreparedDeployment) {
    // Phase C: post-commit maintenance. The new release is serving; failures
    // here are recorded as `cleanup_pending` warnings, never deployment errors.
    if let Err(error) = run_maintenance(mutation, &prepared.context_dir) {
        finish_cleanup_pending(mutation, Some(&prepared.context_dir), &error);
        return;
    }
    if let Err(error) =
        advance_phase(mutation, &prepared.deployment, Some(release_state::DeploymentPhase::Completed), None)
    {
        finish_cleanup_pending(mutation, Some(&prepared.context_dir), &error);
        return;
    }
    if let Err(error) = mutation.clear_active() {
        finish_cleanup_pending(mutation, Some(&prepared.context_dir), &error);
    }
}

/// Advances the persisted deployment phase and writes the record (with the
/// build context attached in Phase A). `phase` is `None` only for the initial
/// `created` write that records the context before the first transition.
fn advance_phase(
    mutation: &SiteMutation,
    deployment: &release_state::DeploymentRecord,
    phase: Option<release_state::DeploymentPhase>,
    context: Option<&Path>,
) -> Result<()> {
    let mut record = deployment.clone();
    if let Some(phase) = phase {
        record.set_phase(phase);
    }
    if let Some(context) = context {
        record.set_context(context.display().to_string());
    }
    mutation.set_active(&record)
}

/// Post-commit maintenance: staging pointer cleanup, old-release pruning, and
/// temporary build context removal. All operate on the committed release, so
/// their failures never affect serving traffic.
fn run_maintenance(mutation: &SiteMutation, context_dir: &Path) -> Result<()> {
    stage("Pruning old releases");
    mutation.clear_staged_release()?;
    prune::run_locked(mutation, mutation.config().releases_keep)?;
    stage("Cleaning up");
    cleanup(mutation, Some(context_dir))
}

/// Records a post-commit maintenance failure as observable `cleanup_pending`
/// state: the new release is already serving, so the deployment itself did not
/// fail. The residual record keeps the site serialization-idle (a next
/// deployment may proceed and finish cleanup).
fn finish_cleanup_pending(mutation: &SiteMutation, context: Option<&Path>, error: &Error) {
    let site = mutation.site();
    let _ = cleanup(mutation, context);
    if let Ok(active) = mutation.active() {
        if let Some(mut record) = active {
            record.set_phase(release_state::DeploymentPhase::CleanupPending);
            record.set_error(error.to_string());
            let _ = mutation.set_active(&record);
        }
    }
    eprintln!(
        "Warning: the new release is active, but post-deploy maintenance was incomplete:\n  {error:#}\n  Run 'bonesremote release list --site {site}' to inspect. The residual state does not block future deployments and is cleared by the next successful deploy."
    );
}

fn stage(name: &str) {
    println!("{} {}", ansi("1;36", "->"), ansi("2", &format!("{name}...")));
}

fn ansi(code: &str, value: &str) -> String {
    format!("\x1b[{code}m{value}\x1b[0m")
}

fn process_start_ticks() -> Result<u64> {
    release_process::current_start_ticks().context("Failed to read deployment process start time")
}

fn deployment_started_at() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc().format(TIMESTAMP_FORMAT).context("Failed to format deployment start time")
}

fn cleanup(mutation: &SiteMutation, context: Option<&Path>) -> Result<()> {
    if let Some(context) = context {
        lifecycle::context::cleanup(mutation.site(), context)?;
    }
    Ok(())
}

fn abort(mutation: &SiteMutation, context: Option<&Path>, tags: &[String], error: Error) -> Result<()> {
    let mut error = abort_context_only(mutation, context, error);
    if !tags.is_empty() {
        let excluded_release = mutation.staged_release().ok().flatten();
        if let Err(cleanup_error) =
            failed::remove_unreferenced_manifest_images(mutation, tags, excluded_release.as_deref())
        {
            error = error.context(format!("Failed to remove loaded Compose images: {cleanup_error:#}"));
        }
    }
    if let Err(drop_error) = failed::run_locked(mutation) {
        error = error.context(format!("Failed to remove failed release: {drop_error:#}"));
    }
    Err(error)
}

fn finish_abort<T>(mutation: &SiteMutation, context: Option<&Path>, tags: &[String], error: Error) -> Result<T> {
    let result = abort(mutation, context, tags, error);
    match clear_active_after_result(mutation, result) {
        Ok(()) => Err(anyhow::anyhow!("Deployment abort unexpectedly succeeded")),
        Err(error) => Err(error),
    }
}

fn clear_active_after_result(mutation: &SiteMutation, result: Result<()>) -> Result<()> {
    if let Err(clear_error) = mutation.clear_active() {
        return result
            .map_err(|error| error.context(format!("Failed to clear active deployment state: {clear_error:#}")));
    }
    result
}

fn abort_context_only(mutation: &SiteMutation, context: Option<&Path>, mut error: Error) -> Error {
    if let Err(cleanup_error) = cleanup(mutation, context) {
        error = error.context(format!("Cleanup failed: {cleanup_error:#}"));
    }
    error
}
