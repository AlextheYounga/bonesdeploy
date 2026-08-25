use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;

const MAX_RELEASE_NAME_ATTEMPTS: u32 = 10;

static RANDOM_FALLBACK_SEQUENCE: AtomicU32 = AtomicU32::new(0);

pub fn run(mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release stage")?;

    let project_root = &snapshot.project_root;
    require_dir(project_root, "project_root directory")?;
    require_dir(&snapshot.project_root.join(paths::RELEASES_DIR), "releases")?;
    require_dir(&snapshot.project_root.join(paths::SHARED_DIR), "shared")?;

    let release_name = create_unique_release_dir(&snapshot.project_root.to_string_lossy(), &snapshot.revision)?;

    mutation.set_staged_release(&release_name)?;

    println!("Staged release: {release_name}");
    Ok(())
}

fn require_dir(path: &Path, label: &str) -> Result<()> {
    if !path.is_dir() {
        bail!("Site not provisioned: {} does not exist ({label}). Run 'bonesdeploy site setup' first.", path.display());
    }
    Ok(())
}

/// Creates a release directory with an exclusive `create_dir`. Name collisions
/// (two deployments staged within the same second) are retried with a fresh
/// name, so no deployment can ever reuse or overwrite an existing release
/// directory, including the active one.
pub fn create_unique_release_dir(project_root: &str, revision_commit: &str) -> Result<String> {
    let mut next_name = || create_release_name(revision_commit, &random_suffix());
    create_unique_release_dir_with(project_root, &mut next_name)
}

pub fn create_unique_release_dir_with(
    project_root: &str,
    next_name: &mut dyn FnMut() -> Result<String>,
) -> Result<String> {
    for _ in 0..MAX_RELEASE_NAME_ATTEMPTS {
        let name = next_name()?;
        let dir = release_state::release_dir(project_root, &name);
        match fs::create_dir(&dir) {
            Ok(()) => return Ok(name),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error).with_context(|| format!("Failed to create release directory {}", dir.display()));
            }
        }
    }
    bail!("Failed to create a unique release directory after {MAX_RELEASE_NAME_ATTEMPTS} attempts")
}

/// Builds a release identity like `20260804_190321-46a0b75c-a7f2`: a
/// one-second timestamp, the first eight characters of the resolved source
/// commit, and a random suffix.
pub fn create_release_name(revision_commit: &str, suffix: &str) -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc();
    let timestamp = now.format(TIMESTAMP_FORMAT).context("Failed to format release timestamp")?;

    let short_commit = &revision_commit[..8.min(revision_commit.len())];
    Ok(format!("{timestamp}-{short_commit}-{suffix}"))
}

/// Returns a compact random suffix (4 lowercase hex chars) read from
/// `/dev/urandom`. Falls back to a pid-derived sequence if the device is
/// unavailable; correctness does not depend on it because release directory
/// creation is exclusive and retried.
fn random_suffix() -> String {
    let mut bytes = [0u8; 2];
    let mut urandom = fs::File::open("/dev/urandom");
    let read_ok = urandom.as_mut().is_ok_and(|file| file.read_exact(&mut bytes).is_ok());

    if !read_ok {
        let seq = RANDOM_FALLBACK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let value = (process::id() ^ seq).to_le_bytes();
        bytes = [value[0], value[1]];
    }

    format!("{:02x}{:02x}", bytes[0], bytes[1])
}
