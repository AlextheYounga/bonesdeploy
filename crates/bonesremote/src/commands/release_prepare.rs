use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config::{Runtime, load_runtime};
use shared::paths;
use shared::paths::default_web_root;
use shared::registry;

use crate::privileges;
use crate::release::scripts as deploy_output;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release prepare")?;
    registry::validate_site_name(site)?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let scripts_dir =
        paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR).join(paths::DEPLOYMENT_PREPARE_DIR);
    if !scripts_dir.is_dir() {
        println!("No prepare scripts at {}; skipping prepare.", scripts_dir.display());
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No prepare scripts found at {}; skipping prepare.", scripts_dir.display());
        return Ok(());
    }

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let runtime = load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| Runtime {
        web_root: default_web_root(),
        build_image: String::new(),
        runtime_user: String::new(),
        runtime_group: String::new(),
        release_group: String::new(),
    });

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running prepare script {script_name}...");

        let status = deploy_output::run_prepare_script(
            &script,
            &release_dir,
            &release_dir.join(format!("{script_name}.log")),
            &deploy_output::PrepareScriptEnv {
                project_name: &cfg.site,
                project_root: &cfg.site_root,
                runtime_user: &cfg.runtime_user,
                web_root: &runtime.web_root,
            },
        )
        .with_context(|| format!("Failed to execute prepare script {}", script.display()))?;

        if !status.success() {
            bail!("Prepare script {script_name} exited with status {status}");
        }
    }

    Ok(())
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::list_scripts;

    #[test]
    fn list_scripts_sorts_prepare_scripts() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-prepare-list-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("02_second.sh"), "")?;
        fs::write(root.join("01_first.sh"), "")?;
        fs::create_dir_all(root.join("nested"))?;

        let scripts = list_scripts(&root)?;

        assert_eq!(scripts, vec![root.join("01_first.sh"), root.join("02_second.sh")]);

        fs::remove_dir_all(&root).ok();
        Ok(())
    }
}
