use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::artifact::{ArtifactManifest, write_manifest};
use flate2::Compression;
use flate2::write::GzEncoder;
use sha2::{Digest, Sha256};
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
    framed_manifest(&manifest, payload)
}

fn framed_manifest(manifest: &ArtifactManifest, payload: &[u8]) -> Result<Vec<u8>> {
    let mut frame = Vec::new();
    write_manifest(&mut frame, manifest)?;
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
        receive(&mut Cursor::new(framed_manifest(&wrong_site, &payload)?), SITE, REVISION, tempdir()?.path()).is_err()
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
    assert!(oversized.validate().is_err());
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
