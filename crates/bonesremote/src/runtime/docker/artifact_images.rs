use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::artifact::{self, ComposeImage};

pub fn load_artifact_images(context: &Path, images: &[ComposeImage]) -> Result<()> {
    let archive = context.join(artifact::COMPOSE_IMAGE_ARCHIVE_FILE);
    if !archive.is_file() {
        bail!("Compose artifact is missing image archive {}", archive.display());
    }
    let status = Command::new("docker")
        .args(["load", "--input"])
        .arg(&archive)
        .status()
        .context("Failed to load Compose artifact images")?;
    if !status.success() {
        bail!("Failed to load Compose artifact images: {status}");
    }
    for image in images {
        let status = Command::new("docker")
            .args(["image", "inspect", &image.tag])
            .status()
            .with_context(|| format!("Failed to verify loaded Compose image {}", image.tag))?;
        if !status.success() {
            bail!("Compose artifact did not load required image {}", image.tag);
        }
    }
    Ok(())
}

pub fn validate_artifact_images(context: &Path, images: &[ComposeImage]) -> Result<()> {
    let inventory = context.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE);
    let contents = fs::read(&inventory)
        .with_context(|| format!("Compose artifact is missing image inventory {}", inventory.display()))?;
    let declared: Vec<ComposeImage> =
        serde_json::from_slice(&contents).context("Compose artifact image inventory is not valid JSON")?;
    if declared != images {
        bail!("Compose artifact image inventory does not match the manifest");
    }
    Ok(())
}

pub fn validate_release_override(release: &Path, images: &[ComposeImage]) -> Result<()> {
    let override_path = release.join(artifact::COMPOSE_OVERRIDE_FILE);
    let metadata = fs::symlink_metadata(&override_path)
        .with_context(|| format!("Compose artifact is missing generated override {}", override_path.display()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("Generated Compose override is not a regular non-symlink file: {}", override_path.display());
    }
    let contents = fs::read(&override_path)
        .with_context(|| format!("Failed to read generated Compose override {}", override_path.display()))?;
    if contents != artifact::compose_override_contents(images).as_bytes() {
        bail!("Generated Compose override does not match the artifact manifest: {}", override_path.display());
    }
    Ok(())
}

pub fn release_image_tags(release: &Path) -> Result<Vec<String>> {
    let inventory = release.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE);
    let contents = match fs::read(&inventory) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to read release image inventory {}", inventory.display()));
        }
    };
    let images: Vec<ComposeImage> = serde_json::from_slice(&contents)
        .with_context(|| format!("Release image inventory is not valid JSON: {}", inventory.display()))?;
    Ok(images.into_iter().map(|image| image.tag).collect())
}

pub fn remove_unreferenced_release_images(release: &Path, retained: &[PathBuf]) -> Result<()> {
    let mut executor = DockerImageExecutor;
    remove_unreferenced_release_images_with(release, retained, &mut executor)
}

pub fn remove_unreferenced_image_tags(tags: &[String], retained: &[PathBuf]) -> Result<()> {
    let mut executor = DockerImageExecutor;
    remove_unreferenced_image_tags_with(tags, retained, &mut executor)
}

pub(crate) fn remove_unreferenced_release_images_with(
    release: &Path,
    retained: &[PathBuf],
    executor: &mut impl ImageExecutor,
) -> Result<()> {
    remove_unreferenced_image_tags_with(&release_image_tags(release)?, retained, executor)
}

fn remove_unreferenced_image_tags_with(
    tags: &[String],
    retained: &[PathBuf],
    executor: &mut impl ImageExecutor,
) -> Result<()> {
    for tag in unreferenced_image_tags(tags, retained)? {
        if !executor.remove(&tag)? && executor.exists(&tag)? {
            bail!("Failed to remove release-specific Compose image {tag}");
        }
    }
    Ok(())
}

pub(crate) trait ImageExecutor {
    fn remove(&mut self, tag: &str) -> Result<bool>;
    fn exists(&mut self, tag: &str) -> Result<bool>;
}

struct DockerImageExecutor;

impl ImageExecutor for DockerImageExecutor {
    fn remove(&mut self, tag: &str) -> Result<bool> {
        Ok(Command::new("docker").args(["image", "rm", tag]).status()?.success())
    }

    fn exists(&mut self, tag: &str) -> Result<bool> {
        Ok(Command::new("docker").args(["image", "inspect", tag]).status()?.success())
    }
}

pub fn unreferenced_image_tags(tags: &[String], retained: &[PathBuf]) -> Result<Vec<String>> {
    let retained_tags = retained.iter().try_fold(Vec::new(), |mut tags, path| {
        tags.extend(release_image_tags(path)?);
        Ok::<_, anyhow::Error>(tags)
    })?;
    Ok(tags.iter().filter(|tag| !retained_tags.contains(tag)).cloned().collect())
}

pub fn unreferenced_release_image_tags(release: &Path, retained: &[PathBuf]) -> Result<Vec<String>> {
    let tags = release_image_tags(release)?;
    unreferenced_image_tags(&tags, retained)
}
