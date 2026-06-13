use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::release_state;

#[path = "deploy_output.rs"]
mod deploy_output;

use super::activate_release;
use super::drop_failed_release;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let release_path = release_state::release_dir(&cfg, &release_name);
    let build_root = release_state::build_root(&cfg);
    let paths = cfg.data.deployment_paths();
    let deployment_dir = PathBuf::from(&paths.repo_deployment);

    if !release_path.exists() {
        bail!("Staged release directory does not exist: {}", release_path.display());
    }

    if !build_root.exists() {
        bail!("Build workspace does not exist: {}", build_root.display());
    }

    let scripts = list_deployment_scripts(&deployment_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found. Skipping deploy scripts.");
    } else {
        for script in scripts {
            let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
            let log_path = deploy_output::deployment_log_path(&paths, &release_name, script_name);
            println!("Running {script_name}...");
            println!("Log: {}", log_path.display());

            let status = deploy_output::run_deployment_script(
                &script,
                &build_root,
                &log_path,
                &deploy_output::ScriptEnv {
                    project_name: &cfg.data.project_name,
                    project_root: &cfg.data.project_root,
                    repo_path: &cfg.data.repo_path,
                    web_root: &cfg.data.web_root,
                },
            )
            .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;

            if !status.success() {
                println!("Deployment script {script_name} failed.");
                println!("Log: {}", log_path.display());
                drop_failed_release::run(config_path)
                    .with_context(|| "Failed to drop staged release after deployment script failure")?;
                bail!("Deployment script {script_name} failed with status {status}");
            }
        }

        println!("All deployment scripts completed.");
    }

    publish_release_tree(&build_root, &release_path)?;

    activate_release::run(config_path)
}

fn publish_release_tree(build_root: &Path, release_path: &Path) -> Result<()> {
    clear_directory(release_path)?;

    let copy_source = build_root.join(".");
    let status = Command::new("cp").arg("-a").arg(&copy_source).arg(release_path).status().with_context(|| {
        format!("Failed to copy build workspace {} to release tree {}", build_root.display(), release_path.display())
    })?;

    if !status.success() {
        bail!(
            "Failed to publish release tree from {} to {}: status {status}",
            build_root.display(),
            release_path.display()
        );
    }

    println!("Published release tree: {}", release_path.display());
    Ok(())
}

fn clear_directory(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read directory {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type().with_context(|| format!("Failed to inspect {}", entry_path.display()))?;

        if file_type.is_dir() {
            fs::remove_dir_all(&entry_path)
                .with_context(|| format!("Failed to remove directory {}", entry_path.display()))?;
        } else {
            fs::remove_file(&entry_path).with_context(|| format!("Failed to remove {}", entry_path.display()))?;
        }
    }

    Ok(())
}

fn list_deployment_scripts(deployment_dir: &Path) -> Result<Vec<PathBuf>> {
    if !deployment_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut scripts = Vec::new();
    for entry in fs::read_dir(deployment_dir)
        .with_context(|| format!("Failed to read deployment directory {}", deployment_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            scripts.push(entry.path());
        }
    }

    scripts.sort();
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::{Result, anyhow};

    use super::{clear_directory, list_deployment_scripts, publish_release_tree};

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn write_file(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Removes all direct children of a directory without removing the directory itself.
    #[test]
    fn clear_directory_removes_all_direct_children() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_clear")?;
        write_file(&root.join("file.txt"), "hello")?;
        write_file(&root.join("nested/inner.txt"), "world")?;

        clear_directory(&root)?;

        assert!(fs::read_dir(&root)?.next().is_none());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    /// Returns deployment script files sorted and excludes subdirectories.
    #[test]
    fn list_deployment_scripts_returns_sorted_files_only() -> Result<()> {
        let deployment_dir = temp_dir("bonesremote_deploy_scripts")?;
        write_file(&deployment_dir.join("20_restart.sh"), "#!/usr/bin/env bash\n")?;
        write_file(&deployment_dir.join("10_build.sh"), "#!/usr/bin/env bash\n")?;
        fs::create_dir_all(deployment_dir.join("ignored_dir"))?;

        let scripts = list_deployment_scripts(&deployment_dir)?;
        let script_names: Result<Vec<String>> = scripts
            .into_iter()
            .map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .ok_or_else(|| anyhow!("missing file name"))
            })
            .collect();

        assert_eq!(script_names?, vec!["10_build.sh", "20_restart.sh"]);

        fs::remove_dir_all(deployment_dir).ok();
        Ok(())
    }

    /// Replaces the release tree contents with a fresh copy from the build workspace.
    #[test]
    fn publish_release_tree_replaces_release_contents_with_build_workspace() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_publish")?;
        let build_root = root.join("build_workspace");
        let release_root = root.join("release_tree");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&release_root)?;

        write_file(&build_root.join("public/index.html"), "<h1>ok</h1>")?;
        write_file(&build_root.join(".env.example"), "KEY=value")?;
        write_file(&release_root.join("stale.txt"), "old")?;

        publish_release_tree(&build_root, &release_root)?;

        assert!(!release_root.join("stale.txt").exists());
        assert_eq!(fs::read_to_string(release_root.join("public/index.html"))?, "<h1>ok</h1>");
        assert_eq!(fs::read_to_string(release_root.join(".env.example"))?, "KEY=value");

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
