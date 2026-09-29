use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;

use anyhow::{Result, bail};
use bonesdeploy_core::artifact::{self, ComposeImage};
use tempfile::TempDir;

use super::{active_start_with, active_stop_with};

#[test]
fn active_start_reconciles_current_with_wait_timeout() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut calls = Vec::new();

    active_start_with("atlas", fixture.root(), 45, &mut |command, action| {
        calls.push(call(command, action));
        Ok(())
    })?;

    assert_eq!(actions(&calls), ["start Compose stack"]);
    assert_eq!(
        operation(&calls[0])?,
        ["up", "--detach", "--no-build", "--pull", "never", "--remove-orphans", "--wait", "--wait-timeout", "45"]
    );
    assert_eq!(calls[0].directory, fixture.root().join("current"));
    Ok(())
}

#[test]
fn active_stop_never_removes_containers_or_volumes() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut calls = Vec::new();

    active_stop_with("atlas", fixture.root(), &mut |command, action| {
        calls.push(call(command, action));
        Ok(())
    })?;

    assert_eq!(actions(&calls), ["stop Compose stack"]);
    assert_eq!(operation(&calls[0])?, ["stop"]);
    assert!(!calls[0].arguments.iter().any(|argument| matches!(argument.as_str(), "down" | "rm" | "--volumes" | "-v")));
    Ok(())
}

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
    super::validate_release_override(&fixture.root().join("current-release"), slice::from_ref(&image))?;

    let files = super::ComposeFiles::discover(&fixture.root().join("current-release"))?;
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

    assert!(super::validate_release_override(&release, slice::from_ref(&image)).is_err());
    fs::write(&path, "services: {}\n")?;
    assert!(super::validate_release_override(&release, slice::from_ref(&image)).is_err());
    fs::remove_file(&path)?;
    let outside = fixture.root().join("override-outside");
    fs::write(&outside, artifact::compose_override_contents(slice::from_ref(&image)))?;
    symlink(&outside, &path)?;
    assert!(super::validate_release_override(&release, slice::from_ref(&image)).is_err());
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

    assert_eq!(super::unreferenced_release_image_tags(&discarded, &[retained])?, ["discarded"]);
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

    assert_eq!(super::unreferenced_image_tags(&["shared".into(), "failed".into()], &[retained])?, ["failed"]);
    Ok(())
}

#[test]
fn manifest_cleanup_removes_pre_promotion_tags_and_preserves_retained_or_absent_tags() -> Result<()> {
    let fixture = Fixture::new()?;
    let discarded = fixture.root().join("discarded");
    let retained = fixture.root().join("retained");
    fs::create_dir(&discarded)?;
    fs::create_dir(&retained)?;
    fs::write(
        discarded.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[
            serde_json::json!({"service":"web", "tag":"pre-promotion"}),
            serde_json::json!({"service":"job", "tag":"shared"}),
            serde_json::json!({"service":"worker", "tag":"already-absent"}),
        ])?,
    )?;
    fs::write(
        retained.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
        serde_json::to_vec(&[serde_json::json!({"service":"job", "tag":"shared"})])?,
    )?;
    let mut executor = FakeImageExecutor::new(["pre-promotion", "shared"]);

    super::remove_unreferenced_release_images_with(&discarded, &[retained], &mut executor)?;

    assert_eq!(executor.removed, ["pre-promotion"]);
    assert!(executor.images.contains("shared"));
    assert!(!executor.images.contains("pre-promotion"));
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

struct RecordedCall {
    action: String,
    arguments: Vec<String>,
    directory: PathBuf,
}

fn call(command: &Command, action: &str) -> RecordedCall {
    RecordedCall {
        action: action.to_owned(),
        arguments: command.get_args().map(|argument| argument.to_string_lossy().into_owned()).collect(),
        directory: command.get_current_dir().unwrap_or_else(|| Path::new("")).to_owned(),
    }
}

fn actions(calls: &[RecordedCall]) -> Vec<&str> {
    calls.iter().map(|call| call.action.as_str()).collect()
}

fn operation(call: &RecordedCall) -> Result<Vec<&str>> {
    let Some(file) = call.arguments.iter().rposition(|argument| argument == "--file") else {
        bail!("Compose command must identify an explicit file");
    };
    Ok(call.arguments[file + 2..].iter().map(String::as_str).collect())
}

struct FakeImageExecutor {
    images: HashSet<String>,
    removed: Vec<String>,
}

impl FakeImageExecutor {
    fn new(tags: impl IntoIterator<Item = &'static str>) -> Self {
        Self { images: tags.into_iter().map(String::from).collect(), removed: Vec::new() }
    }
}

impl super::ImageExecutor for FakeImageExecutor {
    fn remove(&mut self, tag: &str) -> Result<bool> {
        if self.images.remove(tag) {
            self.removed.push(tag.to_owned());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn exists(&mut self, tag: &str) -> Result<bool> {
        Ok(self.images.contains(tag))
    }
}
