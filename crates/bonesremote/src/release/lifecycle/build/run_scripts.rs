use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::{
    self, build_env, build_group_for, build_timeout_seconds, build_user_for, is_numbered_shell_script, variables,
};
use bonesdeploy_core::paths;
use serde_json::Value;

use super::build_user::BuildScriptEnv;
use super::container::BuildContainer;
use super::ownership;

pub fn run(snapshot: &super::super::DeploymentSnapshot, context: &Path) -> Result<()> {
    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let cfg = &snapshot.config;
    let build_user = build_user_for(&cfg.project_name);
    let build_group = build_group_for(&cfg.project_name);
    ownership::chown_tree_to_user(context, &build_user, &build_group)?;

    let scripts_dir = snapshot.deployment_dir.join(paths::DEPLOYMENT_BUILD_DIR);
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

    let build_env_vars = resolve_build_env(cfg, context)?;
    let deployment_dir = scripts_dir.parent().context("Build scripts directory has no deployment parent")?;
    let build_cache_dir = paths::bonesdeploy_user_cache(&build_user);

    let build_env = BuildScriptEnv {
        project_name: &cfg.project_name,
        build_user: &build_user,
        build_group: &build_group,
        web_root: &cfg.runtime.web_root,
        deployment_dir,
        build_cache_dir: &build_cache_dir,
        build_env_vars: &build_env_vars,
        script_timeout_seconds: build_timeout_seconds(cfg),
    };
    let mut container = BuildContainer::start(context, &build_env)?;

    let logs_dir = paths::bonesremote_site_logs(&snapshot.site);
    fs::create_dir_all(&logs_dir).with_context(|| format!("Failed to create logs directory {}", logs_dir.display()))?;

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running build script {script_name}...");

        let status = container
            .run_script(&script, &logs_dir.join(format!("{script_name}.log")))
            .with_context(|| format!("Failed to execute build script {}", script.display()))?;

        if !status.success() {
            bail!("Build script {script_name} exited with status {status}");
        }
    }

    container.remove()?;

    Ok(())
}

pub fn resolve_build_env(cfg: &config::Bones, source_context: &Path) -> Result<Vec<(String, String)>> {
    let mut env_vars = derived_config_env(cfg)?;

    let env_build = build_env::load(source_context)?;
    for (key, value) in env_build {
        if CONTAINER_ENV_DENYLIST.contains(&key.as_str()) {
            bail!(".env.build variable `{key}` is reserved for the build container contract");
        }
        env_vars.push((key, value));
    }

    Ok(env_vars)
}

const DERIVED_ENV_DENYLIST: &[&str] = &[
    "app.remote_name",
    "app.ssh_user",
    "app.host",
    "app.port",
    "app.branch",
    "app.repo_path",
    "app.project_root",
    "runtime.permissions",
    "runtime.backend",
    "runtime.node_version",
    "app.server.host",
    "app.server.port",
    "app.dns",
    "build.timeout_seconds",
];

const CONTAINER_ENV_DENYLIST: &[&str] = variables::CONTAINER_CONTROLLED;

pub fn derived_config_env(cfg: &config::Bones) -> Result<Vec<(String, String)>> {
    let value = serde_json::to_value(cfg).context("Failed to serialize configuration for build environment")?;
    let mut values = Vec::new();
    flatten_scalars(&value, &mut Vec::new(), &mut values);
    Ok(values)
}

fn flatten_scalars<'a>(value: &'a Value, path: &mut Vec<&'a str>, values: &mut Vec<(String, String)>) {
    match value {
        Value::Object(entries) => {
            for (key, value) in entries {
                path.push(key);
                flatten_scalars(value, path, values);
                path.pop();
            }
        }
        Value::String(value) => add_scalar(path, value, values),
        Value::Bool(value) => add_scalar(path, &value.to_string(), values),
        Value::Number(value) => add_scalar(path, &value.to_string(), values),
        Value::Array(_) | Value::Null => {}
    }
}

fn add_scalar(path: &[&str], value: &str, values: &mut Vec<(String, String)>) {
    let path_name = path.join(".");
    if path.is_empty()
        || DERIVED_ENV_DENYLIST
            .iter()
            .any(|denied| path_name == *denied || path_name.starts_with(&format!("{denied}.")))
    {
        return;
    }

    let name = format!("BONES_{}", path.join("_").to_ascii_uppercase());
    values.push((name, value.to_string()));
}

pub fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|name| name.to_str()).is_some_and(is_numbered_shell_script) {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}
