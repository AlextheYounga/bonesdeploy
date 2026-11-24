use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write, stdin};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::RemoteDeploymentConfig;
use bonesdeploy_core::{config, paths};

use std::thread_local;

thread_local! {
    static CONF_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub fn override_control_plane_root(root: PathBuf) -> ScopedConfRoot {
    ScopedConfRoot(CONF_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root))))
}

#[must_use = "the scope must be retained for the root override to remain active"]
pub struct ScopedConfRoot(Option<PathBuf>);

impl Drop for ScopedConfRoot {
    fn drop(&mut self) {
        CONF_ROOT_OVERRIDE.with(|slot| {
            slot.replace(self.0.take());
        });
    }
}

fn resolved_conf_root() -> PathBuf {
    CONF_ROOT_OVERRIDE
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(|| PathBuf::from(paths::DEFAULT_CONF_ROOT_PARENT))
}

pub fn read_stdin_descriptor() -> Result<RemoteDeploymentConfig> {
    let mut input = String::new();
    stdin().read_to_string(&mut input).context("Failed to read deployment config from stdin")?;
    let descriptor: RemoteDeploymentConfig =
        serde_json::from_str(&input).context("Failed to parse deployment config descriptor from stdin")?;
    validate_descriptor(&descriptor)?;
    Ok(descriptor)
}

fn validate_descriptor(descriptor: &RemoteDeploymentConfig) -> Result<()> {
    descriptor.validate()
}

#[expect(clippy::panic, reason = "the required PathBuf API cannot return site validation errors")]
pub fn snapshot_path(site: &str) -> PathBuf {
    if let Err(error) = config::validate_site_name(site) {
        panic!("invalid site name for snapshot path: {error}");
    }
    resolved_conf_root().join(site).join("bones.json")
}

pub fn store(site: &str, descriptor: &RemoteDeploymentConfig) -> Result<()> {
    config::validate_site_name(site)?;
    validate_descriptor(descriptor)?;
    let path = snapshot_path(site);
    let parent = path.parent().context("Control-plane snapshot has no parent directory")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create control-plane directory {}", parent.display()))?;
    // 0750 matches the provisioned /srv/conf/<site> mode (root, runtime group).
    fs::set_permissions(parent, fs::Permissions::from_mode(0o750))
        .with_context(|| format!("Failed to set control-plane directory permissions {}", parent.display()))?;

    let mut content = serde_json::to_string_pretty(descriptor).context("Failed to serialize control-plane snapshot")?;
    content.push('\n');
    let temp = parent.join(format!(".bones.json.tmp-{}", process::id()));
    {
        let mut file = File::create(&temp).with_context(|| format!("Failed to create {}", temp.display()))?;
        file.write_all(content.as_bytes()).with_context(|| format!("Failed to write {}", temp.display()))?;
        file.flush().with_context(|| format!("Failed to flush {}", temp.display()))?;
        file.sync_all().with_context(|| format!("Failed to sync {}", temp.display()))?;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o644))
            .with_context(|| format!("Failed to set permissions on {}", temp.display()))?;
    }
    fs::rename(&temp, &path).with_context(|| format!("Failed to install {}", path.display()))?;
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .with_context(|| format!("Failed to sync {}", parent.display()))?;
    Ok(())
}

pub fn load(site: &str) -> Result<RemoteDeploymentConfig> {
    config::validate_site_name(site)?;
    let path = snapshot_path(site);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!(
                "Site configuration synchronized copy missing {} — run bonesdeploy site doctor locally or redeploy to synchronize",
                path.display()
            )
        }
        Err(error) => return Err(error).with_context(|| format!("Failed to read {}", path.display())),
    };
    let descriptor: RemoteDeploymentConfig =
        serde_json::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    validate_descriptor(&descriptor).with_context(|| format!("Invalid control-plane snapshot {}", path.display()))?;
    Ok(descriptor)
}
