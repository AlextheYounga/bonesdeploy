use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::artifact::{self, ComposeImage};
use bonesdeploy_core::build_contract::{self, TARGET_PLATFORM_NAME};
use bonesdeploy_core::config::{Bones, validate_site_name};
use serde::Deserialize;
use tempfile::NamedTempFile;

use super::{docker::DockerClient, source::BuildContext};

const BASE_FILES: [&str; 4] = ["compose.yaml", "compose.yml", "docker-compose.yaml", "docker-compose.yml"];
const OVERRIDE_FILES: [&str; 2] = ["compose.override.yaml", "compose.override.yml"];
pub fn build(config: &Bones, context: &BuildContext, docker: &DockerClient) -> Result<Vec<ComposeImage>> {
    reserve_artifact_paths(context.path())?;
    let files = ComposeFiles::discover(context.path())?;
    let environment = write_environment_file(&build_contract::environment(config, context.path())?)?;
    let compose = ComposeCommand {
        docker,
        project_directory: context.path(),
        environment_file: environment.path(),
        files: &files,
    };

    run(compose.command(config, ["config", "--quiet"])?, "validate Compose configuration")?;
    run(compose.command(config, ["pull"])?, "pull Compose images")?;
    run(compose.command(config, ["build"])?, "build Compose images")?;

    let output = compose
        .command(config, ["config", "--format", "json"])?
        .output()
        .context("Failed to discover Compose service images")?;
    if !output.status.success() {
        bail!("Failed to discover Compose service images: {}", output.status);
    }
    let images = inventory(config, &context.revision, &output.stdout)?;
    tag_images(docker, config, &images, &output.stdout)?;
    write_release_files(context.path(), &images)?;
    save_images(docker, context.path(), &images)?;
    Ok(images)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComposeFiles {
    paths: Vec<PathBuf>,
}

impl ComposeFiles {
    fn discover(source: &Path) -> Result<Self> {
        let base = existing_files(source, &BASE_FILES)?;
        let overrides = existing_files(source, &OVERRIDE_FILES)?;
        let base_file = match base.as_slice() {
            [] => bail!("No Compose file found in {}", source.display()),
            [file] => file,
            _ => bail!("Multiple Compose base files found in {}", source.display()),
        };
        if overrides.len() > 1 {
            bail!("Multiple Compose override files found in {}", source.display());
        }
        let mut paths = vec![base_file.clone()];
        paths.extend(overrides);
        Ok(Self { paths })
    }
}

struct ComposeCommand<'a> {
    docker: &'a DockerClient,
    project_directory: &'a Path,
    environment_file: &'a Path,
    files: &'a ComposeFiles,
}

impl ComposeCommand<'_> {
    fn command<I, S>(&self, config: &Bones, arguments: I) -> Result<Command>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        validate_site_name(&config.project_name)?;
        let mut command = self.docker.command()?;
        command
            .env_clear()
            .env("DOCKER_DEFAULT_PLATFORM", TARGET_PLATFORM_NAME)
            .args(["compose", "--project-name", &format!("bonesdeploy-{}", config.project_name)])
            .args(["--project-directory", self.project_directory.to_string_lossy().as_ref()])
            .args(["--env-file", self.environment_file.to_string_lossy().as_ref()]);
        self.docker.apply_after_env_clear(&mut command)?;
        for file in &self.files.paths {
            command.args(["--file", file.to_string_lossy().as_ref()]);
        }
        command.args(arguments).current_dir(self.project_directory);
        Ok(command)
    }
}

#[derive(Deserialize)]
struct ComposeConfig {
    services: BTreeMap<String, ComposeService>,
}

#[derive(Deserialize)]
struct ComposeService {
    image: Option<String>,
    #[serde(default)]
    platform: Option<String>,
}

fn inventory(config: &Bones, revision: &str, output: &[u8]) -> Result<Vec<ComposeImage>> {
    let compose: ComposeConfig =
        serde_json::from_slice(output).context("Docker Compose returned invalid configuration JSON")?;
    if compose.services.is_empty() {
        bail!("Compose configuration contains no services");
    }
    compose
        .services
        .into_iter()
        .map(|(service, definition)| {
            if definition.platform.as_deref().is_some_and(|platform| platform != TARGET_PLATFORM_NAME) {
                bail!(
                    "Compose service `{service}` targets `{}` instead of {TARGET_PLATFORM_NAME}",
                    definition.platform.unwrap_or_default()
                );
            }
            ComposeImage::new(&config.project_name, service, revision)
        })
        .collect()
}

fn source_images(config: &Bones, output: &[u8]) -> Result<BTreeMap<String, String>> {
    let compose: ComposeConfig =
        serde_json::from_slice(output).context("Docker Compose returned invalid configuration JSON")?;
    Ok(compose
        .services
        .into_iter()
        .map(|(service, definition)| {
            // Compose names a build-only image from the stable project and service names.
            let image = definition.image.unwrap_or_else(|| format!("bonesdeploy-{}-{}", config.project_name, service));
            (service, image)
        })
        .collect())
}

fn tag_images(docker: &DockerClient, config: &Bones, images: &[ComposeImage], output: &[u8]) -> Result<()> {
    let source_images = source_images(config, output)?;
    for image in images {
        let source = source_images
            .get(&image.service)
            .with_context(|| format!("Compose configuration omitted image for service `{}`", image.service))?;
        run(image_tag_command(docker, source, &image.tag)?, &format!("tag image for service `{}`", image.service))?;
    }
    Ok(())
}

fn write_release_files(source: &Path, images: &[ComposeImage]) -> Result<()> {
    write_new_file(
        &source.join(artifact::COMPOSE_OVERRIDE_FILE),
        artifact::compose_override_contents(images).as_bytes(),
    )?;
    write_new_file(&source.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE), &serde_json::to_vec_pretty(images)?)?;
    Ok(())
}

fn save_images(docker: &DockerClient, source: &Path, images: &[ComposeImage]) -> Result<()> {
    let output = source.join(artifact::COMPOSE_IMAGE_ARCHIVE_FILE);
    let temporary = tempfile::Builder::new()
        .prefix(".bonesdeploy-compose-images-")
        .tempfile_in(source)
        .context("Failed to create private Compose image archive file")?;
    run(image_save_command(docker, temporary.path(), images)?, "save Compose images into artifact")?;
    persist_no_clobber(temporary.path(), &output)
}

fn reserve_artifact_paths(source: &Path) -> Result<()> {
    for name in
        [artifact::COMPOSE_OVERRIDE_FILE, artifact::COMPOSE_IMAGE_INVENTORY_FILE, artifact::COMPOSE_IMAGE_ARCHIVE_FILE]
    {
        let path = source.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => bail!("Reserved Compose artifact path already exists: {}", path.display()),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to inspect reserved Compose artifact path {}", path.display()));
            }
        }
    }
    Ok(())
}

fn write_new_file(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("Failed to create generated Compose artifact {}", path.display()))?;
    file.write_all(contents)
        .with_context(|| format!("Failed to write generated Compose artifact {}", path.display()))?;
    Ok(())
}

fn persist_no_clobber(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("Failed to inspect temporary Compose image archive {}", source.display()))?;
    if !metadata.is_file() {
        bail!("Temporary Compose image archive is not a regular file: {}", source.display());
    }
    fs::hard_link(source, destination).with_context(|| {
        format!("Failed to persist Compose image archive without replacing {}", destination.display())
    })?;
    Ok(())
}

fn write_environment_file(environment: &[(String, String)]) -> Result<NamedTempFile> {
    let mut file = tempfile::Builder::new().prefix("bonesdeploy-compose-env-").tempfile()?;
    for (key, value) in environment {
        writeln!(file, "{key}={value}")?;
    }
    Ok(file)
}

fn image_tag_command(docker: &DockerClient, source: &str, tag: &str) -> Result<Command> {
    let mut command = docker.command()?;
    command.args(["image", "tag", source, tag]);
    Ok(command)
}

fn image_save_command(docker: &DockerClient, output: &Path, images: &[ComposeImage]) -> Result<Command> {
    let mut command = docker.command()?;
    command.args(["image", "save", "--output"]).arg(output);
    command.args(images.iter().map(|image| image.tag.as_str()));
    Ok(command)
}

fn run(mut command: Command, action: &str) -> Result<()> {
    let status = command.status().with_context(|| format!("Failed to {action}"))?;
    if status.success() { Ok(()) } else { bail!("Failed to {action}: {status}") }
}

fn existing_files(source: &Path, candidates: &[&str]) -> Result<Vec<PathBuf>> {
    candidates
        .iter()
        .filter_map(|name| {
            let path = source.join(name);
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.is_file() => Some(Ok(path)),
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    Some(Err(anyhow::anyhow!("Compose file must not be a symlink: {}", path.display())))
                }
                Ok(_) => Some(Err(anyhow::anyhow!("Compose file is not a regular file: {}", path.display()))),
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                Err(error) => {
                    Some(Err(error).with_context(|| format!("Failed to inspect Compose file {}", path.display())))
                }
            }
        })
        .collect()
}
