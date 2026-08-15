use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Error, Result};
use bonesdeploy_core::config::build_user_for;
use bonesdeploy_core::paths;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::commands::{drop_failed_release, ensure_site_idle, release, service};
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;
use crate::release::lifecycle::build::ensure_build_user_ready;
use crate::release::lifecycle::preflight;
use crate::release::state as release_state;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;
    let mutation = SiteMutation::acquire(site)?;
    ensure_site_idle(site)?;

    let build_user = build_user_for(mutation.site());
    let project_root = PathBuf::from(paths::default_project_root_for(mutation.site()));
    ensure_build_user_ready(&build_user, &project_root)?;

    let target_revision = revision.map_or_else(|| mutation.config().branch.clone(), ToOwned::to_owned);
    let repo_path = PathBuf::from(paths::default_repo_path_for(mutation.site()));
    let revision_commit = lifecycle::checkout::resolve_revision_commit(&repo_path, &target_revision)?;
    let snapshot =
        lifecycle::DeploymentSnapshot::new(mutation.site(), mutation.config(), revision_commit, PathBuf::new());
    run_staged_deployment(&mutation, snapshot)
}

#[expect(clippy::too_many_lines)]
fn run_staged_deployment(mutation: &SiteMutation, snapshot: lifecycle::DeploymentSnapshot) -> Result<()> {
    let site = mutation.site();

    // Phase A: prepare and validate the new release while the old release still
    // serves. Any failure here aborts and returns the site to idle.
    stage("Staging release");
    if let Err(error) = lifecycle::stage::run(&snapshot) {
        return finish_abort(mutation, None, error);
    }

    let release_name = match release_state::read_staged_release(site) {
        Ok(release_name) => release_name,
        Err(error) => return finish_abort(mutation, None, error),
    };
    let mut deployment = release_state::DeploymentRecord::new(
        release_name.clone(),
        snapshot.revision.clone(),
        release_state::DeploymentPhase::Created,
        process::id(),
        process_start_ticks()?,
        deployment_started_at()?,
    );
    if let Err(error) = release_state::write_active_deployment(site, &deployment) {
        return finish_abort(mutation, None, error);
    }

    stage("Exporting source");
    let context_dir = match lifecycle::checkout::ensure_build_context(&snapshot) {
        Ok(context) => context,
        Err(error) => return finish_abort(mutation, None, error),
    };
    deployment.context = Some(context_dir.display().to_string());
    if let Err(error) = advance_phase(mutation, &deployment, None, Some(&context_dir)) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    let snapshot = snapshot.with_deployment_dir(context_dir.join(paths::LOCAL_INFRA_DIR).join(paths::DEPLOYMENT_DIR));
    if let Err(error) = lifecycle::checkout::run(&snapshot, &context_dir) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::SourceExported), Some(&context_dir))
    {
        return finish_abort(mutation, Some(&context_dir), error);
    }

    stage("Building release");
    if let Err(error) = lifecycle::build::run(&snapshot, &context_dir) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Built), Some(&context_dir))
    {
        return finish_abort(mutation, Some(&context_dir), error);
    }

    stage("Preparing release");
    if let Err(error) = lifecycle::build::promote(&snapshot, &context_dir) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Promoted), Some(&context_dir))
    {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) = lifecycle::wire_shared::run(&snapshot) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) = lifecycle::prepare::run(&snapshot) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Prepared), Some(&context_dir))
    {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) = lifecycle::build::finalize(&snapshot) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Sealed), Some(&context_dir))
    {
        return finish_abort(mutation, Some(&context_dir), error);
    }

    // Perfect-before-cut-over gate: last check while the old release still
    // serves. `nginx -t` runs before any reload; a failure aborts.
    stage("Verifying before cut-over");
    if let Err(error) = preflight::validate_ready(mutation, &release_name, || preflight::run_nginx_test(site)) {
        return finish_abort(mutation, Some(&context_dir), error);
    }

    let previous_release = match release_state::current_release_dir(&mutation.config().project_root) {
        Ok(release) => release,
        Err(error) => return finish_abort(mutation, Some(&context_dir), error),
    };
    deployment.previous_release = previous_release.file_name().map(|name| name.to_string_lossy().into_owned());

    // Phase B: cut-over — the commit point. Failure restores the previous
    // release (transactional rollback), leaving the site idle.
    stage("Activating release");
    if let Err(error) = lifecycle::activate::run(&snapshot) {
        return finish_abort(mutation, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Activated), Some(&context_dir))
    {
        return finish_abort(mutation, Some(&context_dir), error);
    }

    stage("Restarting services");
    if let Err(error) = service::run(mutation) {
        return finish_failed_activation(mutation, &previous_release, Some(&context_dir), error);
    }
    if let Err(error) =
        advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Verified), Some(&context_dir))
    {
        return finish_abort_without_release_drop(mutation, Some(&context_dir), error);
    }

    // Phase C: post-commit maintenance. The new release is serving; failures
    // here are recorded as `cleanup_pending` warnings, never deployment errors.
    if let Err(error) = run_maintenance(mutation, &context_dir) {
        finish_cleanup_pending(mutation, Some(&context_dir), &error);
        return Ok(());
    }
    if let Err(error) = advance_phase(mutation, &deployment, Some(release_state::DeploymentPhase::Completed), None) {
        finish_cleanup_pending(mutation, Some(&context_dir), &error);
        return Ok(());
    }
    release_state::clear_active_deployment(site)?;
    Ok(())
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
        record.phase = phase;
    }
    if let Some(context) = context {
        record.context = Some(context.display().to_string());
    }
    release_state::write_active_deployment(mutation.site(), &record)
}

/// Post-commit maintenance: staging pointer cleanup, old-release pruning, and
/// temporary build context removal. All operate on the committed release, so
/// their failures never affect serving traffic.
fn run_maintenance(mutation: &SiteMutation, context_dir: &Path) -> Result<()> {
    stage("Pruning old releases");
    release_state::clear_staged_release(mutation.site())?;
    release::prune::run_locked(mutation)?;
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
    if let Ok(active) = release_state::read_active_deployment(site) {
        if let Some(mut record) = active {
            record.phase = release_state::DeploymentPhase::CleanupPending;
            record.error = Some(error.to_string());
            let _ = release_state::write_active_deployment(site, &record);
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

fn finish_failed_activation(
    mutation: &SiteMutation,
    previous_release: &Path,
    context: Option<&Path>,
    error: Error,
) -> Result<()> {
    let project_root = &mutation.config().project_root;
    if let Err(restore_error) = restore_previous_release(Path::new(project_root), previous_release) {
        return finish_abort_without_release_drop(
            mutation,
            context,
            error.context(format!("Failed to restore previous release: {restore_error:#}")),
        );
    }

    let error = match service::run(mutation) {
        Ok(()) => error,
        Err(restart_error) => error.context(format!("Failed to restart the restored release: {restart_error:#}")),
    };
    finish_abort(mutation, context, error)
}

fn restore_previous_release(project_root: &Path, previous_release: &Path) -> Result<()> {
    let current_link = PathBuf::from(project_root).join(paths::CURRENT_LINK);
    release_state::point_symlink_atomically(&current_link, previous_release)
}

fn process_start_ticks() -> Result<u64> {
    let path = format!("/proc/{}/stat", process::id());
    let stat = fs::read_to_string(&path).with_context(|| format!("Failed to read {path}"))?;
    release::list::process_start_ticks(&stat).context("Failed to read deployment process start time")
}

fn deployment_started_at() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc().format(TIMESTAMP_FORMAT).context("Failed to format deployment start time")
}

fn cleanup(mutation: &SiteMutation, context: Option<&Path>) -> Result<()> {
    if let Some(context) = context {
        lifecycle::checkout::cleanup_build_context(mutation.site(), context)?;
    }
    Ok(())
}

fn abort(mutation: &SiteMutation, context: Option<&Path>, error: Error) -> Result<()> {
    let mut error = abort_context_only(mutation, context, error);
    if let Err(drop_error) = drop_failed_release::run_locked(mutation) {
        error = error.context(format!("Failed to remove failed release: {drop_error:#}"));
    }
    Err(error)
}

fn finish_abort(mutation: &SiteMutation, context: Option<&Path>, error: Error) -> Result<()> {
    let result = abort(mutation, context, error);
    clear_active_after_result(mutation.site(), result)
}

fn finish_abort_without_release_drop(mutation: &SiteMutation, context: Option<&Path>, error: Error) -> Result<()> {
    let error = abort_context_only(mutation, context, error);
    clear_active_after_result(mutation.site(), Err(error))
}

fn clear_active_after_result(site: &str, result: Result<()>) -> Result<()> {
    if let Err(clear_error) = release_state::clear_active_deployment(site) {
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::Path;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use bonesdeploy_core::paths;

    use super::restore_previous_release;

    #[test]
    fn failed_activation_restores_previous_release() -> Result<()> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("bonesremote_restore_{}_{}", process::id(), nonce));
        let releases = root.join(paths::RELEASES_DIR);
        let previous = releases.join("previous");
        let failed = releases.join("failed");
        fs::create_dir_all(&previous)?;
        fs::create_dir(&failed)?;
        symlink(&failed, root.join(paths::CURRENT_LINK))?;

        restore_previous_release(Path::new(&root), &previous)?;

        assert_eq!(fs::read_link(root.join(paths::CURRENT_LINK))?, previous);
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
