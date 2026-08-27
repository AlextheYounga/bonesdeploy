//! Embedded BonesInfra wheel and project template materialization.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use rust_embed::Embed;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

#[derive(Embed)]
#[folder = "assets/"]
#[include = "bonesinfra-*.whl"]
struct WheelAsset;

#[derive(Embed)]
#[folder = "python/src/bonesinfra/"]
#[include = "assets/**"]
#[include = "frameworks/**/templates/**"]
struct TemplateAsset;

const STAMP_FILE: &str = ".stamp";
const PREVIOUS_FRAMEWORK_PATH: &str = "infra/.framework";

/// Writes the embedded wheel and all managed templates into a project.
///
/// # Errors
/// Fails when either managed artifact cannot be atomically replaced.
pub fn materialize_project_artifacts(project_root: &Path) -> Result<PathBuf> {
    let infra = project_root.join(paths::LOCAL_INFRA_DIR);
    fs::create_dir_all(&infra).with_context(|| format!("Failed to create {}", infra.display()))?;

    let wheel_name = embedded_wheel_name()?;
    let wheel = embedded_wheel()?;
    remove_project_wheels(&infra)?;
    replace_file(&infra.join(&wheel_name), &wheel)?;
    replace_templates(&project_root.join(paths::LOCAL_INFRA_TEMPLATES_DIR))?;
    remove_path(&project_root.join(PREVIOUS_FRAMEWORK_PATH))?;
    Ok(infra.join(wheel_name))
}

/// Runs `python -m bonesinfra` from the current project's cached environment.
///
/// # Errors
/// Fails when the project artifact or dependency environment cannot be prepared,
/// or when the command exits non-zero.
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

/// Runs one BonesInfra command with a typed provisioning request on stdin.
///
/// # Errors
/// Returns an error when the child process exits non-zero.
pub fn run_with_request(args: &[&str], request_body: &str) -> Result<()> {
    let project_root = env::current_dir().context("Failed to determine project directory")?;
    let executable = ensure_available(&project_root)?;
    let mut command = base_command(&executable, &project_root, args);
    command.stdin(Stdio::piped());
    let mut child = command.spawn().with_context(|| format!("Failed to run bonesinfra {}", args.join(" ")))?;
    let mut stdin = child.stdin.take().context("bonesinfra stdin was not piped")?;
    stdin
        .write_all(request_body.as_bytes())
        .with_context(|| format!("Failed to write request to bonesinfra {}", args.join(" ")))?;
    drop(stdin);
    let status = child.wait().with_context(|| format!("Failed to wait on bonesinfra {}", args.join(" ")))?;
    if !status.success() {
        bail!("bonesinfra {} failed", args.join(" "));
    }
    Ok(())
}

/// Prepares the dependency environment for a project.
///
/// # Errors
/// Fails when the project artifact is missing or dependencies cannot install.
pub fn prepare_in(project_root: &Path) -> Result<()> {
    ensure_available(project_root).map(|_| ())
}

/// Returns the embedded wheel bytes.
pub fn embedded_wheel() -> Result<Vec<u8>> {
    let asset = WheelAsset::get(&embedded_wheel_name()?).context("embedded BonesInfra wheel is missing")?;
    validate_wheel(asset.data.as_ref())?;
    Ok(asset.data.into_owned())
}

fn embedded_wheel_name() -> Result<String> {
    let mut wheels = WheelAsset::iter().filter(|path| path.ends_with(".whl"));
    let wheel = wheels.next().context("embedded BonesInfra wheel is missing")?.into_owned();
    if wheels.next().is_some() {
        bail!("multiple embedded BonesInfra wheels found");
    }
    Ok(wheel)
}

/// Returns the embedded template paths relative to the BonesInfra package.
pub fn embedded_template_paths() -> impl Iterator<Item = String> {
    TemplateAsset::iter().map(|path| path.into_owned())
}

fn ensure_available(project_root: &Path) -> Result<PathBuf> {
    let project_root = project_root
        .canonicalize()
        .with_context(|| format!("Failed to resolve project root {}", project_root.display()))?;
    let wheel = project_wheel(&project_root)?;
    let wheel_bytes = fs::read(&wheel).with_context(|| format!("Failed to read {}", wheel.display()))?;
    validate_wheel(&wheel_bytes)?;

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
    let stamp = wheel_stamp(&wheel_bytes);
    if python.is_file() && materialized_stamp(&environment).as_deref() == Some(stamp.as_str()) {
        return Ok(python);
    }

    setup_venv(&environment, &wheel)?;
    if !python.is_file() {
        bail!("BonesInfra setup finished at {}, but {} is missing.", environment.display(), python.display());
    }
    fs::write(environment.join(STAMP_FILE), stamp)
        .with_context(|| format!("Failed to write BonesInfra stamp in {}", environment.display()))?;
    Ok(python)
}

fn base_command(executable: &Path, project_root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(executable);
    command.current_dir(project_root).args(["-m", "bonesinfra"]).args(args);
    command
}

fn replace_templates(destination: &Path) -> Result<()> {
    let parent = destination.parent().context("template directory has no parent")?;
    let staging = tempfile::Builder::new().prefix(".templates.").tempdir_in(parent)?;
    for relative_path in embedded_template_paths() {
        let Some(asset) = TemplateAsset::get(&relative_path) else { continue };
        let target = if relative_path.starts_with("assets/") {
            staging.path().join("shared").join(relative_path.trim_start_matches("assets/"))
        } else if let Some(path) = relative_path.strip_prefix("frameworks/") {
            let Some((framework, template)) = path.split_once("/templates/") else { continue };
            staging.path().join("frameworks").join(framework).join(template)
        } else {
            continue;
        };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, asset.data.as_ref())?;
    }
    replace_directory(destination, staging)
}

fn replace_directory(destination: &Path, staging: tempfile::TempDir) -> Result<()> {
    let previous = destination.with_extension("previous");
    remove_path(&previous)?;
    if destination.exists() || destination.is_symlink() {
        fs::rename(destination, &previous)?;
    }
    if let Err(error) = fs::rename(staging.keep(), destination) {
        if previous.exists() {
            let _ = fs::rename(&previous, destination);
        }
        return Err(error).with_context(|| format!("Failed to replace {}", destination.display()));
    }
    remove_path(&previous)
}

fn replace_file(destination: &Path, bytes: &[u8]) -> Result<()> {
    let parent = destination.parent().context("wheel path has no parent")?;
    let staging = tempfile::NamedTempFile::new_in(parent)?;
    fs::write(staging.path(), bytes)?;
    staging
        .persist(destination)
        .map(|_| ())
        .map_err(|error| error.error)
        .with_context(|| format!("Failed to replace {}", destination.display()))
}

fn project_wheel(project_root: &Path) -> Result<PathBuf> {
    let infra = project_root.join(paths::LOCAL_INFRA_DIR);
    let wheels = fs::read_dir(&infra)
        .with_context(|| format!("Failed to read {}", infra.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("bonesinfra-"))
                && path.extension().is_some_and(|extension| extension == "whl")
        })
        .collect::<Vec<_>>();
    let [wheel] = wheels.as_slice() else {
        bail!("Project-local BonesInfra wheel is missing at {}. Run bonesdeploy init or update.", infra.display());
    };
    Ok(wheel.clone())
}

fn remove_project_wheels(infra: &Path) -> Result<()> {
    for entry in fs::read_dir(infra)? {
        let path = entry?.path();
        if path
            .file_name()
            .is_some_and(|name| name == "bonesinfra.whl" || name.to_string_lossy().starts_with("bonesinfra-"))
            && path.extension().is_some_and(|extension| extension == "whl")
        {
            remove_path(&path)?;
        }
    }
    Ok(())
}

fn setup_venv(environment: &Path, wheel: &Path) -> Result<()> {
    let python = environment.join(".venv/bin/python");
    if !python.is_file() {
        let status = Command::new("python3").args(["-m", "venv", ".venv"]).current_dir(environment).status()?;
        if !status.success() {
            bail!("Failed to create venv in {}.", environment.display());
        }
    }
    let status = Command::new(&python).args(["-m", "pip", "install", "--force-reinstall"]).arg(wheel).status()?;
    if !status.success() {
        bail!("Failed to install BonesInfra wheel from {}.", wheel.display());
    }
    Ok(())
}

fn validate_wheel(bytes: &[u8]) -> Result<()> {
    if !bytes.starts_with(b"PK\x03\x04") {
        bail!("embedded BonesInfra artifact is not a ZIP wheel");
    }
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).context("embedded BonesInfra artifact is not a valid wheel archive")?;
    let wheel_metadata = (0..archive.len())
        .find_map(|index| {
            let file = archive.by_index(index).ok()?;
            file.name().ends_with(".dist-info/WHEEL").then(|| file.name().to_string())
        })
        .context("BonesInfra wheel has no WHEEL metadata")?;
    let mut metadata = archive.by_name(&wheel_metadata).context("failed to read BonesInfra WHEEL metadata")?;
    let mut contents = String::new();
    metadata.read_to_string(&mut contents)?;
    if !contents.lines().any(|line| line.trim() == "Root-Is-Purelib: true")
        || !contents.lines().any(|line| line.trim() == "Tag: py3-none-any")
    {
        bail!("BonesInfra wheel is not a portable py3-none-any wheel");
    }
    Ok(())
}

fn wheel_stamp(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
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

fn materialized_stamp(directory: &Path) -> Option<String> {
    fs::read_to_string(directory.join(STAMP_FILE)).ok().map(|value| value.trim().to_string())
}

fn remove_path(path: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
