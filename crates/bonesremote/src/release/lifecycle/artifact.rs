//! Bounded, defensive materialization of a local-build artifact.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::artifact::{ArtifactManifest, read_manifest};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

pub const MAX_COMPRESSED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_ARTIFACT_PATH_BYTES: usize = 4 * 1024;
pub const MAX_ARTIFACT_FILES: usize = 100_000;
pub const MAX_ARTIFACT_FILE_BYTES: u64 = MAX_COMPRESSED_BYTES;
pub const MAX_ARTIFACT_EXPANDED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Receives an artifact while verifying all payload bytes, including gzip data
/// after the tar end marker. Kept separate so the extraction reader can be
/// consumed before its digest state is inspected.
pub fn receive(
    reader: &mut dyn Read,
    expected_site: &str,
    expected_revision: &str,
    context: &Path,
) -> Result<ArtifactManifest> {
    let mut input = reader;
    let manifest = read_manifest(&mut input)?;
    manifest.validate()?;
    if manifest.site != expected_site {
        bail!("artifact site does not match deploy site");
    }
    if manifest.revision != expected_revision {
        bail!("artifact revision does not match the configured branch commit");
    }
    if manifest.artifact_length > MAX_COMPRESSED_BYTES {
        bail!("artifact exceeds {MAX_COMPRESSED_BYTES} compressed bytes");
    }

    let compressed = DigestReader::new(&mut input, manifest.artifact_length);
    let mut gzip = GzDecoder::new(compressed);
    extract(&mut gzip, context).context("failed to materialize artifact")?;
    io::copy(&mut gzip, &mut io::sink()).context("failed to finish artifact gzip stream")?;
    let mut compressed = gzip.into_inner();
    io::copy(&mut compressed, &mut io::sink()).context("artifact payload is truncated")?;
    if compressed.remaining != 0 {
        bail!("artifact payload is truncated");
    }
    if format!("{:x}", compressed.digest.finalize()) != manifest.sha256 {
        bail!("artifact digest does not match manifest");
    }
    let mut trailing = [0_u8; 1];
    if input.read(&mut trailing)? != 0 {
        bail!("artifact stream has trailing bytes");
    }
    Ok(manifest)
}

/// Receives the payload after the caller has consumed and validated its manifest.
pub fn receive_payload(
    reader: &mut dyn Read,
    manifest: &ArtifactManifest,
    expected_site: &str,
    context: &Path,
) -> Result<()> {
    if manifest.site != expected_site {
        bail!("artifact site does not match deploy site");
    }
    receive_payload_inner(reader, manifest, context)
}

fn receive_payload_inner(reader: &mut dyn Read, manifest: &ArtifactManifest, context: &Path) -> Result<()> {
    manifest.validate()?;
    if manifest.artifact_length > MAX_COMPRESSED_BYTES {
        bail!("artifact exceeds {MAX_COMPRESSED_BYTES} compressed bytes");
    }
    let compressed = DigestReader::new(reader, manifest.artifact_length);
    let mut gzip = GzDecoder::new(compressed);
    extract(&mut gzip, context).context("failed to materialize artifact")?;
    io::copy(&mut gzip, &mut io::sink()).context("failed to finish artifact gzip stream")?;
    let mut compressed = gzip.into_inner();
    io::copy(&mut compressed, &mut io::sink()).context("artifact payload is truncated")?;
    if compressed.remaining != 0 {
        bail!("artifact payload is truncated");
    }
    if format!("{:x}", compressed.digest.finalize()) != manifest.sha256 {
        bail!("artifact digest does not match manifest");
    }
    let mut trailing = [0_u8; 1];
    if reader.read(&mut trailing)? != 0 {
        bail!("artifact stream has trailing bytes");
    }
    Ok(())
}

struct DigestReader<'a, R: ?Sized> {
    reader: &'a mut R,
    remaining: u64,
    digest: Sha256,
}
impl<'a, R: Read + ?Sized> DigestReader<'a, R> {
    fn new(reader: &'a mut R, remaining: u64) -> Self {
        Self { reader, remaining, digest: Sha256::new() }
    }
}
impl<R: Read + ?Sized> Read for DigestReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Ok(0);
        }
        let limit = usize::try_from(self.remaining).unwrap_or(usize::MAX).min(buffer.len());
        let read = self.reader.read(&mut buffer[..limit])?;
        self.remaining -= read as u64;
        self.digest.update(&buffer[..read]);
        Ok(read)
    }
}

fn extract<R: Read>(reader: R, context: &Path) -> Result<()> {
    let mut archive = Archive::new(reader);
    let mut entries = HashMap::new();
    let mut directory_modes = Vec::new();
    let mut links = Vec::new();
    let mut files = 0_usize;
    let mut expanded = 0_u64;
    for entry in archive.entries().context("invalid tar archive")? {
        let mut entry = entry.context("invalid tar entry")?;
        let path = checked_path(&entry.path().context("invalid tar path")?)?;
        let entry_type = entry.header().entry_type();
        let kind = if entry_type.is_dir() {
            "directory"
        } else if entry_type.is_file() {
            "file"
        } else if entry_type.is_symlink() {
            "symlink"
        } else {
            bail!("artifact contains unsupported tar entry type");
        };
        register_path(&mut entries, &path, kind)?;
        files += 1;
        if files > MAX_ARTIFACT_FILES {
            bail!("artifact has more than {MAX_ARTIFACT_FILES} entries");
        }
        let size = entry.size();
        if entry_type.is_file() {
            if size > MAX_ARTIFACT_FILE_BYTES {
                bail!("artifact file exceeds {MAX_ARTIFACT_FILE_BYTES} bytes");
            }
            expanded = expanded.checked_add(size).context("artifact expanded size overflow")?;
            if expanded > MAX_ARTIFACT_EXPANDED_BYTES {
                bail!("artifact exceeds {MAX_ARTIFACT_EXPANDED_BYTES} expanded bytes");
            }
            let target = context.join(&path);
            create_parent(&target)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .with_context(|| format!("failed to create artifact file {}", target.display()))?;
            let mode = entry.header().mode().unwrap_or(0o644) & 0o777;
            io::copy(&mut entry, &mut file).context("failed to write artifact file")?;
            fs::set_permissions(&target, fs::Permissions::from_mode(mode))?;
        } else if entry_type.is_dir() {
            let target = context.join(&path);
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create artifact directory {}", target.display()))?;
            directory_modes.push((target, entry.header().mode().unwrap_or(0o755) & 0o777));
        } else {
            let target = entry.link_name().context("invalid symlink target")?.context("symlink has no target")?;
            checked_link_target(&path, &target)?;
            links.push((path, target.into_owned()));
        }
    }
    // Apply directory modes after all ordinary entries, so restrictive parent
    // modes never prevent extraction and no symlink can affect these paths.
    for (directory, mode) in directory_modes.into_iter().rev() {
        fs::set_permissions(directory, fs::Permissions::from_mode(mode))?;
    }
    for (path, target) in links {
        let destination = context.join(path);
        create_parent(&destination)?;
        symlink(target, destination).context("failed to create artifact symlink")?;
    }
    Ok(())
}

fn checked_path(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().len() > MAX_ARTIFACT_PATH_BYTES {
        bail!("artifact path exceeds {MAX_ARTIFACT_PATH_BYTES} bytes");
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => result.push(value),
            _ => bail!("artifact path is not a relative child path"),
        }
    }
    if result.as_os_str().is_empty() {
        bail!("artifact path is empty");
    }
    Ok(result)
}

fn register_path(entries: &mut HashMap<PathBuf, &'static str>, path: &Path, kind: &'static str) -> Result<()> {
    if entries.insert(path.to_path_buf(), kind).is_some() {
        bail!("artifact contains duplicate path {}", path.display());
    }
    for ancestor in path.ancestors().skip(1) {
        if let Some(kind) = entries.get(ancestor)
            && *kind != "directory"
        {
            bail!("artifact path conflicts with {kind} {}", ancestor.display());
        }
    }
    if kind != "directory" && entries.keys().any(|other| other.starts_with(path) && other != path) {
        bail!("artifact path conflicts with descendant {}", path.display());
    }
    Ok(())
}

fn checked_link_target(path: &Path, target: &Path) -> Result<()> {
    if target.as_os_str().len() > MAX_ARTIFACT_PATH_BYTES {
        bail!("artifact symlink target exceeds {MAX_ARTIFACT_PATH_BYTES} bytes");
    }
    let mut depth = path.parent().map_or(0, |parent| parent.components().count());
    for component in target.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::ParentDir if depth > 0 => depth -= 1,
            Component::ParentDir => bail!("artifact symlink escapes its context"),
            _ => bail!("artifact symlink target is unsafe"),
        }
    }
    Ok(())
}

fn create_parent(path: &Path) -> Result<()> {
    let parent = path.parent().context("artifact path has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create artifact parent {}", parent.display()))
}

#[cfg(test)]
#[path = "artifact_tests.rs"]
mod tests;
