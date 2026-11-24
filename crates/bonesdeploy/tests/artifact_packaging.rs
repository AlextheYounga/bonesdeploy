use std::env;
use std::fs::{self, File, Permissions};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use bonesdeploy::build::{artifact, source};
use bonesdeploy_core::artifact::{ArtifactKind, ComposeImage};
use bonesdeploy_core::config::Bones;
use flate2::read::GzDecoder;

#[test]
fn packaging_preserves_safe_content_and_rejects_escaping_links() -> Result<()> {
    let repo = tempfile::tempdir()?;
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "package_in_repository", "--nocapture"])
        .current_dir(repo.path())
        .env("BONES_PACKAGE_TEST", "1")
        .output()?;
    ensure!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn package_in_repository() -> Result<()> {
    if env::var_os("BONES_PACKAGE_TEST").is_none() {
        return Ok(());
    }
    for args in
        [&["init", "-b", "main"][..], &["config", "user.email", "test@example.com"], &["config", "user.name", "Test"]]
    {
        ensure!(Command::new("git").args(args).status()?.success());
    }
    fs::write("tracked", "source")?;
    ensure!(Command::new("git").args(["add", "tracked"]).status()?.success());
    ensure!(Command::new("git").args(["commit", "-m", "fixture"]).status()?.success());
    let mut config = Bones::for_site("atlas");
    config.branch = "main".into();
    let build = source::export(&config)?;
    let root = build.path();
    fs::create_dir(root.join("nested"))?;
    fs::write(root.join("nested/run.sh"), "#!/bin/sh\n")?;
    fs::set_permissions(root.join("nested/run.sh"), Permissions::from_mode(0o755))?;
    fs::write(root.join(".env"), "secret")?;
    fs::write(root.join(".env.build"), "build")?;
    symlink("run.sh", root.join("nested/current"))?;
    let component = "segment-with-a-realistic-name-".repeat(2);
    let nested = root.join(&component).join(&component).join(&component);
    fs::create_dir_all(&nested)?;
    fs::write(nested.join("output"), "built")?;
    let target = Path::new("..").join(&component).join("output");
    symlink(&target, nested.join("current"))?;

    let packaged = artifact::package("atlas", &build)?;
    assert_eq!(packaged.manifest.artifact_length, fs::metadata(packaged.path())?.len());
    let mut archive = tar::Archive::new(GzDecoder::new(File::open(packaged.path())?));
    let mut executable = false;
    let mut safe_link = false;
    let mut long_link = false;
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        assert!(path != Path::new(".env") && path != Path::new(".env.build"));
        if path == Path::new("nested/run.sh") {
            executable = entry.header().mode()? & 0o111 != 0;
        }
        if path == Path::new("nested/current") {
            safe_link = entry.link_name()?.as_deref() == Some(Path::new("run.sh"));
        }
        if path == Path::new(&component).join(&component).join(&component).join("current") {
            long_link = entry.link_name()?.as_deref() == Some(target.as_path());
        }
    }
    assert!(executable && safe_link && long_link);
    let image = ComposeImage::new("atlas", "web".into(), &build.revision)?;
    assert!(matches!(
        artifact::package_compose("atlas", &build, vec![image])?.manifest.kind,
        ArtifactKind::ComposeImages { .. }
    ));
    symlink("../../outside", root.join("escape"))?;
    let error = artifact::package("atlas", &build).err().context("escaping symlink must be rejected")?;
    assert!(error.to_string().contains("escapes the build context"));
    Ok(())
}
