use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::slice;

use anyhow::Result;
use bonesdeploy_core::artifact::{self, ComposeImage};
use bonesremote::runtime::docker::command;
use tempfile::TempDir;

#[test]
fn generated_override_is_last_and_pins_release_image_tags() -> Result<()> {
    let fixture = Fixture::new()?;
    fs::write(fixture.root().join("current-release/compose.override.yaml"), "services: {}\n")?;
    let revision = "0123456789abcdef0123456789abcdef01234567";
    let image = ComposeImage::new("atlas", "web".into(), revision)?;
    fs::write(
        fixture.root().join("current-release").join(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(slice::from_ref(&image))?,
    )?;
    fs::write(
        fixture.root().join("current-release").join(artifact::COMPOSE_OVERRIDE_FILE),
        artifact::compose_override_contents(slice::from_ref(&image)),
    )?;
    command::validate_release_override(&fixture.root().join("current-release"), slice::from_ref(&image))?;

    let files = command::ComposeFiles::discover(&fixture.root().join("current-release"))?;
    assert_eq!(
        files
            .paths()
            .iter()
            .map(|path| path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned()))
            .collect::<Vec<_>>(),
        ["compose.yaml", "compose.override.yaml", ".bonesdeploy-compose.override.yaml"]
    );
    assert!(
        fs::read_to_string(fixture.root().join("current-release/.bonesdeploy-compose.override.yaml"))?
            .contains(&image.tag)
    );
    Ok(())
}

#[test]
fn generated_override_validation_rejects_missing_mismatched_and_symlinked_files() -> Result<()> {
    let fixture = Fixture::new()?;
    let release = fixture.root().join("current-release");
    let revision = "0123456789abcdef0123456789abcdef01234567";
    let image = ComposeImage::new("atlas", "web".into(), revision)?;
    let path = release.join(artifact::COMPOSE_OVERRIDE_FILE);

    assert!(command::validate_release_override(&release, slice::from_ref(&image)).is_err());
    fs::write(&path, "services: {}\n")?;
    assert!(command::validate_release_override(&release, slice::from_ref(&image)).is_err());
    fs::remove_file(&path)?;
    let outside = fixture.root().join("override-outside");
    fs::write(&outside, artifact::compose_override_contents(slice::from_ref(&image)))?;
    symlink(&outside, &path)?;
    assert!(command::validate_release_override(&release, slice::from_ref(&image)).is_err());
    Ok(())
}

#[test]
fn pruning_preserves_tags_referenced_by_retained_releases() -> Result<()> {
    let fixture = Fixture::new()?;
    let discarded = fixture.root().join("discarded");
    let retained = fixture.root().join("retained");
    fs::create_dir(&discarded)?;
    fs::create_dir(&retained)?;
    fs::write(
        discarded.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[
            serde_json::json!({"service":"web", "tag":"shared"}),
            serde_json::json!({"service":"job", "tag":"discarded"}),
        ])?,
    )?;
    fs::write(
        retained.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[
            serde_json::json!({"service":"web", "tag":"shared"}),
            serde_json::json!({"service":"job", "tag":"retained"}),
        ])?,
    )?;

    assert_eq!(command::unreferenced_release_image_tags(&discarded, &[retained])?, ["discarded"]);
    Ok(())
}

#[test]
fn manifest_cleanup_calculates_only_unretained_tags() -> Result<()> {
    let fixture = Fixture::new()?;
    let retained = fixture.root().join("retained-manifest");
    fs::create_dir(&retained)?;
    fs::write(
        retained.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[serde_json::json!({"service":"web", "tag":"shared"})])?,
    )?;

    assert_eq!(command::unreferenced_image_tags(&["shared".into(), "failed".into()], &[retained])?, ["failed"]);
    Ok(())
}

struct Fixture {
    directory: TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let directory = TempDir::new()?;
        fs::create_dir_all(directory.path().join("shared"))?;
        let current = directory.path().join("current-release");
        fs::create_dir(&current)?;
        fs::write(current.join("compose.yaml"), "services: {}\n")?;
        symlink(&current, directory.path().join("current"))?;
        Ok(Self { directory })
    }

    fn root(&self) -> &Path {
        self.directory.path()
    }
}
