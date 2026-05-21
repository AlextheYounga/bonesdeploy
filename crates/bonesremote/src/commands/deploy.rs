use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::release_state;

use super::activate_release;
use super::drop_failed_release;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let runtime_path = release_state::release_dir(&cfg, &release_name);
    let build_root = release_state::build_root(&cfg);
    let deployment_dir = Path::new(&cfg.data.git_dir).join("bones").join("deployment");

    if !runtime_path.exists() {
        bail!("Staged runtime directory does not exist: {}", runtime_path.display());
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
            println!("Running {script_name}...");

            let status = Command::new("bash")
                .arg(&script)
                .current_dir(&build_root)
                .status()
                .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;

            if !status.success() {
                println!("Deployment script {script_name} failed.");
                drop_failed_release::run(config_path)
                    .with_context(|| "Failed to drop staged release after deployment script failure")?;
                bail!("Deployment script {script_name} failed with status {status}");
            }
        }

        println!("All deployment scripts completed.");
    }

    publish_runtime_tree(&build_root, &runtime_path)?;

    activate_release::run(config_path)
}

fn publish_runtime_tree(build_root: &Path, runtime_path: &Path) -> Result<()> {
    clear_directory(runtime_path)?;

    let copy_source = build_root.join(".");
    let status = Command::new("cp").arg("-a").arg(&copy_source).arg(runtime_path).status().with_context(|| {
        format!("Failed to copy build workspace {} to runtime tree {}", build_root.display(), runtime_path.display())
    })?;

    if !status.success() {
        bail!(
            "Failed to publish runtime tree from {} to {}: status {status}",
            build_root.display(),
            runtime_path.display()
        );
    }

    println!("Published runtime tree: {}", runtime_path.display());
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
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{clear_directory, list_deployment_scripts, publish_runtime_tree};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&path).unwrap_or_else(|error| panic!("failed to create temp dir: {error}"));
        path
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| panic!("failed to create parent: {error}"));
        }
        fs::write(path, content).unwrap_or_else(|error| panic!("failed to write file: {error}"));
    }

    // Ensures publish prep always starts from a clean runtime dir with no stale artifacts.
    #[test]
    fn clear_directory_removes_all_direct_children() {
        let root = temp_dir("bonesremote_deploy_clear");
        write_file(&root.join("file.txt"), "hello");
        write_file(&root.join("nested/inner.txt"), "world");

        clear_directory(&root).unwrap_or_else(|error| panic!("clear_directory failed: {error}"));

        assert!(fs::read_dir(&root).unwrap_or_else(|error| panic!("failed to read dir: {error}")).next().is_none());

        fs::remove_dir_all(root).ok();
    }

    // Ensures deployment scripts execute in deterministic order and ignore non-script directories.
    #[test]
    fn list_deployment_scripts_returns_sorted_files_only() {
        let deployment_dir = temp_dir("bonesremote_deploy_scripts");
        write_file(&deployment_dir.join("20_restart.sh"), "#!/usr/bin/env bash\n");
        write_file(&deployment_dir.join("10_build.sh"), "#!/usr/bin/env bash\n");
        fs::create_dir_all(deployment_dir.join("ignored_dir"))
            .unwrap_or_else(|error| panic!("failed to create dir: {error}"));

        let scripts = list_deployment_scripts(&deployment_dir)
            .unwrap_or_else(|error| panic!("list_deployment_scripts failed: {error}"));
        let script_names: Vec<String> = scripts
            .into_iter()
            .map(|path| {
                path.file_name().map_or_else(|| panic!("missing file name"), |name| name.to_string_lossy().to_string())
            })
            .collect();

        assert_eq!(script_names, vec!["10_build.sh", "20_restart.sh"]);

        fs::remove_dir_all(deployment_dir).ok();
    }

    // Verifies release publish is a full replacement copy, preserving expected hidden/runtime files.
    #[test]
    fn publish_runtime_tree_replaces_runtime_contents_with_build_workspace() {
        let root = temp_dir("bonesremote_deploy_publish");
        let build_root = root.join("build_workspace");
        let runtime_root = root.join("runtime_release");
        fs::create_dir_all(&build_root).unwrap_or_else(|error| panic!("failed to create build_root: {error}"));
        fs::create_dir_all(&runtime_root).unwrap_or_else(|error| panic!("failed to create runtime_root: {error}"));

        write_file(&build_root.join("public/index.html"), "<h1>ok</h1>");
        write_file(&build_root.join(".env.example"), "KEY=value");
        write_file(&runtime_root.join("stale.txt"), "old");

        publish_runtime_tree(&build_root, &runtime_root)
            .unwrap_or_else(|error| panic!("publish_runtime_tree failed: {error}"));

        assert!(!runtime_root.join("stale.txt").exists());
        assert_eq!(
            fs::read_to_string(runtime_root.join("public/index.html"))
                .unwrap_or_else(|error| panic!("failed to read published file: {error}")),
            "<h1>ok</h1>"
        );
        assert_eq!(
            fs::read_to_string(runtime_root.join(".env.example"))
                .unwrap_or_else(|error| panic!("failed to read published hidden file: {error}")),
            "KEY=value"
        );

        fs::remove_dir_all(root).ok();
    }
}
