use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result};
use rust_embed::Embed;

use crate::config;

#[derive(Embed)]
#[folder = "./kit/"]
struct Kit;

#[derive(Embed)]
#[folder = "../../infra"]
#[exclude = "tests"]
#[exclude = "__pycache__/**"]
#[exclude = ".venv/**"]
#[exclude = ".gitignore"]
#[exclude = "README.md"]
#[exclude = ".python-version"]
#[exclude = "pyproject.toml"]
#[exclude = "uv.lock"]
struct Infra;

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
