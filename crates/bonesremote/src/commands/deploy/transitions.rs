use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config;
use bonesdeploy_core::paths;

use crate::commands::service;
use crate::control_plane;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;
use crate::release::lifecycle::build::ensure_build_user_ready;
use crate::release::lifecycle::preflight;
use crate::release::state as release_state;

pub fn begin(site: &str, revision: &str) -> Result<()> {
    privileges::ensure_root("bonesremote deploy begin")?;
    config::validate_site_name(site)?;
    validate_revision(revision)?;
    let bones = control_plane::load(site)?.into_site_config(site);
    let mutation = SiteMutation::for_transition(site, bones)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, revision.to_owned(), Default::default());
    let release = lifecycle::stage::create_candidate(&snapshot)?;
    println!("{release}");
    Ok(())
}

pub fn prepare(site: &str, release: &str) -> Result<()> {
    let (mutation, snapshot, context) = transition_context(site, release)?;
    privileges::ensure_root("bonesremote deploy prepare")?;

    ensure_build_user_ready(&config::build_user_for(site), &snapshot.project_root)?;
    lifecycle::build::run(&mutation, &snapshot, &context)?;
    lifecycle::build::promote(&mutation, &snapshot, &context)?;
    lifecycle::wire_shared::run(&mutation, &snapshot)?;
    lifecycle::prepare::run(&mutation, &snapshot)?;
    lifecycle::build::finalize(&mutation, &snapshot)?;
    preflight::validate_ready(&mutation, release, || preflight::run_nginx_test(site))?;
    Ok(())
}

pub fn commit(site: &str, release: &str) -> Result<()> {
    let (mutation, snapshot, _context) = transition_context(site, release)?;
    privileges::ensure_root("bonesremote deploy commit")?;
    preflight::validate_ready(&mutation, release, || preflight::run_nginx_test(site))?;

    let previous = release_state::current_release_dir(&mutation.config().project_root)?;
    lifecycle::activate::run(&mutation, &snapshot)?;
    if let Err(error) = service::run_for_release(&mutation) {
        let current_link = Path::new(&mutation.config().project_root).join(paths::CURRENT_LINK);
        release_state::point_symlink_atomically(&current_link, &previous)
            .context("Failed to restore the previous release after service restart failure")?;
        service::run_for_release(&mutation).context("Failed to restart the restored release")?;
        return Err(error);
    }
    Ok(())
}

pub fn complete(site: &str, release: &str) -> Result<()> {
    let (mutation, _snapshot, context) = transition_context(site, release)?;
    privileges::ensure_root("bonesremote deploy complete")?;
    crate::commands::release::prune::run_locked(&mutation, mutation.config().releases_keep)?;
    if context.exists() {
        fs::remove_dir_all(&context)
            .with_context(|| format!("Failed to remove build context {}", context.display()))?;
    }
    Ok(())
}

pub fn abort(site: &str, release: &str) -> Result<()> {
    let (mutation, _snapshot, context) = transition_context(site, release)?;
    privileges::ensure_root("bonesremote deploy abort")?;
    crate::commands::drop_failed_release::ensure_release_not_active(&mutation, release)?;
    let release_dir = mutation.release_dir(release);
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
    }
    if context.exists() {
        fs::remove_dir_all(&context)
            .with_context(|| format!("Failed to remove build context {}", context.display()))?;
    }
    Ok(())
}

fn transition_context(
    site: &str,
    release: &str,
) -> Result<(SiteMutation, lifecycle::DeploymentSnapshot, std::path::PathBuf)> {
    config::validate_site_name(site)?;
    validate_release(release)?;
    let bones = control_plane::load(site)?.into_site_config(site);
    let mutation = SiteMutation::for_transition(site, bones)?;
    if mutation.required_staged_release()? != release {
        bail!("Release {release} is not the currently staged release for {site}");
    }
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, "".to_owned(), Default::default());
    let context =
        Path::new(&mutation.config().project_root).join(paths::TMP_BUILDS_DIR).join(format!("build-{release}"));
    let snapshot = snapshot.with_deployment_dir(context.join(paths::LOCAL_INFRA_DIR).join(paths::DEPLOYMENT_DIR));
    Ok((mutation, snapshot, context))
}

fn validate_revision(revision: &str) -> Result<()> {
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("Deployment revision must be a full hexadecimal Git commit id");
    }
    Ok(())
}

pub fn validate_release(release: &str) -> Result<()> {
    let bytes = release.as_bytes();
    if bytes.len() != 29
        || bytes[8] != b'_'
        || bytes[15] != b'-'
        || bytes[24] != b'-'
        || !bytes[..8].iter().chain(&bytes[9..15]).all(u8::is_ascii_digit)
        || !bytes[16..24].iter().chain(&bytes[25..]).all(u8::is_ascii_hexdigit)
    {
        bail!("Deployment release has an invalid generated release identifier");
    }
    Ok(())
}
