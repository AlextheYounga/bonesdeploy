use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rust_embed::Embed;

use crate::config;

#[derive(Embed)]
#[folder = "../../kit/"]
struct Kit;

#[derive(Embed)]
#[folder = "../../templates/"]
struct Templates;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }
    Ok(())
}

pub fn scaffold_template(template_name: &str, bones_dir: &Path) -> Result<()> {
    let prefix = format!("{template_name}/");
    let mut found = false;

    for file_path in Templates::iter() {
        if !file_path.starts_with(&prefix) {
            continue;
        }

        let Some(asset) = Templates::get(&file_path) else {
            continue;
        };

        let relative_path = file_path.trim_start_matches(&prefix);
        write_asset(bones_dir, relative_path, asset.data.as_ref())?;
        found = true;
    }

    if !found {
        bail!(
            "Embedded template not found: {template_name}. Available templates: {}",
            available_templates().join(", ")
        );
    }

    Ok(())
}

pub fn available_templates() -> Vec<String> {
    let mut templates = BTreeSet::new();

    for file_path in Templates::iter() {
        if let Some((name, _)) = file_path.split_once('/') {
            templates.insert(name.to_string());
        }
    }

    templates.into_iter().collect()
}

pub fn read_asset(path: &str) -> Result<String> {
    let asset = Kit::get(path);
    match asset {
        Some(file) => Ok(String::from_utf8_lossy(file.data.as_ref()).to_string()),
        None => bail!("Embedded asset not found: {path}"),
    }
}

fn write_asset(bones_dir: &Path, relative_path: &str, bytes: &[u8]) -> Result<()> {
    let dest = bones_dir.join(relative_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(&dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if relative_path.starts_with(config::Constants::ASSET_HOOKS_DIR)
        || relative_path.starts_with(config::Constants::ASSET_DEPLOYMENT_DIR)
        || relative_path.starts_with(config::Constants::ASSET_SCRIPTS_DIR)
    {
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}
