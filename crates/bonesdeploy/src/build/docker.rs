use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::docker_env::{DOCKER_CONTEXT, DOCKER_HOST, PATH};

/// The accepted local Docker endpoint for one package operation.
pub struct DockerClient {
    executable: PathBuf,
    selector: DockerSelector,
    endpoint: String,
}

impl DockerClient {
    /// Resolves the current Docker selector to a local Unix socket.
    pub fn new() -> Result<Self> {
        let executable = docker_executable()?;
        let selector = DockerSelector::capture();
        let endpoint = match selector.context.as_deref() {
            Some(context) => {
                inspect_context_endpoint(&executable, context.to_str().context("DOCKER_CONTEXT must be valid UTF-8")?)?
            }
            None => match selector.host.as_deref() {
                Some(host) => host.to_string_lossy().into_owned(),
                None => inspect_context_endpoint(&executable, &current_context(&executable)?)?,
            },
        };
        let endpoint = validate_unix_endpoint(&endpoint)?;
        Ok(Self { executable, selector, endpoint })
    }

    /// Creates a Docker command bound to the accepted endpoint.
    pub fn command(&self) -> Result<Command> {
        self.ensure_selector_is_stable()?;
        let mut command = Command::new(&self.executable);
        self.apply_environment(&mut command);
        Ok(command)
    }

    /// Applies the accepted endpoint after a caller intentionally clears a command environment.
    pub fn apply_after_env_clear(&self, command: &mut Command) -> Result<()> {
        self.ensure_selector_is_stable()?;
        self.apply_environment(command);
        Ok(())
    }

    fn ensure_selector_is_stable(&self) -> Result<()> {
        if DockerSelector::capture() != self.selector {
            bail!("Docker endpoint selector changed during this package operation")
        }
        Ok(())
    }

    fn apply_environment(&self, command: &mut Command) {
        command.env_remove(DOCKER_CONTEXT).env(DOCKER_HOST, &self.endpoint);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DockerSelector {
    host: Option<OsString>,
    context: Option<OsString>,
}

impl DockerSelector {
    fn capture() -> Self {
        Self { host: env::var_os(DOCKER_HOST), context: env::var_os(DOCKER_CONTEXT) }
    }
}

fn inspect_context_endpoint(executable: &Path, context: &str) -> Result<String> {
    let output = Command::new(executable)
        .env_remove(DOCKER_HOST)
        .env_remove(DOCKER_CONTEXT)
        .args(["context", "inspect", context, "--format", "{{.Endpoints.docker.Host}}"])
        .output()
        .context("Failed to inspect Docker context endpoint")?;
    if !output.status.success() {
        bail!("Failed to inspect Docker context `{context}`: {}", output.status)
    }
    let endpoint = String::from_utf8(output.stdout).context("Docker context endpoint was not valid UTF-8")?;
    let endpoint = endpoint.strip_suffix('\n').unwrap_or(&endpoint);
    Ok(endpoint.strip_suffix('\r').unwrap_or(endpoint).to_owned())
}

fn current_context(executable: &Path) -> Result<String> {
    let output = Command::new(executable)
        .env_remove(DOCKER_HOST)
        .env_remove(DOCKER_CONTEXT)
        .args(["context", "show"])
        .output()
        .context("Failed to determine the active Docker context")?;
    if !output.status.success() {
        bail!("Failed to determine the active Docker context: {}", output.status)
    }
    let context = String::from_utf8(output.stdout).context("Docker context name was not valid UTF-8")?;
    let context = context.trim();
    if context.is_empty() {
        bail!("Docker did not report an active context")
    }
    Ok(context.to_owned())
}

fn docker_executable() -> Result<PathBuf> {
    let path = env::var_os(PATH).context("PATH is not set; cannot locate Docker")?;
    env::split_paths(&path)
        .map(|directory| directory.join("docker"))
        .find(|candidate| candidate.is_file())
        .context("Docker executable was not found in PATH")
}

fn validate_unix_endpoint(endpoint: &str) -> Result<String> {
    let Some(path) = endpoint.strip_prefix("unix://") else {
        bail!("Docker endpoint must be a local Unix socket, got `{endpoint}`")
    };
    if path.is_empty()
        || !Path::new(path).is_absolute()
        || path.contains(['?', '#'])
        || endpoint.chars().any(|character| character.is_whitespace() || character.is_control())
    {
        bail!("Docker endpoint must be a local Unix socket with an absolute path, got `{endpoint}`")
    }
    Ok(format!("unix://{path}"))
}
