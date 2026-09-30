use std::fs;
use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::build_contract::{
    self, BUILDER_IMAGE, CACHE_MOUNT, SOURCE_MOUNT, TARGET_PLATFORM_NAME, WORKSPACE_ROOT,
};
use bonesdeploy_core::config::{Bones, build_timeout_seconds, variables};
use bonesdeploy_core::paths;
use tempfile::NamedTempFile;

use super::{command, docker::DockerClient, source::BuildContext};

const WORKLOAD_STOP_TIMEOUT_SECONDS: u64 = 15;

/// Runs numbered native build scripts against an already-exported commit.
pub fn build(config: &Bones, context: &BuildContext, docker: &DockerClient) -> Result<()> {
    let source = context.path();
    let Some((deployment_dir, scripts)) = build_scripts(source)? else {
        return Ok(());
    };

    let timeout = build_timeout_seconds(config);
    ensure_builder_image_with_client(docker, timeout)?;
    let cache = local_cache_path(&config.project_name);
    fs::create_dir_all(&cache).with_context(|| format!("Failed to create local build cache {}", cache.display()))?;
    let environment = build_contract::environment(config, source)?;
    let ownership = mount_ownership(source)?;
    let input = ContainerStart {
        source,
        deployment: &deployment_dir,
        cache: &cache,
        config,
        environment: &environment,
        ownership,
    };
    let mut container = BuildContainer::start(&input, docker)?;
    let build_result = (|| {
        for script in scripts {
            let name = script.file_name().and_then(|value| value.to_str()).unwrap_or("<unknown>");
            println!("Running local build script {name}...");
            container.run_script(&script, timeout)?;
        }
        Ok(())
    })();
    let cleanup_result = container.remove();
    finish_with_cleanup(build_result, cleanup_result)
}

/// Finds the committed deployment bundle and its numbered build scripts.
/// The supplied source must be the exported Git context, never the worktree.
pub fn build_scripts(source_context: &Path) -> Result<Option<(PathBuf, Vec<PathBuf>)>> {
    let deployment_dir = source_context.join(paths::LOCAL_INFRA_DEPLOYMENT_DIR);
    let scripts_dir = deployment_dir.join("build");
    if !scripts_dir.is_dir() {
        return Ok(None);
    }
    let scripts = build_contract::numbered_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        return Ok(None);
    }
    Ok(Some((deployment_dir, scripts)))
}

pub fn local_cache_path(site: &str) -> PathBuf {
    build_contract::cache_path(&paths::bones_cache_root().join("build"), site)
}

pub fn docker_available_linux() -> Result<()> {
    let docker = DockerClient::new()?;
    docker_available_linux_with_client(&docker, None)
}

fn docker_available_linux_with_client(docker: &DockerClient, timeout: Option<u64>) -> Result<()> {
    let output = command::output(docker_info_command(docker)?, "run docker info", timeout)?;
    if String::from_utf8_lossy(&output.stdout).trim() != "linux" {
        bail!("Docker must be reachable and configured for Linux containers for local builds");
    }
    Ok(())
}

fn docker_info_command(docker: &DockerClient) -> Result<Command> {
    let mut command = docker.command()?;
    command.arg("info").args(["--format", "{{.OSType}}"]);
    Ok(command)
}

pub fn docker_image_available() -> Result<bool> {
    let docker = DockerClient::new()?;
    docker_image_available_with_client(&docker, None)
}

fn docker_image_available_with_client(docker: &DockerClient, timeout: Option<u64>) -> Result<bool> {
    match command::output(image_inspect_command(docker)?, "inspect local builder image", timeout) {
        Ok(_) => Ok(true),
        Err(error) if command::failed(&error) => Ok(false),
        Err(error) => Err(error),
    }
}

fn image_inspect_command(docker: &DockerClient) -> Result<Command> {
    let mut command = docker.command()?;
    command.args(["image", "inspect", BUILDER_IMAGE]);
    Ok(command)
}

fn builder_image_pull_command(docker: &DockerClient) -> Result<Command> {
    let mut command = docker.command()?;
    command.args(["pull", "--platform", TARGET_PLATFORM_NAME, BUILDER_IMAGE]);
    Ok(command)
}

pub fn ensure_builder_image() -> Result<()> {
    let docker = DockerClient::new()?;
    ensure_builder_image_with_client(&docker, None)
}

fn ensure_builder_image_with_client(docker: &DockerClient, timeout: Option<u64>) -> Result<()> {
    docker_available_linux_with_client(docker, timeout)?;
    if !docker_image_available_with_client(docker, timeout)? {
        println!("Pulling local builder image...");
        command::run(builder_image_pull_command(docker)?, "pull local builder image", timeout)?;
    }
    probe_target_execution_with_client(docker, timeout)
}

fn target_probe_command(docker: &DockerClient) -> Result<Command> {
    let mut command = docker.command()?;
    command.args(["run", "--rm", "--pull=never", "--platform", TARGET_PLATFORM_NAME]).arg(BUILDER_IMAGE).arg("true");
    Ok(command)
}

pub fn probe_target_execution() -> Result<()> {
    let docker = DockerClient::new()?;
    probe_target_execution_with_client(&docker, None)
}

fn probe_target_execution_with_client(docker: &DockerClient, timeout: Option<u64>) -> Result<()> {
    command::run(target_probe_command(docker)?, "probe local builder target execution", timeout).context(
        "Pinned builder image cannot execute for target linux/amd64; install compatible Docker emulation (for example binfmt/QEMU) or use a compatible host",
    )
}

struct ContainerStart<'a> {
    source: &'a Path,
    deployment: &'a Path,
    cache: &'a Path,
    config: &'a Bones,
    environment: &'a [(String, String)],
    ownership: MountOwnership,
}

fn create_command(
    input: &ContainerStart<'_>,
    docker: &DockerClient,
    name: &str,
    environment_file: &Path,
) -> Result<Command> {
    let mut command = docker.command()?;
    command
        .args([
            "run",
            "--detach",
            "--pull=never",
            "--platform",
            TARGET_PLATFORM_NAME,
            "--security-opt=no-new-privileges",
        ])
        .args(["--workdir", SOURCE_MOUNT, "--name", name])
        .args(["--env", &format!("{}={}", variables::PROJECT_NAME, input.config.project_name)])
        .args(["--env", &format!("{}={WORKSPACE_ROOT}", variables::PROJECT_ROOT)])
        .args(["--env", &format!("{}=", variables::REPO_PATH)])
        .args(["--env", &format!("{}={}", variables::WEB_ROOT, input.config.runtime.web_root)])
        .args(["--env", &format!("{}={}", variables::SERVICE_USER, input.config.project_name)])
        .args(["--env", &format!("{}={CACHE_MOUNT}", variables::BUILD_CACHE_DIR)])
        .args(["--env-file"])
        .arg(environment_file)
        .args(["--volume"])
        .arg(format!("{}:{SOURCE_MOUNT}", input.source.display()))
        .args(["--volume"])
        .arg(format!("{}:{WORKSPACE_ROOT}/deployment:ro", input.deployment.display()))
        .args(["--volume"])
        .arg(format!("{}:{CACHE_MOUNT}:rw", input.cache.display()))
        .arg(BUILDER_IMAGE)
        .args(["sleep", "infinity"]);
    Ok(command)
}

#[derive(Clone, Copy)]
struct MountOwnership {
    uid: u32,
    gid: u32,
}

struct BuildContainer<'a> {
    docker: &'a DockerClient,
    source: PathBuf,
    cache: PathBuf,
    name: String,
    ownership: MountOwnership,
    _environment_file: NamedTempFile,
    removed: bool,
    usable: bool,
    timeout: Option<u64>,
}

impl<'a> BuildContainer<'a> {
    fn start(input: &ContainerStart<'_>, docker: &'a DockerClient) -> Result<Self> {
        let environment_file = write_environment_file(input.environment)?;
        let name = unique_container_name(&input.config.project_name);
        let mut command = create_command(input, docker, &name, environment_file.path())?;
        command.current_dir(input.source);
        if let Err(error) =
            command::run(command, &format!("start local build container {name}"), build_timeout_seconds(input.config))
        {
            let _ = force_remove_container(docker, input.source, &name, build_timeout_seconds(input.config));
            return Err(error);
        }
        Ok(Self {
            docker,
            source: input.source.to_path_buf(),
            cache: input.cache.to_path_buf(),
            name,
            ownership: input.ownership,
            timeout: build_timeout_seconds(input.config),
            _environment_file: environment_file,
            removed: false,
            usable: true,
        })
    }

    fn run_script(&mut self, script: &Path, timeout: Option<u64>) -> Result<()> {
        let script_input =
            fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;
        let mut command = self.docker.command()?;
        command
            .current_dir(&self.source)
            .args(["exec", "-i", &self.name, "bash", "-c", "umask 0002; exec bash -s"])
            .stdin(script_input);
        let result = command::run(command, &format!("execute build script {}", script.display()), timeout);
        if let Err(error) = result {
            if command::timed_out(&error) {
                self.usable = false;
                let cleanup = stop_timed_out_workload(self.docker, &self.source, &self.name);
                return finish_with_cleanup(Err(error), cleanup);
            }
            return Err(error);
        }
        Ok(())
    }

    fn remove(&mut self) -> Result<()> {
        if self.removed {
            return Ok(());
        }
        let ownership_result = self.restore_mount_ownership();
        let removal_result = force_remove_container(self.docker, &self.source, &self.name, self.timeout);
        if removal_result.is_ok() {
            self.removed = true;
        }
        finish_with_cleanup(ownership_result, removal_result)
    }

    fn restore_mount_ownership(&mut self) -> Result<()> {
        if !self.usable {
            return normalize_mount_ownership_in_cleanup_container(self, self.timeout);
        }
        match normalize_mount_ownership(self.docker, &self.name, self.ownership, self.timeout) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.usable = false;
                normalize_mount_ownership_in_cleanup_container(self, self.timeout)
                    .with_context(|| format!("Builder container ownership cleanup also failed: {error:#}"))
            }
        }
    }
}

impl Drop for BuildContainer<'_> {
    fn drop(&mut self) {
        if !self.removed {
            let _ = self.restore_mount_ownership();
            let _ = force_remove_container(self.docker, &self.source, &self.name, self.timeout);
        }
    }
}

fn write_environment_file(environment: &[(String, String)]) -> Result<NamedTempFile> {
    let mut file = tempfile::Builder::new()
        .prefix("bonesdeploy-build-env-")
        .tempfile()
        .context("Failed to create protected build environment file")?;
    fs::set_permissions(file.path(), PermissionsExt::from_mode(0o600))?;
    for (key, value) in environment {
        writeln!(file, "{key}={value}").context("Failed to write protected build environment file")?;
    }
    Ok(file)
}

fn mount_ownership(path: &Path) -> Result<MountOwnership> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect local build mount {}", path.display()))?;
    Ok(MountOwnership { uid: metadata.uid(), gid: metadata.gid() })
}

fn normalize_mount_ownership(
    docker: &DockerClient,
    name: &str,
    ownership: MountOwnership,
    timeout: Option<u64>,
) -> Result<()> {
    let ownership = format!("{}:{}", ownership.uid, ownership.gid);
    let mut command = docker.command()?;
    command.args(["exec", name]).args([
        "find",
        "-P",
        SOURCE_MOUNT,
        CACHE_MOUNT,
        "-exec",
        "chown",
        "-h",
        &ownership,
        "{}",
        "+",
    ]);
    command::run(command, "restore local build mount ownership", timeout)
}

fn normalize_mount_ownership_in_cleanup_container(container: &BuildContainer<'_>, timeout: Option<u64>) -> Result<()> {
    let ownership = format!("{}:{}", container.ownership.uid, container.ownership.gid);
    let mut command = container.docker.command()?;
    command
        .current_dir(&container.source)
        .args(["run", "--rm", "--pull=never", "--platform", TARGET_PLATFORM_NAME])
        .args(["--volume"])
        .arg(format!("{}:{SOURCE_MOUNT}", container.source.display()))
        .args(["--volume"])
        .arg(format!("{}:{CACHE_MOUNT}:rw", container.cache.display()))
        .arg(BUILDER_IMAGE)
        .args(["find", "-P", SOURCE_MOUNT, CACHE_MOUNT, "-exec", "chown", "-h", &ownership, "{}", "+"]);
    command::run(command, "restore local build mount ownership in cleanup container", timeout)
}

fn stop_timed_out_workload(docker: &DockerClient, source: &Path, name: &str) -> Result<()> {
    let mut command = docker.command()?;
    command.current_dir(source).args(["stop", "--time", "10", name]);
    command::run(command, "stop timed out local build container workload", Some(WORKLOAD_STOP_TIMEOUT_SECONDS))
}

fn force_remove_container(docker: &DockerClient, source: &Path, name: &str, timeout: Option<u64>) -> Result<()> {
    let mut command = docker.command()?;
    command.current_dir(source).args(["rm", "--force", name]);
    command::run(command, &format!("remove local build container {name}"), timeout)
}

fn unique_container_name(project: &str) -> String {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("bonesdeploy-build-{project}-{}-{nonce}", process::id())
}

fn finish_with_cleanup(primary: Result<()>, cleanup: Result<()>) -> Result<()> {
    match (primary, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) | (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => {
            Err(error.context(format!("Local build cleanup also failed: {cleanup_error:#}")))
        }
    }
}
