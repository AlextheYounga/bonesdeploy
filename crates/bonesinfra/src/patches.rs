use bonesdeploy_core::state;

use std::cmp::Ordering;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;
use rustinfra::operations::server::{Shell, ShellCommands};
use rustinfra::operations::{OperationOptions, PreparedDeploy};

use crate::{ProvisioningContext, ProvisioningReport, ProvisioningRuntime, reject_custom_provisioning};

const PATCH_ID: &str = "0003-project-infra";
const PATCH_VERSION: Version = Version { major: 0, minor: 8, patch: 0 };
const OWNED_ENTRIES: &[&str] = &["deployment", "infra", "secrets"];
const CONTROL_ENTRIES: &[&str] = &[".git", paths::GITIGNORE_FILE, "bones.toml"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Applies every local patch introduced at or before the target version.
///
/// # Errors
/// Returns an error for an invalid version or an ambiguous/unsafe existing
/// project tree. Existing files are never merged or overwritten.
pub fn apply_local(config: &Bones, project_root: &Path, target_version: &str) -> Result<()> {
    if parse_version(target_version)? < PATCH_VERSION {
        return Ok(());
    }
    let marker = local_marker(config, PATCH_ID)?;
    if marker.exists() {
        return Ok(());
    }
    migrate_to_project_infra(project_root)?;
    write_local_marker(&marker)
}

/// Prepares an idempotent remote patch-marker plan without connecting.
///
/// # Errors
/// Returns an error for an invalid version or operation graph.
pub fn prepare_remote(
    context: &ProvisioningContext,
    runtime: &ProvisioningRuntime,
    target_version: &str,
) -> Result<PreparedDeploy> {
    if parse_version(target_version)? < PATCH_VERSION {
        return runtime.prepare(|_| Ok(()));
    }
    let marker = format!("/var/lib/bonesdeploy/patches/{}/{PATCH_ID}", context.project_name);
    let parent = Path::new(&marker)
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| anyhow::anyhow!("remote patch marker has no parent"))?;
    let command = format!(
        "if [ ! -e {marker} ]; then mkdir -p {parent}; tmp={marker}.tmp-$$; printf 'completed\\n' > \"$tmp\"; mv \"$tmp\" {marker}; fi",
        marker = shell_escape::escape(marker.as_str().into()),
        parent = shell_escape::escape(parent.into()),
    );
    runtime.prepare(|host| {
        host.add_with_options(
            Shell,
            ShellCommands::from(command.clone()),
            OperationOptions {
                name: Some(format!("Write remote patch marker {PATCH_ID}")),
                ..OperationOptions::default()
            },
        )?;
        Ok(())
    })
}

/// Applies remote version-gated patch markers directly through RustInfra.
///
/// # Errors
/// Returns an error for existing custom provisioning, invalid configuration or
/// version, or failed RustInfra operations.
pub async fn apply_remote(config: &Bones, project_root: &Path, target_version: &str) -> Result<ProvisioningReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_remote(&context, &runtime, target_version)?;
    runtime.execute(&deploy).await
}

fn parse_version(value: &str) -> Result<Version> {
    let core = value.split(['-', '+']).next().unwrap_or(value);
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        bail!("invalid version '{value}'");
    }
    Ok(Version {
        major: parts[0].parse().with_context(|| format!("invalid version '{value}'"))?,
        minor: parts[1].parse().with_context(|| format!("invalid version '{value}'"))?,
        patch: parts[2].parse().with_context(|| format!("invalid version '{value}'"))?,
    })
}

fn local_marker(config: &Bones, patch: &str) -> Result<PathBuf> {
    let root = env::var_os(state::XDG_DATA_HOME).map_or_else(
        || {
            env::var_os(state::HOME)
                .map(PathBuf::from)
                .map(|home| home.join(state::LOCAL_DATA_DIR))
                .ok_or_else(|| anyhow::anyhow!("HOME is required to locate local patch markers"))
        },
        |path| Ok(PathBuf::from(path)),
    )?;
    Ok(root.join("bonesdeploy/patches").join(&config.project_name).join(patch))
}

fn write_local_marker(marker: &Path) -> Result<()> {
    let parent = marker.parent().ok_or_else(|| anyhow::anyhow!("local patch marker has no parent"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.tmp-{}", PATCH_ID, process::id()));
    fs::write(&temporary, "completed\n")?;
    fs::rename(temporary, marker)?;
    Ok(())
}

fn migrate_to_project_infra(project_root: &Path) -> Result<()> {
    let old = project_root.join(paths::OLD_BONES_DIR);
    if !old.exists() && !old.is_symlink() {
        return Ok(());
    }
    let source = if old.is_symlink() { old.canonicalize()? } else { old.clone() };
    if !source.is_dir() {
        bail!(".bones is neither a directory nor a symlink; migration refused");
    }
    validate_source(&source)?;
    let staging = tempfile::Builder::new().prefix(".infra.migrate-").tempdir_in(project_root)?;
    copy_owned_content(&source, staging.path())?;
    verify_owned_content(&source, staging.path())?;
    move_owned_content(staging.path(), &project_root.join("infra"))?;
    verify_owned_content(&source, &project_root.join("infra"))?;
    if old.is_symlink() {
        fs::remove_file(old)?;
    } else {
        fs::remove_dir_all(old)?;
    }
    Ok(())
}

fn validate_source(source: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !OWNED_ENTRIES.contains(&name.as_ref()) && !CONTROL_ENTRIES.contains(&name.as_ref()) {
            bail!("Unexpected entry {} in old .bones; migration is ambiguous", entry.path().display());
        }
        if CONTROL_ENTRIES.contains(&name.as_ref()) {
            if entry.path().is_symlink() {
                bail!("Old control entry {} is a symlink; migration refused", entry.path().display());
            }
        } else {
            validate_tree(&entry.path())?;
        }
    }
    let old_infra = source.join("infra");
    if old_infra.is_dir() {
        for name in ["deployment", "secrets"] {
            let nested = old_infra.join(name);
            if nested.exists() || nested.is_symlink() {
                bail!("Both .bones/infra/{name} and .bones/{name} exist; migration is ambiguous");
            }
        }
    }
    Ok(())
}

fn validate_tree(path: &Path) -> Result<()> {
    if path.is_symlink() || !(path.is_dir() || path.is_file()) {
        bail!("Unsafe entry {} in old .bones; migration refused", path.display());
    }
    if path.is_dir() {
        for child in fs::read_dir(path)? {
            validate_tree(&child?.path())?;
        }
    }
    Ok(())
}

fn copy_owned_content(source: &Path, destination: &Path) -> Result<()> {
    let old_infra = source.join("infra");
    if old_infra.is_dir() {
        let custom = destination.join("custom");
        fs::create_dir(&custom)?;
        for entry in fs::read_dir(old_infra)? {
            let entry = entry?;
            copy_tree(&entry.path(), &custom.join(entry.file_name()))?;
        }
        fs::write(custom.join("__init__.py"), "")?;
    }
    for name in ["deployment", "secrets"] {
        let entry = source.join(name);
        if entry.exists() || entry.is_symlink() {
            copy_tree(&entry, &destination.join(name))?;
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        fs::create_dir(destination)?;
        fs::set_permissions(destination, fs::metadata(source)?.permissions())?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if source.is_file() {
        fs::copy(source, destination)?;
        fs::set_permissions(destination, fs::metadata(source)?.permissions())?;
    } else {
        bail!("Unsafe entry {} in old .bones; migration refused", source.display());
    }
    Ok(())
}

fn verify_owned_content(source: &Path, destination: &Path) -> Result<()> {
    let old_infra = source.join("infra");
    if old_infra.is_dir() {
        for entry in fs::read_dir(old_infra)? {
            let entry = entry?;
            verify_tree(&entry.path(), &destination.join("custom").join(entry.file_name()))?;
        }
    }
    for name in ["deployment", "secrets"] {
        let entry = source.join(name);
        if entry.exists() || entry.is_symlink() {
            verify_tree(&entry, &destination.join(name))?;
        }
    }
    Ok(())
}

fn verify_tree(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() != destination.is_dir() || source.is_file() != destination.is_file() {
        bail!("Migration destination has an unsafe type for {}", destination.display());
    }
    if source.is_file() {
        if fs::read(source)? != fs::read(destination)? {
            bail!("Migration verification failed for {}", destination.display());
        }
        return Ok(());
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        verify_tree(&entry.path(), &destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn move_owned_content(staging: &Path, destination: &Path) -> Result<()> {
    for name in ["deployment", "secrets", "custom"] {
        let source = staging.join(name);
        if source.exists() {
            let target = destination.join(name);
            if target.exists() || target.is_symlink() {
                bail!("{} already exists; migration will not merge or overwrite it", target.display());
            }
        }
    }
    fs::create_dir_all(destination)?;
    for name in ["deployment", "secrets", "custom"] {
        let source = staging.join(name);
        if source.exists() {
            fs::rename(&source, destination.join(name))?;
        }
    }
    Ok(())
}
