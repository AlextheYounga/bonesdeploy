use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::build_contract::{
    self, BUILDER_IMAGE, CACHE_MOUNT, SOURCE_MOUNT, TARGET_PLATFORM_NAME, WORKSPACE_ROOT,
};
use bonesdeploy_core::config::{Bones, build_timeout_seconds, variables};
use bonesdeploy_core::paths;
use tempfile::NamedTempFile;

use super::source::BuildContext;

/// Runs numbered native build scripts against an already-exported commit.
pub fn build(config: &Bones, context: &BuildContext) -> Result<()> {
    let source = context.path();

    let Some((deployment_dir, scripts)) = build_scripts(source)? else {
        return Ok(());
    };

    ensure_builder_image()?;
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
    let mut container = BuildContainer::start(&input)?;
    let build_result = (|| {
        for script in scripts {
            let name = script.file_name().and_then(|value| value.to_str()).unwrap_or("<unknown>");
            println!("Running local build script {name}...");
            container.run_script(&script, build_timeout_seconds(config))?;
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
    let output = docker_info_command().output().context("Failed to run docker info")?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != "linux" {
        bail!("Docker must be reachable and configured for Linux containers for local builds");
    }
    Ok(())
}

fn docker_info_command() -> Command {
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

fn builder_image_pull_command() -> Command {
    let mut command = Command::new("docker");
    command.args(["pull", "--platform", TARGET_PLATFORM_NAME, BUILDER_IMAGE]);
    command
}

pub fn ensure_builder_image() -> Result<()> {
    docker_available_linux()?;
    if !docker_image_available()? {
        println!("Pulling local builder image...");
        let status = builder_image_pull_command().status().context("Failed to pull local builder image")?;
        if !status.success() {
            bail!("Failed to pull local builder image {BUILDER_IMAGE}");
        }
    }
    probe_target_execution()
}

fn target_probe_command() -> Command {
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

struct ContainerStart<'a> {
    source: &'a Path,
    deployment: &'a Path,
    cache: &'a Path,
    config: &'a Bones,
    environment: &'a [(String, String)],
    ownership: MountOwnership,
}

fn create_command(input: &ContainerStart<'_>, name: &str, environment_file: &Path) -> Command {
    let mut command = Command::new("docker");
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
    command
}

#[derive(Clone, Copy)]
struct MountOwnership {
    uid: u32,
    gid: u32,
}

struct BuildContainer {
    source: PathBuf,
    name: String,
    ownership: MountOwnership,
    _environment_file: NamedTempFile,
    removed: bool,
}

impl BuildContainer {
    fn start(input: &ContainerStart<'_>) -> Result<Self> {
        let environment_file = write_environment_file(input.environment)?;
        let name = unique_container_name(&input.config.project_name);
        let status = create_command(input, &name, environment_file.path())
            .current_dir(input.source)
            .stdout(Stdio::null())
            .status()
            .with_context(|| format!("Failed to start local build container {name}"))?;
        if !status.success() {
            let _ = force_remove_container(input.source, &name);
            bail!("Failed to start local build container {name}: {status}");
        }
        Ok(Self {
            source: input.source.to_path_buf(),
            name,
            ownership: input.ownership,
            _environment_file: environment_file,
            removed: false,
        })
    }

    fn run_script(&self, script: &Path, timeout: Option<u64>) -> Result<()> {
        let script_input =
            fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;
        let mut child = Command::new("docker")
            .current_dir(&self.source)
            .args(["exec", "-i", &self.name, "bash", "-c", "umask 0002; exec bash -s"])
            .stdin(Stdio::from(script_input))
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

    fn remove(&mut self) -> Result<()> {
        if self.removed {
            return Ok(());
        }
        let ownership_result = normalize_mount_ownership(&self.name, self.ownership);
        let removal_result = force_remove_container(&self.source, &self.name);
        self.removed = true;
        finish_with_cleanup(ownership_result, removal_result)
    }
}

impl Drop for BuildContainer {
    fn drop(&mut self) {
        if !self.removed {
            let _ = normalize_mount_ownership(&self.name, self.ownership);
            let _ = force_remove_container(&self.source, &self.name);
            self.removed = true;
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

fn normalize_mount_ownership(name: &str, ownership: MountOwnership) -> Result<()> {
    let ownership = format!("{}:{}", ownership.uid, ownership.gid);
    let status = Command::new("docker")
        .args(["exec", name])
        .args(["find", "-P", SOURCE_MOUNT, CACHE_MOUNT, "-exec", "chown", "-h", &ownership, "{}", "+"])
        .status()
        .context("Failed to restore local build mount ownership")?;
    if !status.success() {
        bail!("Failed to restore local build mount ownership: {status}");
    }
    Ok(())
}

fn force_remove_container(source: &Path, name: &str) -> Result<()> {
    let status = Command::new("docker")
        .current_dir(source)
        .args(["rm", "--force", name])
        .status()
        .with_context(|| format!("Failed to remove local build container {name}"))?;
    if !status.success() {
        bail!("Failed to remove local build container {name}: {status}");
    }
    Ok(())
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
            Err(error.context(format!("Local build mount ownership cleanup also failed: {cleanup_error:#}")))
        }
    }
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
