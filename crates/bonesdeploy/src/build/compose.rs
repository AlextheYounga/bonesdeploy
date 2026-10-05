use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::artifact::{self, ComposeImage};
use bonesdeploy_core::build_contract::{self, TARGET_PLATFORM_NAME};
use bonesdeploy_core::config::{Bones, build_timeout_seconds, validate_site_name};
use bonesdeploy_core::docker_env::PATH;
use serde::Deserialize;
use tempfile::NamedTempFile;

use super::{command, docker::DockerClient, source::BuildContext};

const BASE_FILES: [&str; 4] = ["compose.yaml", "compose.yml", "docker-compose.yaml", "docker-compose.yml"];
const OVERRIDE_FILES: [&str; 2] = ["compose.override.yaml", "compose.override.yml"];

pub fn build(config: &Bones, context: &mut BuildContext, docker: &DockerClient) -> Result<Vec<ComposeImage>> {
    reserve_artifact_paths(context.path())?;
    let files = ComposeFiles::discover(context.path())?;
    let environment = write_environment_file(&build_contract::environment(config, context.path())?)?;
    let compose = ComposeCommand {
        docker,
        project_directory: context.path(),
        environment_file: environment.path(),
        files: &files,
    };
    let timeout = build_timeout_seconds(config);

    run(compose.command(config, ["config", "--quiet"])?, "validate Compose configuration", timeout)?;
    run(compose.command(config, ["pull"])?, "pull Compose images", timeout)?;
    run(compose.command(config, ["build"])?, "build Compose images", timeout)?;

    let output = command::output(
        compose.command(config, ["config", "--format", "json"])?,
        "discover Compose service images",
        timeout,
    )?;
    let images = inventory(config, &context.revision, &output.stdout)?;
    let mut tags = TemporaryTags::new(docker, timeout);
    let result = (|| {
        tag_images(config, &images, &output.stdout, &mut tags)?;
        write_release_files(context.path(), &images)?;
        context.record_generated(
            Path::new(artifact::COMPOSE_OVERRIDE_FILE),
            super::inventory::EntryType::File,
            0o644,
        )?;
        context.record_generated(
            Path::new(artifact::COMPOSE_IMAGE_INVENTORY_FILE),
            super::inventory::EntryType::File,
            0o644,
        )?;
        save_images(docker, context.path(), &images, timeout)?;
        context.record_generated(
            Path::new(artifact::COMPOSE_IMAGE_ARCHIVE_FILE),
            super::inventory::EntryType::File,
            0o600,
        )?;
        Ok(images)
    })();
    finish_with_tag_cleanup(result, tags.remove())
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
        if let Some(path) = env::var_os(PATH) {
            command.env(PATH, path);
        }
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
            let image = definition.image.unwrap_or_else(|| format!("bonesdeploy-{}-{}", config.project_name, service));
            (service, image)
        })
        .collect())
}

fn tag_images(config: &Bones, images: &[ComposeImage], output: &[u8], tags: &mut TemporaryTags<'_>) -> Result<()> {
    let source_images = source_images(config, output)?;
    for image in images {
        let source = source_images
            .get(&image.service)
            .with_context(|| format!("Compose configuration omitted image for service `{}`", image.service))?;
        run(
            image_tag_command(tags.docker, source, &image.tag)?,
            &format!("tag image for service `{}`", image.service),
            tags.timeout,
        )?;
        tags.record(&image.tag);
    }
    Ok(())
}

fn write_release_files(source: &Path, images: &[ComposeImage]) -> Result<()> {
    write_new_file(
        &source.join(artifact::COMPOSE_OVERRIDE_FILE),
        artifact::compose_override_contents(images).as_bytes(),
    )?;
    write_new_file(&source.join(artifact::COMPOSE_IMAGE_INVENTORY_FILE), &serde_json::to_vec_pretty(images)?)
}

fn save_images(docker: &DockerClient, source: &Path, images: &[ComposeImage], timeout: Option<u64>) -> Result<()> {
    let output = source.join(artifact::COMPOSE_IMAGE_ARCHIVE_FILE);
    let temporary = tempfile::Builder::new()
        .prefix(".bonesdeploy-compose-images-")
        .tempfile_in(source)
        .context("Failed to create private Compose image archive file")?;
    run(image_save_command(docker, temporary.path(), images)?, "save Compose images into artifact", timeout)?;
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
    file.write_all(contents).with_context(|| format!("Failed to write generated Compose artifact {}", path.display()))
}

fn persist_no_clobber(source: &Path, destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("Failed to inspect temporary Compose image archive {}", source.display()))?;
    if !metadata.is_file() {
        bail!("Temporary Compose image archive is not a regular file: {}", source.display());
    }
    fs::hard_link(source, destination)
        .with_context(|| format!("Failed to persist Compose image archive without replacing {}", destination.display()))
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

fn image_remove_command(docker: &DockerClient, tag: &str) -> Result<Command> {
    let mut command = docker.command()?;
    command.args(["image", "rm", tag]);
    Ok(command)
}

fn run(command: Command, action: &str, timeout: Option<u64>) -> Result<()> {
    command::run(command, action, timeout)
}

struct TemporaryTags<'a> {
    docker: &'a DockerClient,
    tags: Vec<String>,
    timeout: Option<u64>,
}

impl<'a> TemporaryTags<'a> {
    fn new(docker: &'a DockerClient, timeout: Option<u64>) -> Self {
        Self { docker, tags: Vec::new(), timeout }
    }

    fn record(&mut self, tag: &str) {
        self.tags.push(tag.into());
    }

    fn remove(self) -> Result<()> {
        let mut failures = Vec::new();
        for tag in self.tags {
            if let Err(error) = run(
                image_remove_command(self.docker, &tag)?,
                &format!("remove generated Compose release tag `{tag}`"),
                self.timeout,
            ) {
                failures.push(format!("{error:#}"));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            bail!("Failed to remove generated Compose release tags: {}", failures.join("; "));
        }
    }
}

fn finish_with_tag_cleanup<T>(primary: Result<T>, cleanup: Result<()>) -> Result<T> {
    match (primary, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(error)) | (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => {
            Err(error.context(format!("Generated Compose release tag cleanup also failed: {cleanup_error:#}")))
        }
    }
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
