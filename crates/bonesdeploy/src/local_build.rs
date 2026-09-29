use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::build_contract::{
    self, BUILDER_IMAGE, CACHE_MOUNT, SOURCE_MOUNT, TARGET_PLATFORM_NAME, WORKSPACE_ROOT,
    deployment_tar_extract_command,
};
use bonesdeploy_core::config::{Bones, build_timeout_seconds, variables};
use bonesdeploy_core::paths;
use tempfile::{NamedTempFile, TempDir};

use crate::infra::git;

#[cfg(test)]
#[path = "local_build/tests.rs"]
mod tests;

pub struct BuildContext {
    context: TempDir,
    pub revision: String,
}

impl BuildContext {
    pub fn path(&self) -> &Path {
        self.context.path()
    }

    #[cfg(test)]
    pub(crate) fn from_tempdir(context: TempDir, revision: String) -> Self {
        Self { context, revision }
    }
}

pub fn build(config: &Bones) -> Result<BuildContext> {
    let context =
        tempfile::Builder::new().prefix("bonesdeploy-build-").tempdir().context("Failed to create build context")?;
    let repo = env::current_dir().context("Failed to determine project directory")?;
    let revision = git::resolve_branch_commit(&repo, &config.branch)?;
    git::export_commit(&repo, &revision, context.path())?;
    sanitize_exported_context(context.path())?;

    let Some((deployment_dir, scripts)) = build_scripts(context.path())? else {
        return Ok(BuildContext { context, revision });
    };

    let cache = local_cache_path(&config.project_name);
    fs::create_dir_all(&cache).with_context(|| format!("Failed to create local build cache {}", cache.display()))?;
    let environment = build_contract::environment(config, context.path())?;
    let input = ContainerStart {
        source: context.path(),
        deployment: &deployment_dir,
        cache: &cache,
        config,
        environment: &environment,
    };
    let mut container = DockerContainer::start(&input)?;
    let build_result = (|| {
        for script in scripts {
            let name = script.file_name().and_then(|value| value.to_str()).unwrap_or("<unknown>");
            println!("Running local build script {name}...");
            container.run_script(&script, build_timeout_seconds(config))?;
        }
        Ok(())
    })();
    let cleanup_result = container.remove();
    finish_with_cleanup(build_result, cleanup_result)?;
    Ok(BuildContext { context, revision })
}

/// Removes the committed runtime environment before anything can read the
/// exported context. Build configuration remains available through `.env.build`.
pub fn sanitize_exported_context(source_context: &Path) -> Result<()> {
    let root_env = source_context.join(paths::DOT_ENV);
    match fs::symlink_metadata(&root_env) {
        Ok(metadata) if metadata.file_type().is_file() || metadata.file_type().is_symlink() => {
            fs::remove_file(&root_env).with_context(|| format!("Failed to remove exported {}", paths::DOT_ENV))?;
        }
        Ok(metadata) => {
            bail!("Exported {} has unsupported file type: {:?}", paths::DOT_ENV, metadata.file_type());
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(|| format!("Failed to inspect exported {}", paths::DOT_ENV)),
    }
    Ok(())
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
    let output = docker_info_command().output().context("Failed to run docker info")?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "linux" {
        bail!("Docker must be reachable and configured for Linux containers for local builds");
    }
    Ok(())
}

pub fn docker_info_command() -> Command {
    let mut command = Command::new("docker");
    command.arg("info").args(["--format", "{{.OSType}}"]);
    command
}

pub fn docker_image_available() -> Result<bool> {
    let status = Command::new("docker")
        .args(["image", "inspect", BUILDER_IMAGE])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to inspect local builder image")?;
    Ok(status.success())
}

pub fn target_probe_command() -> Command {
    let mut command = Command::new("docker");
    command.args(["run", "--rm", "--pull=never", "--platform", TARGET_PLATFORM_NAME]).arg(BUILDER_IMAGE).arg("true");
    command
}

pub fn probe_target_execution() -> Result<()> {
    let status = target_probe_command().status().context("Failed to probe local builder target execution")?;
    if !status.success() {
        bail!(
            "Pinned builder image cannot execute for target {TARGET_PLATFORM_NAME}; install compatible Docker emulation (for example binfmt/QEMU) or use a compatible host"
        );
    }
    Ok(())
}

pub struct ContainerStart<'a> {
    source: &'a Path,
    deployment: &'a Path,
    cache: &'a Path,
    config: &'a Bones,
    environment: &'a [(String, String)],
}

pub fn create_command(input: &ContainerStart<'_>, name: &str, environment_file: &Path) -> Command {
    let mut command = Command::new("docker");
    command
        .arg("run")
        .args(["-d", "--pull=never", "--platform", TARGET_PLATFORM_NAME, "--security-opt=no-new-privileges"])
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
        .arg(format!("{}:{CACHE_MOUNT}:rw", input.cache.display()))
        .arg(BUILDER_IMAGE)
        .args(["sleep", "infinity"]);
    command
}

struct DockerContainer {
    source: PathBuf,
    source_ownership: MountOwnership,
    name: String,
    _environment_file: NamedTempFile,
    removed: bool,
}

impl DockerContainer {
    fn start(input: &ContainerStart<'_>) -> Result<Self> {
        let source_ownership = mount_ownership(input.source)?;
        // Use the fresh source identity to repair an already-contaminated cache too.
        docker_available_linux()?;
        let pull_status = Command::new("docker")
            .args(["pull", "--platform", TARGET_PLATFORM_NAME, BUILDER_IMAGE])
            .status()
            .context("Failed to pull local builder image")?;
        if !pull_status.success() {
            bail!("Failed to pull local builder image {BUILDER_IMAGE}");
        }
        probe_target_execution()?;
        let environment_file = write_environment_file(input.environment)?;
        let name = unique_container_name(&input.config.project_name);
        let status = create_command(input, &name, environment_file.path())
            .status()
            .with_context(|| format!("Failed to start local build container {name}"))?;
        if !status.success() {
            let _ = force_remove_container(input.source, &name);
            bail!("Failed to start local build container {name}: {status}");
        }
        let mut container = Self {
            source: input.source.to_path_buf(),
            source_ownership,
            name,
            _environment_file: environment_file,
            removed: false,
        };
        match container.copy_deployment_tree(input.deployment) {
            Ok(()) => Ok(container),
            Err(error) => finish_with_cleanup(Err(error), container.remove()).map(|()| container),
        }
    }

    fn run_script(&self, script: &Path, timeout: Option<u64>) -> Result<()> {
        let input =
            fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;
        let mut child = Command::new("docker")
            .current_dir(&self.source)
            .args(["exec", "-i", &self.name, "bash", "-c", "umask 0002; exec bash -s"])
            .stdin(Stdio::from(input))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute build script {}", script.display()))?;
        let stdout = child.stdout.take().context("Docker stdout was not piped")?;
        let stderr = child.stderr.take().context("Docker stderr was not piped")?;
        let stdout_thread = thread::spawn(move || stream_output(stdout, false));
        let stderr_thread = thread::spawn(move || stream_output(stderr, true));
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait().context("Failed to wait for local build script")? {
                finish_output(stdout_thread, stderr_thread)?;
                if status.success() {
                    return Ok(());
                }
                bail!("Build script {} exited with status {status}", script.display());
            }
            if let Some(seconds) = timeout.filter(|seconds| started.elapsed() >= Duration::from_secs(*seconds)) {
                child.kill().context("Failed to stop timed out local build script")?;
                let _ = child.wait();
                finish_output(stdout_thread, stderr_thread)?;
                bail!("Build script {} exceeded its {seconds}-second timeout", script.display());
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn copy_deployment_tree(&self, deployment: &Path) -> Result<()> {
        let mut archive = Command::new("tar")
            .current_dir(deployment)
            .args(["--create", "--file=-", "."])
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to archive deployment files from {}", deployment.display()))?;
        let archive_stdout = archive.stdout.take().context("Deployment archive stdout was not piped")?;
        let extract_status = Command::new("docker")
            .current_dir(&self.source)
            .args(["exec", "-i", &self.name, "sh", "-c", &deployment_tar_extract_command()])
            .stdin(Stdio::from(archive_stdout))
            .status()
            .with_context(|| format!("Failed to copy deployment files into local build container {}", self.name))?;
        let archive_status = archive.wait().context("Failed to finish deployment archive")?;
        if !archive_status.success() || !extract_status.success() {
            bail!("Failed to copy deployment files into local build container {}", self.name);
        }
        Ok(())
    }

    fn remove(&mut self) -> Result<()> {
        if self.removed {
            return Ok(());
        }
        remove_container(&self.name, &self.source, self.source_ownership)?;
        self.removed = true;
        Ok(())
    }
}

impl Drop for DockerContainer {
    fn drop(&mut self) {
        if !self.removed {
            let _ = remove_container(&self.name, &self.source, self.source_ownership);
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

#[derive(Clone, Copy)]
struct MountOwnership {
    uid: u32,
    gid: u32,
}

fn mount_ownership(path: &Path) -> Result<MountOwnership> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect local build mount {}", path.display()))?;
    Ok(MountOwnership { uid: metadata.uid(), gid: metadata.gid() })
}

fn normalize_mount_ownership_command(name: &str, ownership: MountOwnership) -> Command {
    let mut command = Command::new("docker");
    let ownership = format!("{}:{}", ownership.uid, ownership.gid);
    command
        .args(["exec", "--user", "0:0", name, "sh", "-c"])
        .arg(format!("find -P {SOURCE_MOUNT} {CACHE_MOUNT} -exec chown -h {ownership} {{}} +"));
    command
}

fn remove_container(name: &str, source: &Path, ownership: MountOwnership) -> Result<()> {
    let ownership_result = normalize_mount_ownership_command(name, ownership)
        .status()
        .with_context(|| format!("Failed to restore local build mount ownership in container {name}"))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                bail!("Failed to restore local build mount ownership in container {name}: {status}")
            }
        });
    let removal_result = force_remove_container(source, name);
    if let Err(error) = ownership_result {
        return Err(error);
    }
    removal_result
}

fn force_remove_container(source: &Path, name: &str) -> Result<()> {
    let status = force_remove_command(name)
        .current_dir(source)
        .status()
        .with_context(|| format!("Failed to remove local build container {name}"))?;
    if !status.success() {
        bail!("Failed to remove local build container {name}: {status}");
    }
    Ok(())
}

fn force_remove_command(name: &str) -> Command {
    let mut command = Command::new("docker");
    command.args(["rm", "--force", "--time", "0", "--ignore", name]);
    command
}

fn finish_with_cleanup(primary: Result<()>, cleanup: Result<()>) -> Result<()> {
    match (primary, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(error.context("Failed to clean up local build container")),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => {
            Err(error.context(format!("Local build container cleanup also failed: {cleanup_error:#}")))
        }
    }
}

fn unique_container_name(project: &str) -> String {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("bonesdeploy-build-{project}-{}-{nonce}", process::id())
}

fn stream_output<R: io::Read>(reader: R, stderr: bool) -> Result<()> {
    for line in BufReader::new(reader).lines() {
        let line = line.context("Failed to read local build output")?;
        if stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
    Ok(())
}

fn finish_output(stdout: thread::JoinHandle<Result<()>>, stderr: thread::JoinHandle<Result<()>>) -> Result<()> {
    stdout.join().map_err(|_| anyhow::anyhow!("Local build stdout reader panicked"))??;
    stderr.join().map_err(|_| anyhow::anyhow!("Local build stderr reader panicked"))??;
    Ok(())
}
