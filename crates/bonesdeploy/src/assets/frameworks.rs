use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use rust_embed::Embed;
use serde_json::{Map, Value};

use bonesdeploy_core::config::Runtime;
use bonesdeploy_core::paths;

use super::{kit, write_asset};
use crate::frameworks;

#[derive(Embed)]
#[folder = "./assets/frameworks/"]
struct FrameworkAssets;

pub fn framework_asset(path: &str) -> Option<Vec<u8>> {
    FrameworkAssets::get(path).map(|asset| asset.data.into_owned())
}

pub fn framework_asset_paths() -> Vec<String> {
    FrameworkAssets::iter().map(|path| path.into_owned()).collect()
}

pub fn framework_names() -> Vec<String> {
    FrameworkAssets::iter()
        .filter_map(|path| path.split('/').next().map(str::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn base_framework_defaults() -> Result<Map<String, Value>> {
    serde_json::to_value(Runtime::default())
        .ok()
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| anyhow!("Failed to serialize base framework defaults"))
}

pub fn framework_defaults(framework: &str) -> Result<Map<String, Value>> {
    let selected = frameworks::Framework::parse(framework)?;
    let Some(values) = selected.runtime_defaults()? else {
        let mut values = base_framework_defaults()?;
        values.insert("template".into(), Value::String(selected.to_string()));
        return Ok(values);
    };
    Ok(values)
}

pub fn scaffold_framework_project(framework: &str, bones_dir: &Path) -> Result<()> {
    let framework = frameworks::Framework::parse(framework)?;
    let deploy_dir = bones_dir.join(paths::DEPLOYMENT_DIR);
    if deploy_dir.exists() {
        fs::remove_dir_all(&deploy_dir)
            .with_context(|| format!("Failed to clear deployment dir: {}", deploy_dir.display()))?;
    }
    kit::scaffold_deployment_functions(bones_dir)?;
    if framework != frameworks::Framework::Custom {
        scaffold_framework_assets(&framework.to_string(), bones_dir, paths::KIT_DEPLOYMENT_DIR)?;
    }
    Ok(())
}

pub fn scaffold_framework_env_build(framework: &str, project_root: &Path, framework_config: &Runtime) -> Result<()> {
    let framework = frameworks::Framework::parse(framework)?;
    let Some(content) = framework.build_environment_example(framework_config) else {
        return Ok(());
    };

    let destination = project_root.join(paths::ENV_BUILD_FILE);
    if destination.exists() {
        return Ok(());
    }

    fs::write(&destination, content).with_context(|| format!("Failed to write {}", destination.display()))?;
    Ok(())
}

fn scaffold_framework_assets(framework: &str, bones_dir: &Path, asset_prefix: &str) -> Result<()> {
    let framework_prefix = format!("{framework}/");

    for file_path in FrameworkAssets::iter() {
        let Some(stripped) = file_path.strip_prefix(&framework_prefix) else {
            continue;
        };

        if !stripped.starts_with(asset_prefix) {
            continue;
        }

        let Some(asset) = FrameworkAssets::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, stripped, asset.data.as_ref())?;
    }

    Ok(())
}
