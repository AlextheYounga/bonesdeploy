use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::validate_project_name;
use bonesdeploy_core::paths;
use serde::{Deserialize, Serialize};

const BASE_FILES: [&str; 4] = ["compose.yaml", "compose.yml", "docker-compose.yaml", "docker-compose.yml"];
const OVERRIDE_FILES: [&str; 2] = ["compose.override.yaml", "compose.override.yml"];
const COMPOSE_CONTROL_ENVIRONMENT: [&str; 16] = [
    "COMPOSE_ANSI",
    "COMPOSE_BAKE",
    "COMPOSE_CONVERT_WINDOWS_PATHS",
    "COMPOSE_DISABLE_ENV_FILE",
    "COMPOSE_ENV_FILES",
    "COMPOSE_EXPERIMENTAL",
    "COMPOSE_FILE",
    "COMPOSE_IGNORE_ORPHANS",
    "COMPOSE_MENU",
    "COMPOSE_PARALLEL_LIMIT",
    "COMPOSE_PATH_SEPARATOR",
    "COMPOSE_PROFILES",
    "COMPOSE_PROJECT_NAME",
    "COMPOSE_PROGRESS",
    "COMPOSE_REMOVE_ORPHANS",
    "COMPOSE_STATUS_STDOUT",
];
const DOCKER_CONTROL_ENVIRONMENT: [&str; 5] =
    ["DOCKER_CERT_PATH", "DOCKER_CONTEXT", "DOCKER_HOST", "DOCKER_TLS", "DOCKER_TLS_VERIFY"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComposeStackStatus {
    pub project_name: String,
    pub files: Vec<String>,
    pub services: Vec<ComposeServiceStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComposeServiceStatus {
    pub name: String,
    pub service: String,
    pub state: String,
    pub health: Option<String>,
    pub exit_code: i64,
    pub publishers: Vec<ComposePublisher>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComposePublisher {
    pub host_ip: String,
    pub target_port: u16,
    pub published_port: u16,
    pub protocol: String,
}

#[derive(Deserialize)]
struct RawComposeServiceStatus {
    #[serde(rename = "Name", default)]
    name: String,
    #[serde(rename = "Service", default)]
    service: String,
    #[serde(rename = "State", default)]
    state: String,
    #[serde(rename = "Health", default)]
    health: String,
    #[serde(rename = "ExitCode", default)]
    exit_code: i64,
    #[serde(rename = "Publishers", default)]
    publishers: Option<Vec<RawComposePublisher>>,
}

#[derive(Deserialize)]
struct RawComposePublisher {
    #[serde(rename = "URL", default)]
    host_ip: String,
    #[serde(rename = "TargetPort", default)]
    target_port: u16,
    #[serde(rename = "PublishedPort", default)]
    published_port: u16,
    #[serde(rename = "Protocol", default)]
    protocol: String,
}

impl ComposeServiceStatus {
    #[must_use]
    pub fn condition(&self) -> &'static str {
        match (self.state.as_str(), self.health.as_deref(), self.exit_code) {
            ("running", Some("healthy"), _) => "healthy",
            ("running", None, _) => "running",
            ("running", Some("unhealthy"), _) => "unhealthy",
            ("running", Some(_), _) => "starting",
            ("exited", _, 0) => "completed",
            _ => "failed",
        }
    }

    #[must_use]
    pub fn publishes_loopback_port(&self, port: u16) -> bool {
        self.publishers.iter().any(|publisher| publisher.published_port == port && publisher.host_ip == "127.0.0.1")
    }

    #[must_use]
    pub fn publishes_public_port(&self) -> bool {
        self.publishers.iter().any(|publisher| {
            publisher.published_port != 0 && !matches!(publisher.host_ip.as_str(), "127.0.0.1" | "::1")
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposeFiles {
    files: Vec<PathBuf>,
}

impl ComposeFiles {
    pub fn discover(release_root: &Path) -> Result<Self> {
        let base = existing_files(release_root, &BASE_FILES)?;
        let override_files = existing_files(release_root, &OVERRIDE_FILES)?;

        let base_file = match base.as_slice() {
            [] => bail!("No Compose file found in {}", release_root.display()),
            [file] => file,
            _ => bail!("Multiple Compose base files found in {}: {}", release_root.display(), display_files(&base)),
        };
        if override_files.len() > 1 {
            bail!(
                "Multiple Compose override files found in {}: {}",
                release_root.display(),
                display_files(&override_files)
            );
        }

        let mut files = vec![base_file.clone()];
        files.extend(override_files);
        Ok(Self { files })
    }

    #[must_use]
    pub fn paths(&self) -> &[PathBuf] {
        &self.files
    }

    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.files.iter().filter_map(|path| path.file_name()).map(|name| name.to_string_lossy().into_owned()).collect()
    }
}

pub struct ComposeCommand<'a> {
    pub project_directory: &'a Path,
    pub env_file: &'a Path,
    pub files: &'a ComposeFiles,
}

pub fn project_name(site: &str) -> Result<String> {
    validate_project_name(site)?;
    Ok(format!("bonesdeploy-{site}"))
}

pub fn prepare_candidate(site: &str, project_root: &Path, context: &Path) -> Result<()> {
    prepare_candidate_with(site, project_root, context, &mut run)
}

fn prepare_candidate_with(
    site: &str,
    project_root: &Path,
    context: &Path,
    execute: &mut impl FnMut(&mut Command, &str) -> Result<()>,
) -> Result<()> {
    let files = ComposeFiles::discover(context)?;
    let env_file = project_root.join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let compose = ComposeCommand { project_directory: context, env_file: &env_file, files: &files };

    execute(&mut compose_command(site, &compose, ["config", "--quiet"])?, "validate Compose configuration")?;
    execute(&mut compose_command(site, &compose, ["pull"])?, "pull Compose images")?;
    execute(&mut compose_command(site, &compose, ["build"])?, "build Compose images")
}

pub fn active_start(site: &str, project_root: &Path, wait_timeout: u64) -> Result<()> {
    active_start_with(site, project_root, wait_timeout, &mut run)
}

fn active_start_with(
    site: &str,
    project_root: &Path,
    wait_timeout: u64,
    execute: &mut impl FnMut(&mut Command, &str) -> Result<()>,
) -> Result<()> {
    let current = project_root.join(paths::CURRENT_LINK);
    let files = ComposeFiles::discover(&current)?;
    let env_file = project_root.join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let timeout = wait_timeout.to_string();
    let compose = ComposeCommand { project_directory: &current, env_file: &env_file, files: &files };
    execute(
        &mut compose_command(
            site,
            &compose,
            ["up", "--detach", "--build", "--remove-orphans", "--wait", "--wait-timeout", &timeout],
        )?,
        "start Compose stack",
    )
}

pub fn validate_active_configuration(site: &str, project_root: &Path) -> Result<()> {
    let current = project_root.join(paths::CURRENT_LINK);
    let files = ComposeFiles::discover(&current)?;
    let env_file = project_root.join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let compose = ComposeCommand { project_directory: &current, env_file: &env_file, files: &files };
    run(&mut compose_command(site, &compose, ["config", "--quiet"])?, "validate active Compose configuration")
}

pub fn active_status(site: &str, project_root: &Path) -> Result<ComposeStackStatus> {
    let current = project_root.join(paths::CURRENT_LINK);
    let files = ComposeFiles::discover(&current)?;
    let env_file = project_root.join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let compose = ComposeCommand { project_directory: &current, env_file: &env_file, files: &files };
    let output = compose_command(site, &compose, ["ps", "--all", "--format", "json"])?
        .output()
        .context("Failed to inspect Compose stack")?;
    if !output.status.success() {
        bail!("Failed to inspect Compose stack: {}", output.status);
    }

    Ok(ComposeStackStatus {
        project_name: project_name(site)?,
        files: files.names(),
        services: parse_ps_output(&output.stdout)?,
    })
}

pub fn parse_ps_output(output: &[u8]) -> Result<Vec<ComposeServiceStatus>> {
    let raw = serde_json::from_slice::<Vec<RawComposeServiceStatus>>(output)
        .or_else(|_| {
            output
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
                .map(serde_json::from_slice)
                .collect()
        })
        .context("Docker Compose returned invalid service status JSON")?;

    Ok(raw.into_iter().map(ComposeServiceStatus::from).collect())
}

pub fn active_stop(site: &str, project_root: &Path) -> Result<()> {
    active_stop_with(site, project_root, &mut run)
}

fn active_stop_with(
    site: &str,
    project_root: &Path,
    execute: &mut impl FnMut(&mut Command, &str) -> Result<()>,
) -> Result<()> {
    let current = project_root.join(paths::CURRENT_LINK);
    let files = ComposeFiles::discover(&current)?;
    let env_file = project_root.join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let compose = ComposeCommand { project_directory: &current, env_file: &env_file, files: &files };
    execute(&mut compose_command(site, &compose, ["stop"])?, "stop Compose stack")
}

pub fn compose_command<I, S>(site: &str, compose: &ComposeCommand<'_>, arguments: I) -> Result<Command>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("docker");
    command.args([OsStr::new("compose"), OsStr::new("--project-name")]);
    command.arg(project_name(site)?);
    command.args([
        OsStr::new("--project-directory"),
        compose.project_directory.as_os_str(),
        OsStr::new("--env-file"),
        compose.env_file.as_os_str(),
    ]);
    for file in compose.files.paths() {
        command.args([OsStr::new("--file"), file.as_os_str()]);
    }
    command.args(arguments).current_dir(compose.project_directory);
    for variable in COMPOSE_CONTROL_ENVIRONMENT {
        command.env_remove(variable);
    }
    for variable in DOCKER_CONTROL_ENVIRONMENT {
        command.env_remove(variable);
    }
    Ok(command)
}

impl From<RawComposeServiceStatus> for ComposeServiceStatus {
    fn from(raw: RawComposeServiceStatus) -> Self {
        Self {
            name: raw.name,
            service: raw.service,
            state: raw.state.to_ascii_lowercase(),
            health: (!raw.health.is_empty()).then(|| raw.health.to_ascii_lowercase()),
            exit_code: raw.exit_code,
            publishers: raw
                .publishers
                .unwrap_or_default()
                .into_iter()
                .map(|publisher| ComposePublisher {
                    host_ip: publisher.host_ip,
                    target_port: publisher.target_port,
                    published_port: publisher.published_port,
                    protocol: publisher.protocol,
                })
                .collect(),
        }
    }
}

fn run(command: &mut Command, action: &str) -> Result<()> {
    let status = command.status().with_context(|| format!("Failed to {action}"))?;
    if status.success() { Ok(()) } else { bail!("Failed to {action}: {status}") }
}

fn existing_files(release_root: &Path, candidates: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for candidate in candidates {
        let path = release_root.join(candidate);
        match fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => files.push(path),
            Ok(_) => bail!("Compose file is not a regular file: {}", path.display()),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("Failed to inspect Compose file {}", path.display()));
            }
        }
    }
    Ok(files)
}

fn display_files(files: &[PathBuf]) -> String {
    files.iter().map(|file| file.display().to_string()).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
