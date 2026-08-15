use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::str;

use anyhow::{Context, Result, anyhow, bail};
use rust_embed::Embed;
use serde_json::{Map, Value};

use bonesdeploy_core::config::Runtime;
use bonesdeploy_core::paths;

use super::{kit, write_asset};
use crate::frameworks;

#[derive(Embed)]
#[folder = "./assets/frameworks/"]
struct FrameworkAssets;

pub fn framework_names() -> Vec<String> {
    FrameworkAssets::iter()
        .filter_map(|path| path.split('/').next().map(str::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn base_framework_defaults() -> Result<Map<String, Value>> {
    serde_json::to_value(Runtime::default())
        .ok()
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| anyhow!("Failed to serialize base framework defaults"))
}

pub fn framework_defaults(framework: &str) -> Result<Map<String, Value>> {
    let asset_path = format!("{framework}/bones.toml");
    framework_defaults_from_bytes(&asset_path, FrameworkAssets::get(&asset_path).map(|asset| asset.data))
}

pub fn scaffold_framework_project(framework: &str, bones_dir: &Path) -> Result<()> {
    let deploy_dir = bones_dir.join(paths::DEPLOYMENT_DIR);
    if deploy_dir.exists() {
        fs::remove_dir_all(&deploy_dir)
            .with_context(|| format!("Failed to clear deployment dir: {}", deploy_dir.display()))?;
    }
    kit::scaffold_deployment_functions(bones_dir)?;
    scaffold_framework_assets(framework, bones_dir, paths::KIT_DEPLOYMENT_DIR)?;
    Ok(())
}

pub fn scaffold_framework_env_build(framework: &str, project_root: &Path, framework_config: &Runtime) -> Result<()> {
    let Some(content) = frameworks::build_environment_example(framework, framework_config) else {
        return Ok(());
    };

    let destination = project_root.join(paths::ENV_BUILD_FILE);
    if destination.exists() {
        return Ok(());
    }

    fs::write(&destination, content).with_context(|| format!("Failed to write {}", destination.display()))?;
    Ok(())
}

fn scaffold_framework_assets(framework: &str, bones_dir: &Path, asset_prefix: &str) -> Result<()> {
    let framework_prefix = format!("{framework}/");

    for file_path in FrameworkAssets::iter() {
        let Some(stripped) = file_path.strip_prefix(&framework_prefix) else {
            continue;
        };

        if !stripped.starts_with(asset_prefix) {
            continue;
        }

        let Some(asset) = FrameworkAssets::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, stripped, asset.data.as_ref())?;
    }

    Ok(())
}

fn framework_defaults_from_bytes(asset_path: &str, bytes: Option<impl AsRef<[u8]>>) -> Result<Map<String, Value>> {
    let Some(bytes) = bytes else {
        bail!("Missing embedded framework defaults at {asset_path}");
    };

    let content =
        str::from_utf8(bytes.as_ref()).with_context(|| format!("Embedded asset {asset_path} is not valid UTF-8"))?;
    let toml_value: toml::Value = toml::from_str(content)
        .with_context(|| format!("Failed to parse embedded framework defaults at {asset_path}"))?;
    let framework = toml_value
        .get("runtime")
        .cloned()
        .ok_or_else(|| anyhow!("Embedded framework defaults at {asset_path} are missing [runtime]"))?;
    let json_value = serde_json::to_value(framework)
        .with_context(|| format!("Failed to convert embedded framework defaults at {asset_path} to JSON"))?;

    json_value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow!("Embedded framework defaults at {asset_path} are not a TOML table"))
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use crate::frameworks;
    use anyhow::Result;

    use std::env;
    use std::fs;
    use std::process;

    use super::{FrameworkAssets, Runtime, framework_defaults, framework_names, scaffold_framework_env_build};

    #[test]
    fn next_framework_includes_the_build_script() {
        assert!(FrameworkAssets::get("next/deployment/build/02_run_build.sh").is_some());
    }

    #[test]
    fn framework_assets_do_not_duplicate_canonical_infrastructure() {
        assert!(FrameworkAssets::iter().all(|path| !path.split('/').any(|part| part == "infra")));
    }

    #[test]
    fn nuxt_build_selects_generate_or_build_from_static_mode() {
        let script = FrameworkAssets::get("nuxt/deployment/build/02_run_build.sh")
            .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
            .unwrap_or_default();
        assert!(script.contains("BONES_RUNTIME_IS_STATIC"));
        assert!(script.contains("corepack pnpm \"$command\""));
        assert!(script.contains("npm run \"$command\""));
    }

    #[test]
    fn every_framework_has_a_build_environment_example() {
        for framework in framework_names() {
            if let Some(content) = frameworks::build_environment_example(&framework, &Runtime::default()) {
                assert!(content.contains("Committed, non-secret"), "{framework} must include build environment header");
            } else {
                assert!(false, "{framework} is missing .env.build");
            }
        }
    }

    #[test]
    fn framework_build_environment_example_does_not_overwrite_existing_file() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesdeploy-framework-env-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        scaffold_framework_env_build("next", &root, &Runtime::default())?;
        let generated = fs::read_to_string(root.join(".env.build"))?;
        assert!(generated.contains("NEXT_PUBLIC_API_URL="));

        fs::write(root.join(".env.build"), "CUSTOM=value\n")?;
        scaffold_framework_env_build("next", &root, &Runtime::default())?;
        assert_eq!(fs::read_to_string(root.join(".env.build"))?, "CUSTOM=value\n");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn framework_pnpm_installs_use_the_persistent_store() {
        for framework in framework_names() {
            let path = format!("{framework}/deployment/build/02_run_build.sh");
            let Some(asset) = FrameworkAssets::get(&path) else {
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

        let laravel = FrameworkAssets::get("laravel/deployment/build/03_build_frontend.sh")
            .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
            .unwrap_or_default();
        if laravel.contains("pnpm install") {
            assert!(laravel.contains("--store-dir \"$PNPM_STORE_DIR\""));
        }
    }

    #[test]
    fn laravel_prepare_only_mutates_candidate_and_required_framework_state() {
        let script = FrameworkAssets::get("laravel/deployment/prepare/01_prepare_laravel.sh")
            .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
            .unwrap_or_default();

        assert!(script.contains("php artisan optimize"));
        for command in ["optimize:clear", "package:discover", "queue:restart", "artisan up"] {
            assert!(!script.contains(command), "prepare must not run {command}");
        }
    }

    #[test]
    fn django_validates_before_mutating_framework_state() {
        let script = FrameworkAssets::get("django/deployment/prepare/01_prepare_django.sh")
            .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
            .unwrap_or_default();
        let check = script.find("manage.py check --deploy").expect("Django deployment check");
        let migrate = script.find("manage.py migrate").expect("Django migration");
        assert!(check < migrate);
    }

    #[test]
    fn framework_defaults_fit_the_single_file_schema() -> Result<()> {
        for framework in framework_names() {
            let defaults = framework_defaults(&framework)?;
            let config: Runtime = serde_json::from_value(serde_json::Value::Object(defaults))?;
            assert_eq!(config.template, framework);
        }
        Ok(())
    }

    #[test]
    fn framework_answers_accept_boolean_template_settings() -> Result<()> {
        let mut answers = framework_defaults("nuxt")?;
        answers.insert("static".into(), serde_json::Value::Bool(true));

        let config: Runtime = serde_json::from_value(serde_json::Value::Object(answers))?;
        assert_eq!(config.extra.get("static").map(ToString::to_string).as_deref(), Some("true"));
        assert!(toml::to_string(&config)?.contains("static = true"));
        Ok(())
    }
}
