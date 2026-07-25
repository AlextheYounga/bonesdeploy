use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;
use shared::config::{self, build_group_for, build_user_for, is_numbered_shell_script, load_runtime};
use shared::env_build;
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

    let build_env_vars = resolve_build_env(cfg, context)?;
    let deployment_dir = scripts_dir.parent().context("Build scripts directory has no deployment parent")?;
    let build_cache_dir = paths::bonesdeploy_user_cache(&build_user);

    let build_env = deploy_output::BuildScriptEnv {
        project_name: &cfg.project_name,
        build_user: &build_user,
        web_root: &runtime.web_root,
        deployment_dir,
        build_cache_dir: &build_cache_dir,
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

fn resolve_build_env(cfg: &config::Bones, source_context: &Path) -> Result<Vec<(String, String)>> {
    let buildtime = cfg.buildtime.clone();

    let mut env_vars = derived_config_env(cfg)?;
    env_vars.extend(buildtime.extra);

    let env_build = env_build::load(source_context)?;
    for (key, value) in env_build {
        env_vars.push((key, value));
    }

    Ok(env_vars)
}

const DERIVED_ENV_DENYLIST: &[&str] = &[
    "runtime.permissions",
    "runtime.shared",
    "runtime.runtime_user",
    "runtime.runtime_group",
    "runtime.release_group",
    "app.server.host",
    "app.server.port",
    "app.dns",
    "app.ssl_enabled",
];

fn derived_config_env(cfg: &config::Bones) -> Result<Vec<(String, String)>> {
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

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;
    use shared::config::load;

    use super::{derived_config_env, list_scripts, resolve_build_env};

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

    #[test]
    fn derived_environment_exports_scalars_but_not_operational_config() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-derived-env-{}", process::id()));
        fs::create_dir_all(&root)?;
        fs::write(
            root.join("bones.toml"),
            "[app]\nproject_name = \"demo\"\n[app.server]\nhost = \"deploy.example.com\"\n[runtime]\ntemplate = \"nuxt\"\nweb_root = \".output/public\"\nis_static = true\n",
        )?;
        let cfg = load(&root.join("bones.toml"))?;

        let env = derived_config_env(&cfg)?;

        assert!(env.contains(&("BONES_RUNTIME_TEMPLATE".to_string(), "nuxt".to_string())));
        assert!(env.contains(&("BONES_RUNTIME_IS_STATIC".to_string(), "true".to_string())));
        assert!(env.contains(&("BONES_APP_PROJECT_NAME".to_string(), "demo".to_string())), "{env:?}");
        assert!(!env.iter().any(|(key, _)| key == "BONES_APP_SERVER_HOST"));
        assert!(!env.iter().any(|(key, _)| key.starts_with("BONES_APP_DNS_")));
        assert!(!env.iter().any(|(key, _)| key.starts_with("BONES_RUNTIME_SHARED_")));
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn build_env_includes_env_build_values() -> Result<()> {
        let site_root = env::temp_dir().join(format!("bonesremote-env-build-include-{}", process::id()));
        let source = env::temp_dir().join(format!("bonesremote-env-build-src-{}", process::id()));
        let _ = fs::remove_dir_all(&site_root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&site_root)?;
        fs::create_dir_all(&source)?;
        fs::write(
            site_root.join("bones.toml"),
            "[app]\nproject_name = \"demo\"\n[app.server]\nhost = \"deploy.example.com\"\n[runtime]\ntemplate = \"nuxt\"\n",
        )?;
        fs::write(source.join(".env.build"), "NEXT_PUBLIC_API_URL=https://api.example.com\n")?;
        let cfg = load(&site_root.join("bones.toml"))?;

        let env = resolve_build_env(&cfg, &source)?;

        assert!(
            env.contains(&("NEXT_PUBLIC_API_URL".to_string(), "https://api.example.com".to_string())),
            "should include .env.build value: {env:?}"
        );
        fs::remove_dir_all(site_root).ok();
        fs::remove_dir_all(source).ok();
        Ok(())
    }

    #[test]
    fn derived_bones_values_are_present_in_build_env() -> Result<()> {
        let site_root = env::temp_dir().join(format!("bonesremote-env-build-derived-{}", process::id()));
        let source = env::temp_dir().join(format!("bonesremote-env-build-derived-src-{}", process::id()));
        let _ = fs::remove_dir_all(&site_root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&site_root)?;
        fs::create_dir_all(&source)?;
        fs::write(
            site_root.join("bones.toml"),
            "[app]\nproject_name = \"demo\"\n[app.server]\nhost = \"deploy.example.com\"\n[runtime]\ntemplate = \"next\"\n",
        )?;
        let cfg = load(&site_root.join("bones.toml"))?;

        let env = resolve_build_env(&cfg, &source)?;

        assert!(env.contains(&("BONES_RUNTIME_TEMPLATE".to_string(), "next".to_string())));
        assert!(env.contains(&("BONES_APP_PROJECT_NAME".to_string(), "demo".to_string())));
        fs::remove_dir_all(site_root).ok();
        fs::remove_dir_all(source).ok();
        Ok(())
    }

    #[test]
    fn denied_values_remain_absent_in_build_env() -> Result<()> {
        let site_root = env::temp_dir().join(format!("bonesremote-env-build-denied-{}", process::id()));
        let source = env::temp_dir().join(format!("bonesremote-env-build-denied-src-{}", process::id()));
        let _ = fs::remove_dir_all(&site_root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&site_root)?;
        fs::create_dir_all(&source)?;
        fs::write(
            site_root.join("bones.toml"),
            "[app]\nproject_name = \"demo\"\n[app.server]\nhost = \"deploy.example.com\"\nport = \"22\"\n[app.dns]\ndomain = \"app.example.com\"\n[runtime]\ntemplate = \"nuxt\"\n",
        )?;
        let cfg = load(&site_root.join("bones.toml"))?;

        let env = resolve_build_env(&cfg, &source)?;

        assert!(!env.iter().any(|(key, _)| key == "BONES_APP_SERVER_HOST"), "server host should be denied");
        assert!(!env.iter().any(|(key, _)| key == "BONES_APP_SERVER_PORT"), "server port should be denied");
        assert!(!env.iter().any(|(key, _)| key.starts_with("BONES_APP_DNS")), "dns should be denied");
        assert!(!env.iter().any(|(key, _)| key.starts_with("BONES_RUNTIME_SHARED")), "shared should be denied");
        fs::remove_dir_all(site_root).ok();
        fs::remove_dir_all(source).ok();
        Ok(())
    }

    #[test]
    fn derived_bones_values_cannot_be_overridden_by_env_build() -> Result<()> {
        let site_root = env::temp_dir().join(format!("bonesremote-env-build-override-{}", process::id()));
        let source = env::temp_dir().join(format!("bonesremote-env-build-override-src-{}", process::id()));
        let _ = fs::remove_dir_all(&site_root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&site_root)?;
        fs::create_dir_all(&source)?;
        fs::write(
            site_root.join("bones.toml"),
            "[app]\nproject_name = \"demo\"\n[app.server]\nhost = \"deploy.example.com\"\n[runtime]\ntemplate = \"next\"\n",
        )?;
        fs::write(source.join(".env.build"), "BONES_RUNTIME_TEMPLATE=evil\n")?;
        let cfg = load(&site_root.join("bones.toml"))?;

        let result = resolve_build_env(&cfg, &source);

        assert!(result.is_err(), "BONES_* in .env.build should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("reserved"), "error should mention reserved: {err}");
        fs::remove_dir_all(site_root).ok();
        fs::remove_dir_all(source).ok();
        Ok(())
    }

    #[test]
    fn missing_env_build_is_not_an_error() -> Result<()> {
        let site_root = env::temp_dir().join(format!("bonesremote-env-build-missing-{}", process::id()));
        let source = env::temp_dir().join(format!("bonesremote-env-build-missing-src-{}", process::id()));
        let _ = fs::remove_dir_all(&site_root);
        let _ = fs::remove_dir_all(&source);
        fs::create_dir_all(&site_root)?;
        fs::create_dir_all(&source)?;
        fs::write(
            site_root.join("bones.toml"),
            "[app]\nproject_name = \"demo\"\n[app.server]\nhost = \"deploy.example.com\"\n[runtime]\ntemplate = \"nuxt\"\n",
        )?;
        let cfg = load(&site_root.join("bones.toml"))?;

        let env = resolve_build_env(&cfg, &source)?;

        assert!(env.contains(&("BONES_RUNTIME_TEMPLATE".to_string(), "nuxt".to_string())));
        assert!(!env.iter().any(|(key, _)| key == "NEXT_PUBLIC_API_URL"), "no .env.build means no extra vars");
        fs::remove_dir_all(site_root).ok();
        fs::remove_dir_all(source).ok();
        Ok(())
    }
}
