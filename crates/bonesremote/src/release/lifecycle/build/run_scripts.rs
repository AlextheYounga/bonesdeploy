use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config::{self, build_group_for, build_user_for, extract_env_vars, load_buildtime, load_runtime};
use shared::paths;

use super::ownership;
use crate::release::script_runner as deploy_output;

pub(super) fn run(site: &str, context: &Path, cfg: &config::Bones) -> Result<()> {
    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let build_user = build_user_for(&cfg.project_name);
    let build_group = build_group_for(&cfg.project_name);
    ownership::chown_tree_to_user(context, &build_user, &build_group)?;

    let scripts_dir = paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR).join(paths::DEPLOYMENT_BUILD_DIR);
    if !scripts_dir.is_dir() {
        println!(
            "No deployment scripts at {}; running build steps directly on the exported source tree.",
            scripts_dir.display()
        );
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found at {}; skipping build.", scripts_dir.display());
        return Ok(());
    }

    let runtime = load_runtime(&paths::bonesremote_site_root(site))
        .with_context(|| format!("Failed to load runtime configuration for {site}"))?;

    let build_env_vars = resolve_build_env(site, cfg)?;

    let build_env = deploy_output::BuildScriptEnv {
        project_name: &cfg.project_name,
        build_user: &build_user,
        web_root: &runtime.web_root,
        build_env_vars: &build_env_vars,
    };
    let mut container = deploy_output::BuildContainer::start(context, &build_env)?;

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running build script {script_name}...");

        let status = container
            .run_script(&script, &context.join(format!("{script_name}.log")))
            .with_context(|| format!("Failed to execute build script {}", script.display()))?;

        if !status.success() {
            bail!("Build script {script_name} exited with status {status}");
        }
    }

    container.remove()?;

    Ok(())
}

fn resolve_build_env(site: &str, cfg: &config::Bones) -> Result<Vec<(String, String)>> {
    let buildtime = load_buildtime(&paths::bonesremote_site_root(site))?.unwrap_or_default();

    let mut env_vars: Vec<(String, String)> = buildtime.extra.into_iter().collect();

    if buildtime.vars.is_empty() {
        return Ok(env_vars);
    }

    let env_path = Path::new(&cfg.project_root).join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let env_content = fs::read_to_string(&env_path).with_context(|| {
        format!("buildtime.toml requests vars but {}. Run `bonesdeploy secrets push` first.", env_path.display())
    })?;

    let vars = extract_env_vars(&env_content, &buildtime.vars);

    for name in &buildtime.vars {
        if !vars.iter().any(|(k, _)| k == name) {
            bail!("buildtime.toml requests `{name}` but it was not found in {}.", env_path.display());
        }
    }

    env_vars.extend(vars);
    Ok(env_vars)
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_script(&path) {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

fn is_script(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let bytes = name.as_bytes();
    bytes.len() > 6
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2] == b'_'
        && path.extension().is_some_and(|extension| extension == "sh")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::list_scripts;

    #[test]
    fn list_scripts_only_includes_numbered_shell_scripts() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-build-list-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("02_second.sh"), "")?;
        fs::write(root.join("01_first.sh"), "")?;
        fs::write(root.join("README.md"), "# Build Scripts")?;
        fs::write(root.join("1_not_ordered.sh"), "")?;
        fs::write(root.join("01-not-a-script.sh"), "")?;

        let scripts = list_scripts(&root)?;

        assert_eq!(scripts, vec![root.join("01_first.sh"), root.join("02_second.sh")]);

        fs::remove_dir_all(&root).ok();
        Ok(())
    }
}
