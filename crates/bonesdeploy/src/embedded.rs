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

#[derive(Embed)]
#[folder = "scripts/"]
struct Scripts;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        if file_path.starts_with("runtime/") {
            continue;
        }

        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }
    Ok(())
}

pub fn scaffold_runtime_base(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        if !file_path.starts_with("runtime/") {
            continue;
        }

        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }

    Ok(())
}

pub fn scaffold_runtime_template(template_name: &str, bones_dir: &Path) -> Result<()> {
    let prefix = format!("{template_name}/runtime/");
    let mut found = false;

    for file_path in Templates::iter() {
        if !file_path.starts_with(&prefix) {
            continue;
        }

        let Some(asset) = Templates::get(&file_path) else {
            continue;
        };

        found = true;
        let relative_path = file_path.trim_start_matches(&prefix);
        if relative_path == "vars/setup.yml" {
            continue;
        }

        write_asset(bones_dir, &format!("runtime/{relative_path}"), asset.data.as_ref())?;
    }

    if !found {
        bail!(
            "Embedded runtime template not found: {template_name}. Available templates: {}",
            available_templates().join(", ")
        );
    }

    Ok(())
}

pub fn read_template_runtime_vars(template_name: &str) -> Result<String> {
    let path = format!("{template_name}/runtime/vars/setup.yml");
    let Some(file) = Templates::get(&path) else {
        bail!(
            "Embedded runtime vars not found for template: {template_name}. Available templates: {}",
            available_templates().join(", ")
        );
    };

    Ok(String::from_utf8_lossy(file.data.as_ref()).to_string())
}

pub fn read_template_bones_config(template_name: &str) -> Result<String> {
    let path = format!("{template_name}/bones.yaml");
    let Some(file) = Templates::get(&path) else {
        bail!(
            "Embedded bones config not found for template: {template_name}. Available templates: {}",
            available_templates().join(", ")
        );
    };

    Ok(String::from_utf8_lossy(file.data.as_ref()).to_string())
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
    if let Some(file) = Kit::get(path) {
        return Ok(String::from_utf8_lossy(file.data.as_ref()).to_string());
    }

    if let Some(file) = Scripts::get(path) {
        return Ok(String::from_utf8_lossy(file.data.as_ref()).to_string());
    }

    bail!("Embedded asset not found: {path}")
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
    use crate::config;

    use super::read_asset;

    /// Does not pass a `--config` flag to the doctor command in the hooks script.
    #[test]
    fn hooks_script_does_not_pass_config_to_doctor() {
        let hooks_script = read_asset("hooks/hooks.sh");
        assert!(hooks_script.is_ok(), "hooks.sh should be embedded");

        let hooks_script = hooks_script.unwrap_or_default();
        assert!(!hooks_script.contains("bonesremote doctor --config"));
    }

    /// Embeds the remote python bootstrap script from the crate-local scripts directory.
    #[test]
    fn python_bootstrap_script_is_embedded_from_crate_scripts_directory() {
        let script = read_asset(config::Constants::PYTHON_BOOTSTRAP_SCRIPT_ASSET);
        assert!(script.is_ok(), "python bootstrap script should be embedded");

        let script = script.unwrap_or_default();
        assert!(script.contains("apt-get install -y python3 python3-apt"));
    }
}
