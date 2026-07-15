use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Error, Result, bail};
use shared::config;
use shared::config::build_user_for;
use shared::paths;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::commands::{drop_failed_release, release_list, release_prune, service};
use crate::privileges;
use crate::release::lifecycle;
use crate::release::script_runner::ensure_build_user_ready;
use crate::release::state as release_state;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;
    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let _lock = release_state::DeploymentLock::acquire(site)?;
    if let Some(active) = release_state::read_active_deployment(site)? {
        bail!(
            "Release {} was interrupted. Run 'bonesdeploy releases' and cancel it before starting another deployment.",
            active.release
        );
    }
    if let Ok(staged) = release_state::read_staged_release(site) {
        bail!(
            "Release {staged} is staged without an active deployment. Run 'bonesdeploy releases' before starting another deployment."
        );
    }
    let build_user = build_user_for(&cfg.project_name);
    ensure_build_user_ready(&build_user, Path::new(&cfg.project_root))?;

    let target_revision = revision.map_or_else(|| cfg.branch.clone(), ToOwned::to_owned);
    run_staged_deployment(site, &target_revision)
}

fn run_staged_deployment(site: &str, target_revision: &str) -> Result<()> {
    if let Err(error) = lifecycle::stage::run(site) {
        return finish_abort(site, None, error);
    }

    let release = match release_state::read_staged_release(site) {
        Ok(release) => release,
        Err(error) => return finish_abort(site, None, error),
    };
    let mut deployment = release_state::ActiveDeployment {
        release,
        pid: process::id(),
        process_start_ticks: process_start_ticks()?,
        phase: release_state::DeploymentPhase::Building,
        started_at: deployment_started_at()?,
        context: None,
    };
    if let Err(error) = release_state::write_active_deployment(site, &deployment) {
        return finish_abort(site, None, error);
    }

    let context_dir = match lifecycle::checkout::ensure_build_context(site) {
        Ok(context) => context,
        Err(error) => return finish_abort(site, None, error),
    };
    deployment.context = Some(context_dir.display().to_string());
    if let Err(error) = release_state::write_active_deployment(site, &deployment) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = lifecycle::checkout::run(site, &target_revision, &context_dir) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = lifecycle::build::run(site, &context_dir) {
        return finish_abort(site, Some(&context_dir), error);
    }

    deployment.phase = release_state::DeploymentPhase::Preparing;
    if let Err(error) = release_state::write_active_deployment(site, &deployment) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = lifecycle::build::promote(site, &context_dir) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = lifecycle::wire_shared::run(site) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = lifecycle::prepare::run(site) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = lifecycle::activate::run(site) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = service::run(site) {
        return finish_abort(site, Some(&context_dir), error);
    }

    if let Err(error) = release_prune::run(site) {
        return finish_abort_without_release_drop(site, Some(&context_dir), error);
    }

    if let Err(error) = cleanup(site, Some(&context_dir)) {
        return finish_abort_without_release_drop(site, Some(&context_dir), error);
    }
    release_state::clear_active_deployment(site)?;
    Ok(())
}

fn process_start_ticks() -> Result<u64> {
    let path = format!("/proc/{}/stat", process::id());
    let stat = fs::read_to_string(&path).with_context(|| format!("Failed to read {path}"))?;
    release_list::process_start_ticks(&stat).context("Failed to read deployment process start time")
}

fn deployment_started_at() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc().format(TIMESTAMP_FORMAT).context("Failed to format deployment start time")
}

fn cleanup(site: &str, context: Option<&Path>) -> Result<()> {
    if let Some(context) = context {
        lifecycle::checkout::cleanup_build_context(site, context)?;
    }
    Ok(())
}

fn abort(site: &str, context: Option<&Path>, error: Error) -> Result<()> {
    let error = match abort_without_release_drop(site, context, error) {
        Ok(()) => unreachable!("abort_without_release_drop always returns an error"),
        Err(error) => error,
    };
    let mut error = error;
    if let Err(drop_error) = drop_failed_release::run(site) {
        error = error.context(format!("Failed to remove failed release: {drop_error:#}"));
    }
    Err(error)
}

fn finish_abort(site: &str, context: Option<&Path>, error: Error) -> Result<()> {
    let result = abort(site, context, error);
    clear_active_after_result(site, result)
}

fn finish_abort_without_release_drop(site: &str, context: Option<&Path>, error: Error) -> Result<()> {
    let result = abort_without_release_drop(site, context, error);
    clear_active_after_result(site, result)
}

fn clear_active_after_result(site: &str, result: Result<()>) -> Result<()> {
    if let Err(clear_error) = release_state::clear_active_deployment(site) {
        return result
            .map_err(|error| error.context(format!("Failed to clear active deployment state: {clear_error:#}")));
    }
    result
}

fn abort_without_release_drop(site: &str, context: Option<&Path>, error: Error) -> Result<()> {
    let mut error = error;
    if let Err(cleanup_error) = cleanup(site, context) {
        error = error.context(format!("Cleanup failed: {cleanup_error:#}"));
    }
    Err(error)
}

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path).context("Failed to load remote site state")?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let releases = release_state::list_releases_sorted(&cfg.project_root)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg.project_root)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    let previous_dir = release_state::release_dir(&cfg.project_root, &previous_name);
    let current_link = PathBuf::from(&cfg.project_root).join(paths::CURRENT_LINK);
    release_state::point_symlink_atomically(&current_link, &previous_dir)?;
    service::run(site)?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}
