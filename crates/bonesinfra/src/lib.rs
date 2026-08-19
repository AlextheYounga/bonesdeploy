//! Embedded BonesInfra distribution materialized into project infrastructure.

use std::borrow::Cow;
use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "python/"]
#[exclude = ".venv/**"]
#[exclude = "**/__pycache__/**"]
#[exclude = "**/*.egg-info/**"]
#[exclude = "docs/**"]
#[exclude = "tests/**"]
struct PythonSource;

const FRAMEWORK_PATH: &str = "infra/.framework";
const STAMP_FILE: &str = ".stamp";

/// Materializes the complete embedded BonesInfra distribution into a project.
///
/// # Errors
/// Fails when the managed framework cannot be atomically replaced.
pub fn materialize_project_framework(project_root: &Path) -> Result<PathBuf> {
    let framework = project_root.join(FRAMEWORK_PATH);
    let infra = framework.parent().context("BonesInfra framework has no parent directory")?;
    fs::create_dir_all(infra).with_context(|| format!("Failed to create {}", infra.display()))?;

    let staging = tempfile::Builder::new()
        .prefix(".core.")
        .tempdir_in(infra)
        .with_context(|| format!("Failed to create managed framework staging directory in {}", infra.display()))?;
    write_embedded_source(staging.path())?;

    let previous = infra.join(".framework.previous");
    remove_path(&previous)?;
    if framework.exists() || framework.is_symlink() {
        fs::rename(&framework, &previous)
            .with_context(|| format!("Failed to stage existing managed framework at {}", framework.display()))?;
    }
    if let Err(error) = fs::rename(staging.path(), &framework) {
        if previous.exists() {
            let _ = fs::rename(&previous, &framework);
        }
        return Err(error).with_context(|| format!("Failed to replace managed framework at {}", framework.display()));
    }
    remove_path(&previous)?;
    Ok(framework)
}

/// Runs `python -m bonesinfra` from the current project's managed framework.
///
/// # Errors
/// Fails when the project core or dependency environment cannot be prepared or
/// when the command exits non-zero.
pub fn run(args: &[&str]) -> Result<()> {
    let project_root = env::current_dir().context("Failed to determine project directory")?;
    let executable = ensure_available(&project_root)?;
    let mut command = base_command(&executable, &project_root, args);
    command.stdin(Stdio::null());

    let status = command
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {}", args.join(" ")))?
        .wait()
        .with_context(|| format!("Failed to wait on bonesinfra {}", args.join(" ")))?;
    if !status.success() {
        bail!("bonesinfra {} failed", args.join(" "));
    }
    Ok(())
}

/// Prepares the dependency environment for a project. This is intended for
/// command-test setup and does not materialize source.
///
/// # Errors
/// Fails when the project-local core is missing or dependencies cannot install.
pub fn prepare_in(project_root: &Path) -> Result<()> {
    ensure_available(project_root).map(|_| ())
}

/// Returns the paths packaged in the embedded Python distribution.
pub fn embedded_source_paths() -> impl Iterator<Item = Cow<'static, str>> {
    PythonSource::iter()
}

fn ensure_available(project_root: &Path) -> Result<PathBuf> {
    let project_root = project_root
        .canonicalize()
        .with_context(|| format!("Failed to resolve project root {}", project_root.display()))?;
    let framework = project_root.join(FRAMEWORK_PATH);
    if !framework.join("pyproject.toml").is_file() || !framework.join("src/bonesinfra/__main__.py").is_file() {
        bail!(
            "Project-local BonesInfra framework is missing at {}. Run bonesdeploy init or update.",
            framework.display()
        );
    }

    let environment = environment_dir(&project_root);
    fs::create_dir_all(&environment).with_context(|| format!("Failed to create {}", environment.display()))?;
    let lock_path = environment.join(".lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("Failed to open BonesInfra lock at {}", lock_path.display()))?;
    lock.lock().with_context(|| format!("Failed to lock BonesInfra environment at {}", lock_path.display()))?;

    let python = environment.join(".venv/bin/python");
    let stamp = package_version(&framework)?;
    if python.is_file() && materialized_stamp(&environment).as_deref() == Some(stamp.as_str()) {
        return Ok(python);
    }

    setup_venv(&environment, &framework)?;
    if !python.is_file() {
        bail!("BonesInfra setup finished at {}, but {} is missing.", environment.display(), python.display());
    }
    fs::write(environment.join(STAMP_FILE), stamp)
        .with_context(|| format!("Failed to write BonesInfra stamp in {}", environment.display()))?;
    Ok(python)
}

fn base_command(executable: &Path, project_root: &Path, args: &[&str]) -> Command {
    let framework = project_root.join(FRAMEWORK_PATH);
    let mut command = Command::new(executable);
    command.current_dir(project_root).env("PYTHONPATH", framework.join("src"));
    command.args(["-m", "bonesinfra"]);
    command.args(args);
    command
}

fn write_embedded_source(destination: &Path) -> Result<()> {
    for file_path in embedded_source_paths() {
        let Some(asset) = PythonSource::get(&file_path) else {
            continue;
        };
        let target = destination.join(file_path.as_ref());
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        fs::write(&target, asset.data.as_ref()).with_context(|| format!("Failed to write {}", target.display()))?;
    }
    Ok(())
}

fn setup_venv(environment: &Path, core: &Path) -> Result<()> {
    let python = environment.join(".venv/bin/python");
    if !python.is_file() {
        let status = Command::new("python3")
            .args(["-m", "venv", ".venv"])
            .current_dir(environment)
            .status()
            .with_context(|| format!("Failed to create venv in {}", environment.display()))?;
        if !status.success() {
            bail!("Failed to create venv in {}.", environment.display());
        }
    }
    let status = Command::new(&python)
        .args(["-m", "pip", "install", "-e"])
        .arg(core)
        .status()
        .with_context(|| format!("Failed to install BonesInfra dependencies from {}", core.display()))?;
    if !status.success() {
        bail!("Failed to install BonesInfra dependencies from {}.", core.display());
    }
    Ok(())
}

fn environment_dir(project_root: &Path) -> PathBuf {
    let mut environment = paths::bones_cache_root().join("bonesinfra/projects");
    for component in project_root.components() {
        if let Component::Normal(part) = component {
            environment.push(part);
        }
    }
    environment
}

fn package_version(core: &Path) -> Result<String> {
    fs::read_to_string(core.join("pyproject.toml"))
        .with_context(|| format!("Failed to read project-local BonesInfra package metadata in {}", core.display()))?
        .lines()
        .find_map(|line| line.strip_prefix("version = \"")?.strip_suffix('"'))
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Project-local BonesInfra pyproject.toml has no package version"))
}

fn materialized_stamp(directory: &Path) -> Option<String> {
    fs::read_to_string(directory.join(STAMP_FILE)).ok().map(|value| value.trim().to_string())
}

fn remove_path(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_dir() {
            fs::remove_dir_all(path).with_context(|| format!("Failed to remove {}", path.display()))?;
        } else {
            fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
        }
    }
    Ok(())
}
