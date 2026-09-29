//! Bounded, defensive materialization of a local-build artifact.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::artifact::read_manifest;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

pub const MAX_COMPRESSED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_ARTIFACT_PATH_BYTES: usize = 4 * 1024;
pub const MAX_ARTIFACT_FILES: usize = 100_000;
pub const MAX_ARTIFACT_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_ARTIFACT_EXPANDED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Receives an artifact while verifying all payload bytes, including gzip data
/// after the tar end marker. Kept separate so the extraction reader can be
/// consumed before its digest state is inspected.
pub fn receive(reader: &mut dyn Read, expected_site: &str, expected_revision: &str, context: &Path) -> Result<()> {
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
    Ok(())
}

struct DigestReader<'a, R> {
    reader: &'a mut R,
    remaining: u64,
    digest: Sha256,
}
impl<'a, R: Read> DigestReader<'a, R> {
    fn new(reader: &'a mut R, remaining: u64) -> Self {
        Self { reader, remaining, digest: Sha256::new() }
    }
}
impl<R: Read> Read for DigestReader<'_, R> {
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
mod tests {
    use std::io::Cursor;

    use bonesdeploy_core::artifact::{ArtifactManifest, write_manifest};
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tempfile::tempdir;

    use super::*;

    const SITE: &str = "demo";
    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

    fn archive(entries: &[(&str, &[u8])]) -> Result<Vec<u8>> {
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        let mut tar = tar::Builder::new(&mut gzip);
        for (path, contents) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append_data(&mut header, path, *contents)?;
        }
        tar.finish()?;
        drop(tar);
        Ok(gzip.finish()?)
    }

    fn framed(payload: &[u8]) -> Result<Vec<u8>> {
        let digest = format!("{:x}", Sha256::digest(payload));
        let manifest = ArtifactManifest::new(SITE.into(), REVISION.into(), payload.len() as u64, &digest);
        let mut frame = Vec::new();
        write_manifest(&mut frame, &manifest)?;
        frame.extend_from_slice(payload);
        Ok(frame)
    }

    fn framed_manifest(manifest: &ArtifactManifest, payload: &[u8]) -> Result<Vec<u8>> {
        let mut frame = Vec::new();
        write_manifest(&mut frame, &manifest)?;
        frame.extend_from_slice(payload);
        Ok(frame)
    }

    fn special_archive(entry_type: tar::EntryType, path: &str, link: Option<&str>) -> Result<Vec<u8>> {
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        let mut tar = tar::Builder::new(&mut gzip);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(entry_type);
        header.set_size(0);
        header.set_mode(0o755);
        if let Some(link) = link {
            header.set_link_name(link)?;
        }
        header.set_cksum();
        tar.append_data(&mut header, path, Cursor::new([]))?;
        tar.finish()?;
        drop(tar);
        Ok(gzip.finish()?)
    }

    #[test]
    fn receipt_materializes_regular_files_with_declared_digest() -> Result<()> {
        let context = tempdir()?;
        let mut frame = Cursor::new(framed(&archive(&[("bin/run", b"#!/bin/sh")])?)?);
        receive(&mut frame, SITE, REVISION, context.path())?;
        assert_eq!(fs::read(context.path().join("bin/run"))?, b"#!/bin/sh");
        assert_eq!(fs::metadata(context.path().join("bin/run"))?.permissions().mode() & 0o111, 0o111);
        Ok(())
    }

    #[test]
    fn receipt_rejects_digest_mismatch_and_trailing_transport_data() -> Result<()> {
        let payload = archive(&[("file", b"contents")])?;
        let mut bad_digest = framed(&payload)?;
        if let Some(last) = bad_digest.last_mut() {
            *last ^= 1;
        } else {
            bail!("test artifact frame is empty");
        }
        assert!(receive(&mut Cursor::new(bad_digest), SITE, REVISION, tempdir()?.path()).is_err());

        let mut trailing = framed(&payload)?;
        trailing.push(0);
        assert!(receive(&mut Cursor::new(trailing), SITE, REVISION, tempdir()?.path()).is_err());
        Ok(())
    }

    #[test]
    fn receipt_rejects_truncated_frames_and_payloads() -> Result<()> {
        assert!(receive(&mut Cursor::new(vec![0, 0, 0]), SITE, REVISION, tempdir()?.path()).is_err());
        let payload = archive(&[("file", b"contents")])?;
        let mut frame = framed(&payload)?;
        frame.pop();
        assert!(receive(&mut Cursor::new(frame), SITE, REVISION, tempdir()?.path()).is_err());
        Ok(())
    }

    #[test]
    fn receipt_rejects_wrong_identity_and_oversized_declared_payload() -> Result<()> {
        let payload = archive(&[("file", b"contents")])?;
        let digest = format!("{:x}", Sha256::digest(&payload));
        let wrong_site = ArtifactManifest::new("other".into(), REVISION.into(), payload.len() as u64, &digest);
        assert!(
            receive(&mut Cursor::new(framed_manifest(&wrong_site, &payload)?), SITE, REVISION, tempdir()?.path())
                .is_err()
        );
        let wrong_revision = ArtifactManifest::new(
            SITE.into(),
            "fedcba9876543210fedcba9876543210fedcba98".into(),
            payload.len() as u64,
            &digest,
        );
        assert!(
            receive(&mut Cursor::new(framed_manifest(&wrong_revision, &payload)?), SITE, REVISION, tempdir()?.path())
                .is_err()
        );
        let oversized = ArtifactManifest::new(SITE.into(), REVISION.into(), MAX_COMPRESSED_BYTES + 1, &digest);
        assert!(
            receive(&mut Cursor::new(framed_manifest(&oversized, &[])?), SITE, REVISION, tempdir()?.path()).is_err()
        );
        Ok(())
    }

    #[test]
    fn receipt_rejects_parent_paths_and_duplicate_paths() -> Result<()> {
        assert!(checked_path(Path::new("../escape")).is_err());
        assert!(checked_path(Path::new("/escape")).is_err());
        let mut duplicate = Cursor::new(framed(&archive(&[("file", b"one"), ("file", b"two")])?)?);
        assert!(receive(&mut duplicate, SITE, REVISION, tempdir()?.path()).is_err());
        Ok(())
    }

    #[test]
    fn receipt_rejects_conflicts_unsafe_links_and_special_entries() -> Result<()> {
        let mut conflict = Cursor::new(framed(&archive(&[("file", b"one"), ("file/child", b"two")])?)?);
        assert!(receive(&mut conflict, SITE, REVISION, tempdir()?.path()).is_err());
        let unsafe_link = special_archive(tar::EntryType::Symlink, "link", Some("../escape"))?;
        assert!(receive(&mut Cursor::new(framed(&unsafe_link)?), SITE, REVISION, tempdir()?.path()).is_err());
        for entry_type in [tar::EntryType::Link, tar::EntryType::Fifo] {
            let payload = special_archive(entry_type, "special", Some("target"))?;
            assert!(receive(&mut Cursor::new(framed(&payload)?), SITE, REVISION, tempdir()?.path()).is_err());
        }
        Ok(())
    }

    #[test]
    fn receipt_preserves_directory_modes_and_limits_symlink_targets() -> Result<()> {
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        let mut tar = tar::Builder::new(&mut gzip);
        let mut directory = tar::Header::new_gnu();
        directory.set_entry_type(tar::EntryType::Directory);
        directory.set_size(0);
        directory.set_mode(0o750);
        directory.set_cksum();
        tar.append_data(&mut directory, "private", Cursor::new([]))?;
        tar.finish()?;
        drop(tar);
        let payload = gzip.finish()?;
        let context = tempdir()?;
        receive(&mut Cursor::new(framed(&payload)?), SITE, REVISION, context.path())?;
        assert_eq!(fs::metadata(context.path().join("private"))?.permissions().mode() & 0o777, 0o750);
        assert!(checked_link_target(Path::new("link"), Path::new(&"x".repeat(MAX_ARTIFACT_PATH_BYTES + 1))).is_err());
        Ok(())
    }
}
