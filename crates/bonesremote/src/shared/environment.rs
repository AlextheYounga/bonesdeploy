use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt, chown};
use std::path::{Path, PathBuf};
use std::process;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config;
use bonesdeploy_core::paths;

use crate::release::SiteMutation;
use crate::release::lifecycle::build::ownership;

const MAX_ENVIRONMENT_BYTES: u64 = 1024 * 1024;
pub(crate) const ENVIRONMENT_MODE: u32 = 0o640;

pub fn install(mutation: &SiteMutation, reader: &mut dyn Read) -> Result<()> {
    let mut bytes = Vec::new();
    reader.take(MAX_ENVIRONMENT_BYTES + 1).read_to_end(&mut bytes).context("Failed to read shared environment")?;
    if bytes.len() as u64 > MAX_ENVIRONMENT_BYTES {
        bail!("Shared environment exceeds {MAX_ENVIRONMENT_BYTES} bytes");
    }
    let contents = str::from_utf8(&bytes).context("Shared environment is not valid UTF-8")?;
    config::validate_dotenv(contents)?;

    let shared = mutation.shared_dir();
    ensure_real_directory(&shared)?;
    let group = config::runtime_group_for(mutation.site());
    let gid = ownership::site_group_gid(&group)?;
    let target = shared.join(paths::DOT_ENV);
    let temporary = temporary_path(&target)?;

    let result = write_environment(&temporary, &target, &bytes, gid);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_environment(temporary: &Path, target: &Path, contents: &[u8], gid: u32) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .with_context(|| format!("Failed to create temporary environment {}", temporary.display()))?;
    file.write_all(contents).context("Failed to write shared environment")?;
    file.flush().context("Failed to flush shared environment")?;
    fs::set_permissions(temporary, fs::Permissions::from_mode(ENVIRONMENT_MODE))?;
    chown_regular_file(temporary, 0, gid)?;
    file.sync_all().context("Failed to sync shared environment")?;
    drop(file);

    fs::rename(temporary, target)
        .with_context(|| format!("Failed to atomically replace shared environment {}", target.display()))?;
    sync_parent(target)
}

pub(crate) fn validate_live_environment(path: &Path, expected_gid: u32) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path).with_context(|| {
        format!("Shared environment is missing: {}. Run 'bonesdeploy secrets push'.", path.display())
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "Shared environment must be a regular non-symlink file: {}. Run 'bonesdeploy secrets push' to repair it.",
            path.display()
        );
    }
    if metadata.uid() != 0 || metadata.gid() != expected_gid || metadata.mode() & 0o7777 != ENVIRONMENT_MODE {
        bail!(
            "Shared environment has unsafe ownership or permissions: {}. Run 'bonesdeploy secrets push' to repair it.",
            path.display()
        );
    }
    Ok(metadata)
}

pub(crate) fn copy_live_environment(source: &Path, destination: &Path, gid: u32) -> Result<()> {
    validate_live_environment(source, gid)?;
    let mut input = File::open(source).with_context(|| format!("Failed to open {}", source.display()))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .with_context(|| format!("Failed to create {}", destination.display()))?;
    io::copy(&mut input, &mut output).context("Failed to preserve shared environment")?;
    fs::set_permissions(destination, fs::Permissions::from_mode(ENVIRONMENT_MODE))?;
    chown_regular_file(destination, 0, gid)?;
    output.sync_all().context("Failed to sync preserved shared environment")
}

fn chown_regular_file(path: &Path, uid: u32, gid: u32) -> Result<()> {
    chown(path, Some(uid), Some(gid)).with_context(|| format!("Failed to set ownership on {}", path.display()))
}

fn ensure_real_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("Failed to inspect {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("Shared path is not a real directory: {}", path.display());
    }
    Ok(())
}

fn temporary_path(target: &Path) -> Result<PathBuf> {
    let parent = target.parent().context("Shared environment has no parent directory")?;
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    Ok(parent.join(format!(".env.tmp-{}-{nonce}", process::id())))
}

fn sync_parent(path: &Path) -> Result<()> {
    let parent = path.parent().context("Shared environment has no parent directory")?;
    File::open(parent)?.sync_all().with_context(|| format!("Failed to sync {}", parent.display()))
}
