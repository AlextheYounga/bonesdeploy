use std::fs;
use std::io::{Cursor, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{Result, bail};
use bonesremote::shared::{ArchiveLimits, extract_archive};
use tempfile::tempdir;
use zip::write::{FileOptions, ZipWriter};

type Entry<'a> = (&'a str, u32, &'a [u8]);

fn archive(entries: &[Entry<'_>]) -> Result<Cursor<Vec<u8>>> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, mode, contents) in entries {
        let options = FileOptions::<()>::default().unix_permissions(*mode);
        if name.ends_with('/') {
            writer.add_directory(*name, options)?;
        } else if mode & 0o170_000 == 0o120_000 {
            writer.add_symlink(*name, str::from_utf8(contents)?, options)?;
        } else {
            writer.start_file(*name, options)?;
            writer.write_all(contents)?;
        }
    }
    Ok(Cursor::new(writer.finish()?.into_inner()))
}

fn special_file_archive() -> Result<Cursor<Vec<u8>>> {
    let mut bytes = archive(&[("shared/device", 0o100644, b"x")])?.into_inner();
    let Some(central) = bytes.windows(4).rposition(|signature| signature == b"PK\x01\x02") else {
        bail!("test ZIP should have a central directory");
    };
    bytes[central + 4..central + 6].copy_from_slice(&(3_u16 << 8 | 20).to_le_bytes());
    bytes[central + 38..central + 42].copy_from_slice(&(0o010644_u32 << 16).to_le_bytes());
    Ok(Cursor::new(bytes))
}

fn extract(entries: &[Entry<'_>], limits: ArchiveLimits) -> Result<PathBuf> {
    let destination = tempdir()?;
    let path = destination.keep();
    extract_archive(archive(entries)?, &path, limits)?;
    Ok(path)
}

fn limits() -> ArchiveLimits {
    ArchiveLimits { min_free_bytes: 0, ..ArchiveLimits::default() }
}

#[test]
fn extracts_shared_root_omits_env_and_preserves_safe_relative_symlink() -> Result<()> {
    let root = extract(
        &[
            ("shared/", 0o40755, &[]),
            ("shared/bin/", 0o40755, &[]),
            ("shared/bin/run", 0o100755, b"run"),
            ("shared/.env", 0o100600, b"SECRET=must-not-import\n"),
            ("shared/current", 0o120777, b"bin/run"),
        ],
        limits(),
    )?;

    assert_eq!(fs::read(root.join("bin/run"))?, b"run");
    assert!(!root.join(".env").exists());
    assert_eq!(fs::read_link(root.join("current"))?, Path::new("bin/run"));
    assert_eq!(fs::metadata(root.join("bin/run"))?.permissions().mode() & 0o007, 0);
    assert_eq!(fs::metadata(root.join("bin"))?.permissions().mode() & 0o007, 0);
    Ok(())
}

#[test]
fn accepts_file_without_explicit_shared_directory_entry() -> Result<()> {
    let root = extract(&[("shared/file", 0o100644, b"contents")], limits())?;

    assert_eq!(fs::read_to_string(root.join("file"))?, "contents");
    Ok(())
}

#[test]
fn rejects_malformed_zip() -> Result<()> {
    let destination = tempdir()?;
    let result = extract_archive(Cursor::new(b"not a ZIP archive"), destination.path(), limits());

    match result {
        Ok(()) => bail!("malformed ZIP should be rejected"),
        Err(error) => assert!(error.to_string().contains("valid ZIP")),
    }
    Ok(())
}

#[test]
fn rejects_overlong_path_and_symlink_target() -> Result<()> {
    let long_path = format!("shared/{}", "a".repeat(20));
    let destination = tempdir()?;
    let result = extract_archive(
        archive(&[(long_path.as_str(), 0o100644, b"x")])?,
        destination.path(),
        ArchiveLimits { max_path_bytes: 10, min_free_bytes: 0, ..limits() },
    );
    assert!(result.is_err(), "overlong path should be rejected");

    let long_target = "a".repeat(20);
    let destination = tempdir()?;
    let result = extract_archive(
        archive(&[("shared/link", 0o120777, long_target.as_bytes())])?,
        destination.path(),
        ArchiveLimits { max_path_bytes: 10, min_free_bytes: 0, ..limits() },
    );
    assert!(result.is_err(), "overlong symlink target should be rejected");
    Ok(())
}

#[test]
fn implicit_parent_directories_have_no_world_bits() -> Result<()> {
    let root = extract(&[("shared/nested/file", 0o100644, b"contents")], limits())?;

    assert_eq!(fs::metadata(root.join("nested"))?.permissions().mode() & 0o007, 0);
    Ok(())
}

#[test]
fn rejects_invalid_paths_duplicates_conflicts_and_special_files() -> Result<()> {
    let cases: &[(&[Entry<'_>], &str)] = &[
        (&[("/shared/file", 0o100644, b"x" as &[u8])], "absolute path"),
        (&[("shared/../outside", 0o100644, b"x")], "traversal"),
        (&[("other/file", 0o100644, b"x")], "outside root"),
        (&[("shared/file", 0o100644, b"x"), ("shared/./file", 0o100644, b"y")], "duplicate"),
        (&[("shared/file", 0o100644, b"x"), ("shared/file/child", 0o100644, b"y")], "path conflict"),
    ];
    for (entries, name) in cases {
        let destination = tempdir()?;
        match extract_archive(archive(entries)?, destination.path(), limits()) {
            Ok(()) => bail!("{name} should have an error"),
            Err(error) => assert!(!error.to_string().is_empty(), "{name} should have an error"),
        }
    }

    let destination = tempdir()?;
    match extract_archive(special_file_archive()?, destination.path(), limits()) {
        Ok(()) => bail!("special file should have an error"),
        Err(error) => assert!(!error.to_string().is_empty()),
    }
    Ok(())
}

#[test]
fn rejects_entry_and_expanded_size_limits() -> Result<()> {
    let destination = tempdir()?;
    let result = extract_archive(
        archive(&[("shared/a", 0o100644, b"x"), ("shared/b", 0o100644, b"y")])?,
        destination.path(),
        ArchiveLimits { max_entries: 1, min_free_bytes: 0, ..limits() },
    );
    match result {
        Ok(()) => bail!("entry limit should be enforced"),
        Err(error) => assert!(error.to_string().contains("entries")),
    }

    let destination = tempdir()?;
    let result = extract_archive(
        archive(&[("shared/a", 0o100644, b"xx")])?,
        destination.path(),
        ArchiveLimits { max_expanded_bytes: 1, min_free_bytes: 0, ..limits() },
    );
    match result {
        Ok(()) => bail!("expanded limit should be enforced"),
        Err(error) => assert!(error.to_string().contains("expanded")),
    }
    Ok(())
}

#[test]
fn rejects_absolute_and_outside_root_symlink_targets() -> Result<()> {
    for target in [b"/tmp/outside".as_slice(), b"../../outside"] {
        let destination = tempdir()?;
        match extract_archive(archive(&[("shared/link", 0o120777, target)])?, destination.path(), limits()) {
            Ok(()) => bail!("unsafe symlink should be rejected"),
            Err(error) => assert!(error.to_string().contains("symlink")),
        }
    }
    Ok(())
}
