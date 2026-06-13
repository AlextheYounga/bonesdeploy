use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rust_embed::Embed;

use crate::config;

#[derive(Embed)]
#[folder = "./embeds/kit/"]
struct Kit;

#[derive(Embed)]
#[folder = "../../infra"]
#[exclude = "__pycache__/**"]
#[exclude = ".venv/**"]
#[exclude = ".gitignore"]
#[exclude = "README.md"]
#[exclude = ".python-version"]
#[exclude = "pyproject.toml"]
#[exclude = "uv.lock"]
struct Infra;

#[derive(Embed)]
#[folder = "./embeds/runtimes/"]
struct Runtimes;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }

    for file_path in Infra::iter() {
        let Some(asset) = Infra::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, &format!("infra/{file_path}"), asset.data.as_ref())?;
    }
    Ok(())
}

pub fn scaffold_runtime_base(bones_dir: &Path) -> Result<()> {
    for file_path in Infra::iter() {
        let Some(asset) = Infra::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, &format!("infra/{file_path}"), asset.data.as_ref())?;
    }

    Ok(())
}
pub fn scaffold_runtime_template(template_name: &str, bones_dir: &Path) -> Result<()> {
    let prefix = format!("{template_name}/");
    let mut found = false;

    for file_path in Runtimes::iter() {
        if !file_path.starts_with(&prefix) {
            continue;
        }

        let Some(asset) = Runtimes::get(&file_path) else {
            continue;
        };

        found = true;
        let relative_path = file_path.trim_start_matches(&prefix);

        write_asset(bones_dir, relative_path, asset.data.as_ref())?;
    }

    if !found {
        bail!(
            "Embedded runtime template not found: {template_name}. Available templates: {}",
            available_templates().join(", ")
        );
    }

    Ok(())
}

pub fn read_template_runtime_config(template_name: &str) -> Result<String> {
    let path = format!("{template_name}/runtime.yaml");
    let Some(file) = Runtimes::get(&path) else {
        bail!(
            "Embedded runtime config not found for template: {template_name}. Available templates: {}",
            available_templates().join(", ")
        );
    };

    Ok(String::from_utf8_lossy(file.data.as_ref()).to_string())
}

pub fn read_kit_runtime_config() -> Result<String> {
    let path = "runtime.yaml";
    let Some(file) = Kit::get(path) else {
        bail!("Embedded kit runtime config not found");
    };
    Ok(String::from_utf8_lossy(file.data.as_ref()).to_string())
}

pub fn available_templates() -> Vec<String> {
    let mut templates = BTreeSet::new();

    for file_path in Runtimes::iter() {
        if let Some((name, _)) = file_path.split_once('/') {
            templates.insert(name.to_string());
        }
    }

    templates.into_iter().collect()
}

fn write_asset(bones_dir: &Path, relative_path: &str, bytes: &[u8]) -> Result<()> {
    let dest = bones_dir.join(relative_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(&dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if relative_path.starts_with(config::Constants::ASSET_HOOKS_DIR)
        || relative_path.starts_with(config::Constants::ASSET_DEPLOYMENT_DIR)
    {
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{Result, anyhow};

    /// Does not pass a `--config` flag to the doctor command in the hooks script.
    #[test]
    fn hooks_script_does_not_pass_config_to_doctor() -> Result<()> {
        let hooks_script = super::Kit::get("hooks/hooks.sh").ok_or_else(|| anyhow!("hooks.sh should be embedded"))?;
        let hooks_script = String::from_utf8_lossy(hooks_script.data.as_ref()).to_string();
        assert!(!hooks_script.contains("bonesremote doctor --config"));
        Ok(())
    }
}
