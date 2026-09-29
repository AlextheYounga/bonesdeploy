use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::build_contract::{
    self, BUILDER_IMAGE, CACHE_MOUNT, SOURCE_MOUNT, TARGET_PLATFORM_NAME, WORKSPACE_ROOT,
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

/// Exports the one committed source tree used by every deployment backend.
pub fn export(config: &Bones) -> Result<BuildContext> {
    let context =
        tempfile::Builder::new().prefix("bonesdeploy-build-").tempdir().context("Failed to create build context")?;
    let repo = env::current_dir().context("Failed to determine project directory")?;
    let revision = git::resolve_branch_commit(&repo, &config.branch)?;
    git::export_commit(&repo, &revision, context.path())?;
    sanitize_exported_context(context.path())?;

    Ok(BuildContext { context, revision })
}

pub fn build(config: &Bones) -> Result<BuildContext> {
    let context = export(config)?;
    build_native(config, &context)?;
    Ok(context)
}

/// Runs numbered native build scripts against an already-exported commit.
pub fn build_native(config: &Bones, context: &BuildContext) -> Result<()> {
    let source = context.path();

    let Some((_deployment_dir, scripts)) = build_scripts(source)? else {
        return Ok(());
    };

    let cache = local_cache_path(&config.project_name);
    fs::create_dir_all(&cache).with_context(|| format!("Failed to create local build cache {}", cache.display()))?;
    let environment = build_contract::environment(config, source)?;
    let user = mount_user(source)?;
    let input = ContainerStart { source, cache: &cache, config, environment: &environment, user };
    for script in scripts {
        let name = script.file_name().and_then(|value| value.to_str()).unwrap_or("<unknown>");
        println!("Running local build script {name}...");
        run_script(&input, &script, build_timeout_seconds(config))?;
    }
    Ok(())
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
    cache: &'a Path,
    config: &'a Bones,
    environment: &'a [(String, String)],
    user: MountUser,
}

pub fn create_command(input: &ContainerStart<'_>, environment_file: &Path) -> Command {
    let mut command = Command::new("docker");
    command
        .args(["run", "--rm", "--pull=never", "--platform", TARGET_PLATFORM_NAME, "--security-opt=no-new-privileges"])
        .args(["--user", &format!("{}:{}", input.user.uid, input.user.gid)])
        .args(["--workdir", SOURCE_MOUNT])
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
        .args(["bash", "-s"]);
    command
}

#[derive(Clone, Copy)]
struct MountUser {
    uid: u32,
    gid: u32,
}

fn run_script(input: &ContainerStart<'_>, script: &Path, timeout: Option<u64>) -> Result<()> {
    let script_input =
        fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;
    let environment_file = write_environment_file(input.environment)?;
    let mut child = create_command(input, environment_file.path())
        .current_dir(input.source)
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

fn mount_user(path: &Path) -> Result<MountUser> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect local build mount {}", path.display()))?;
    Ok(MountUser { uid: metadata.uid(), gid: metadata.gid() })
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
