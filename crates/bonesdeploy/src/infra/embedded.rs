use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str;

use anyhow::{Context, Result, anyhow, bail};
use rust_embed::Embed;
use serde_json::{Map, Value};

use shared::paths;

#[derive(Embed)]
#[folder = "./kit/"]
struct Kit;

#[derive(Embed)]
#[folder = "./runtimes/"]
struct RuntimeAssets;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }

    Ok(())
}

pub fn runtime_names() -> Vec<String> {
    RuntimeAssets::iter()
        .filter_map(|path| path.split('/').next().map(str::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn base_runtime_defaults() -> Result<Map<String, Value>> {
    runtime_defaults_from_bytes("kit/runtime.toml", Kit::get("runtime.toml").map(|asset| asset.data))
}

pub fn runtime_defaults(runtime: &str) -> Result<Map<String, Value>> {
    let asset_path = format!("{runtime}/runtime.toml");
    runtime_defaults_from_bytes(&asset_path, RuntimeAssets::get(&asset_path).map(|asset| asset.data))
}

pub fn scaffold_runtime_deployment(runtime: &str, bones_dir: &Path) -> Result<()> {
    let deploy_dir = bones_dir.join(paths::DEPLOYMENT_DIR);
    if deploy_dir.exists() {
        fs::remove_dir_all(&deploy_dir)
            .with_context(|| format!("Failed to clear deployment dir: {}", deploy_dir.display()))?;
    }
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_DEPLOYMENT_DIR)
}

pub fn scaffold_runtime_secrets(runtime: &str, bones_dir: &Path) -> Result<()> {
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_SECRETS_DIR)
}

fn scaffold_runtime_assets(runtime: &str, bones_dir: &Path, asset_prefix: &str) -> Result<()> {
    let runtime_prefix = format!("{runtime}/");

    for file_path in RuntimeAssets::iter() {
        let Some(stripped) = file_path.strip_prefix(&runtime_prefix) else {
            continue;
        };

        if !stripped.starts_with(asset_prefix) {
            continue;
        }

        let Some(asset) = RuntimeAssets::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, stripped, asset.data.as_ref())?;
    }

    Ok(())
}

fn runtime_defaults_from_bytes(asset_path: &str, bytes: Option<impl AsRef<[u8]>>) -> Result<Map<String, Value>> {
    let Some(bytes) = bytes else {
        bail!("Missing embedded runtime defaults at {asset_path}");
    };

    let content =
        str::from_utf8(bytes.as_ref()).with_context(|| format!("Embedded asset {asset_path} is not valid UTF-8"))?;
    let toml_value: toml::Value = toml::from_str(content)
        .with_context(|| format!("Failed to parse embedded runtime defaults at {asset_path}"))?;
    let json_value = serde_json::to_value(toml_value)
        .with_context(|| format!("Failed to convert embedded runtime defaults at {asset_path} to JSON"))?;

    json_value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow!("Embedded runtime defaults at {asset_path} are not a TOML table"))
}

fn write_asset(bones_dir: &Path, relative_path: &str, bytes: &[u8]) -> Result<()> {
    let dest = bones_dir.join(relative_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(&dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if relative_path.starts_with(paths::KIT_HOOKS_DIR) || relative_path.starts_with(paths::KIT_DEPLOYMENT_DIR) {
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{base_runtime_defaults, runtime_defaults, runtime_names, scaffold, scaffold_runtime_deployment};
    use serde_json::Value;

    #[test]
    fn runtime_names_come_from_embedded_assets() {
        let runtimes = runtime_names();

        assert!(runtimes.contains(&String::from("laravel")));
        assert!(runtimes.contains(&String::from("next")));
    }

    #[test]
    fn runtime_defaults_read_local_runtime_toml() -> Result<()> {
        let defaults = runtime_defaults("laravel")?;

        assert_eq!(defaults.get("template"), Some(&Value::String(String::from("laravel"))));
        assert_eq!(defaults.get("php_version"), Some(&Value::String(String::from("8.5"))));
        Ok(())
    }

    #[test]
    fn vue_runtime_uses_vite_dist_web_root() -> Result<()> {
        let defaults = runtime_defaults("vue")?;

        assert_eq!(defaults.get("web_root"), Some(&Value::String(String::from("dist"))));
        Ok(())
    }

    #[test]
    fn base_runtime_defaults_read_embedded_kit_runtime() -> Result<()> {
        let defaults = base_runtime_defaults()?;

        assert!(defaults.contains_key("permissions"));
        Ok(())
    }

    #[test]
    fn scaffold_runtime_deployment_writes_runtime_scripts() -> Result<()> {
        let temp = TempDir::new()?;

        scaffold_runtime_deployment("laravel", temp.path())?;

        let deploy_dir = temp.path().join("deployment");
        assert!(deploy_dir.is_dir());
        let entries: Vec<_> = fs::read_dir(&deploy_dir)?.collect::<Result<Vec<_>, _>>()?;
        assert!(
            entries.iter().any(|e| fs::read_to_string(e.path()).is_ok_and(|c| c.contains("composer install"))),
            "no deployment script contains 'composer install'"
        );

        Ok(())
    }

    #[test]
    fn scaffold_runtime_deployment_replaces_kit_scripts() -> Result<()> {
        let temp = TempDir::new()?;

        scaffold(temp.path())?;
        let deploy_dir = temp.path().join("deployment");
        assert!(deploy_dir.join("02_run_build.sh").exists(), "kit script should exist after scaffold");

        scaffold_runtime_deployment("laravel", temp.path())?;

        let entries: Vec<String> = fs::read_dir(&deploy_dir)?
            .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
            .collect();
        assert!(
            !entries.iter().any(|n| n == "02_run_build.sh"),
            "kit script '02_run_build.sh' should be removed after runtime scaffold, got: {entries:?}"
        );
        assert!(
            entries.iter().any(|n| n == "02_install_php_deps.sh"),
            "laravel script should exist after runtime scaffold, got: {entries:?}"
        );

        Ok(())
    }
}
