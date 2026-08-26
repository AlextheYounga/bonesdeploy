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
    if descriptor.branch.is_empty() {
        bail!("Deployment config descriptor has an empty branch");
    }
    config::validate_runtime(&descriptor.runtime)
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
    fs::set_permissions(parent, fs::Permissions::from_mode(0o755))
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

#[cfg(test)]
mod tests {
    use super::*;
    use bonesdeploy_core::config::{Build, Runtime, RuntimeBackend};
    use std::env;

    fn root(name: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("bonesremote-control-plane-{name}-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn descriptor(branch: &str, backend: RuntimeBackend) -> RemoteDeploymentConfig {
        RemoteDeploymentConfig {
            branch: branch.to_string(),
            releases_keep: 5,
            runtime: Runtime { backend, ..Runtime::default() },
            build: Build::default(),
            services: vec!["postgres".to_string()],
        }
    }

    #[test]
    fn overwrite_replaces_content_and_leaves_no_temp_files() -> Result<()> {
        let root = root("overwrite");
        let _scope = override_control_plane_root(root.clone());
        store("atlas", &descriptor("main", RuntimeBackend::Native))?;
        store("atlas", &descriptor("release", RuntimeBackend::Docker))?;
        assert_eq!(load("atlas")?.branch, "release");
        assert_eq!(fs::read_dir(root.join("atlas"))?.count(), 1);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn stored_snapshot_is_mode_0644_when_created_and_replaced() -> Result<()> {
        let root = root("mode");
        let _scope = override_control_plane_root(root.clone());
        store("atlas", &descriptor("main", RuntimeBackend::Native))?;
        store("atlas", &descriptor("next", RuntimeBackend::Native))?;
        assert_eq!(fs::metadata(snapshot_path("atlas"))?.permissions().mode() & 0o777, 0o644);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn load_reports_missing_snapshot_with_actionable_message() {
        let root = root("missing");
        let _scope = override_control_plane_root(root.clone());
        let Err(error) = load("atlas") else {
            return;
        };
        let message = error.to_string();
        assert!(message.contains("bones.json"));
        assert!(message.contains("bonesdeploy site doctor locally"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn load_rejects_unknown_fields() -> Result<()> {
        let root = root("unknown");
        let _scope = override_control_plane_root(root.clone());
        let path = snapshot_path("atlas");
        let Some(parent) = path.parent() else { return Ok(()) };
        fs::create_dir_all(parent)?;
        fs::write(path, r#"{"branch":"main","releases_keep":5,"runtime":{},"build":{},"services":[],"bogus":true}"#)?;
        assert!(load("atlas").is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn store_rejects_empty_branch() {
        let error = match store("atlas", &descriptor("", RuntimeBackend::Native)) {
            Ok(()) => return,
            Err(error) => error,
        };
        assert!(error.to_string().contains("empty branch"));
    }

    #[test]
    fn load_round_trips_branch_backend_and_runtime_extra() -> Result<()> {
        let root = root("round-trip");
        let _scope = override_control_plane_root(root.clone());
        let mut expected = descriptor("feature/login", RuntimeBackend::Docker);
        expected.runtime.extra.insert("workers".to_string(), toml::Value::Integer(3));
        store("atlas", &expected)?;
        let actual = load("atlas")?;
        assert_eq!(actual.branch, "feature/login");
        assert_eq!(actual.runtime.backend, RuntimeBackend::Docker);
        assert_eq!(actual.runtime.extra.get("workers"), Some(&toml::Value::Integer(3)));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
