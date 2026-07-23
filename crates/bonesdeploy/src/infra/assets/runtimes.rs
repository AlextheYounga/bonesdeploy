use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::str;

use anyhow::{Context, Result, anyhow, bail};
use rust_embed::Embed;
use serde_json::{Map, Value};

use shared::config::Runtime;
use shared::paths;

use super::{kit, write_asset};

#[derive(Embed)]
#[folder = "./assets/runtimes/"]
struct RuntimeAssets;

pub fn runtime_names() -> Vec<String> {
    RuntimeAssets::iter()
        .filter_map(|path| path.split('/').next().map(str::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn base_runtime_defaults() -> Result<Map<String, Value>> {
    serde_json::to_value(Runtime::default())
        .ok()
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| anyhow!("Failed to serialize base runtime defaults"))
}

pub fn runtime_defaults(runtime: &str) -> Result<Map<String, Value>> {
    let asset_path = format!("{runtime}/bones.toml");
    runtime_defaults_from_bytes(&asset_path, RuntimeAssets::get(&asset_path).map(|asset| asset.data))
}

pub fn scaffold_runtime_deployment(runtime: &str, bones_dir: &Path) -> Result<()> {
    let deploy_dir = bones_dir.join(paths::DEPLOYMENT_DIR);
    if deploy_dir.exists() {
        fs::remove_dir_all(&deploy_dir)
            .with_context(|| format!("Failed to clear deployment dir: {}", deploy_dir.display()))?;
    }
    kit::scaffold_deployment_functions(bones_dir)?;
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_DEPLOYMENT_DIR)
}

pub fn scaffold_runtime_env_build(runtime: &str, project_root: &Path) -> Result<()> {
    let Some(content) = runtime_env_build_content(runtime) else {
        return Ok(());
    };

    let destination = project_root.join(paths::ENV_BUILD_FILE);
    if destination.exists() {
        return Ok(());
    }

    fs::write(&destination, content).with_context(|| format!("Failed to write {}", destination.display()))?;
    Ok(())
}

fn runtime_env_build_content(runtime: &str) -> Option<String> {
    let lines = match runtime {
        "django" | "rails" => [
            "# Committed, non-secret values used while building this project.",
            "# Pin Node when this project includes a frontend build.",
            "NODE_VERSION=",
        ]
        .as_slice(),
        "laravel" => {
            ["# Committed, non-secret values used while building this project.", "NODE_VERSION=", "PHP_VERSION=8.5"]
                .as_slice()
        }
        "next" => [
            "# Committed, non-secret values used while building this project.",
            "NODE_VERSION=",
            "NEXT_PUBLIC_API_URL=",
            "NEXT_PUBLIC_SITE_NAME=",
        ]
        .as_slice(),
        "nuxt" => [
            "# Committed, non-secret values used while building this project.",
            "NODE_VERSION=",
            "NUXT_PUBLIC_SITE_URL=",
        ]
        .as_slice(),
        "sveltekit" | "vue" => {
            ["# Committed, non-secret values used while building this project.", "NODE_VERSION="].as_slice()
        }
        _ => return None,
    };

    Some(format!("{}\n", lines.join("\n")))
}

fn scaffold_runtime_assets(runtime: &str, bones_dir: &Path, asset_prefix: &str) -> Result<()> {
    let runtime_prefix = format!("{runtime}/");

    for file_path in RuntimeAssets::iter() {
        let Some(stripped) = file_path.strip_prefix(&runtime_prefix) else {
            continue;
        };

        if !stripped.starts_with(asset_prefix) {
            continue;
        }

        let Some(asset) = RuntimeAssets::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, stripped, asset.data.as_ref())?;
    }

    Ok(())
}

fn runtime_defaults_from_bytes(asset_path: &str, bytes: Option<impl AsRef<[u8]>>) -> Result<Map<String, Value>> {
    let Some(bytes) = bytes else {
        bail!("Missing embedded runtime defaults at {asset_path}");
    };

    let content =
        str::from_utf8(bytes.as_ref()).with_context(|| format!("Embedded asset {asset_path} is not valid UTF-8"))?;
    let toml_value: toml::Value = toml::from_str(content)
        .with_context(|| format!("Failed to parse embedded runtime defaults at {asset_path}"))?;
    let runtime = toml_value
        .get("runtime")
        .cloned()
        .ok_or_else(|| anyhow!("Embedded runtime defaults at {asset_path} are missing [runtime]"))?;
    let json_value = serde_json::to_value(runtime)
        .with_context(|| format!("Failed to convert embedded runtime defaults at {asset_path} to JSON"))?;

    json_value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow!("Embedded runtime defaults at {asset_path} are not a TOML table"))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use std::fs;

    use super::{
        Runtime, RuntimeAssets, runtime_defaults, runtime_env_build_content, runtime_names, scaffold_runtime_env_build,
    };

    #[test]
    fn next_runtime_includes_the_build_script() {
        assert!(RuntimeAssets::get("next/deployment/build/02_run_build.sh").is_some());
    }

    #[test]
    fn every_runtime_has_a_build_environment_example() {
        for runtime in runtime_names() {
            assert!(runtime_env_build_content(&runtime).is_some(), "{runtime} is missing .env.build");
        }
    }

    #[test]
    fn runtime_build_environment_example_does_not_overwrite_existing_file() -> Result<()> {
        let root = std::env::temp_dir().join(format!("bonesdeploy-runtime-env-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        scaffold_runtime_env_build("next", &root)?;
        let generated = fs::read_to_string(root.join(".env.build"))?;
        assert!(generated.contains("NEXT_PUBLIC_API_URL="));

        fs::write(root.join(".env.build"), "CUSTOM=value\n")?;
        scaffold_runtime_env_build("next", &root)?;
        assert_eq!(fs::read_to_string(root.join(".env.build"))?, "CUSTOM=value\n");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn runtime_pnpm_installs_use_the_persistent_store() {
        for runtime in runtime_names() {
            let path = format!("{runtime}/deployment/build/02_run_build.sh");
            let Some(asset) = RuntimeAssets::get(&path) else {
                continue;
            };
            let script = String::from_utf8_lossy(asset.data.as_ref());
            if script.contains("pnpm install") {
                assert!(
                    script.contains("--store-dir \"$PNPM_STORE_DIR\""),
                    "{path} must use the persistent pnpm store"
                );
            }
        }

        let laravel = RuntimeAssets::get("laravel/deployment/build/03_build_frontend.sh")
            .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
            .unwrap_or_default();
        if laravel.contains("pnpm install") {
            assert!(laravel.contains("--store-dir \"$PNPM_STORE_DIR\""));
        }
    }

    #[test]
    fn runtime_defaults_fit_the_single_file_schema() -> Result<()> {
        for runtime in runtime_names() {
            let defaults = runtime_defaults(&runtime)?;
            let config: Runtime = serde_json::from_value(serde_json::Value::Object(defaults))?;
            assert_eq!(config.template, runtime);
        }
        Ok(())
    }

    #[test]
    fn runtime_answers_accept_boolean_template_settings() -> Result<()> {
        let mut answers = runtime_defaults("nuxt")?;
        answers.insert("static".into(), serde_json::Value::Bool(true));

        let config: Runtime = serde_json::from_value(serde_json::Value::Object(answers))?;
        assert_eq!(config.extra.get("static").map(ToString::to_string).as_deref(), Some("true"));
        assert!(toml::to_string(&config)?.contains("static = true"));
        Ok(())
    }
}
