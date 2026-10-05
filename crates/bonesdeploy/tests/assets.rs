use std::fs;

use anyhow::{Context, Result};
use bonesdeploy::frameworks::Framework;
use bonesdeploy::infra::assets::frameworks::{
    base_framework_defaults, framework_asset, framework_asset_paths, framework_defaults, framework_names,
    scaffold_framework_env_build, scaffold_framework_project,
};
use bonesdeploy::infra::assets::kit::kit_asset;
use bonesdeploy::infra::assets::skill::doc_names;
use bonesdeploy_core::config::Runtime;

fn asset_text(path: &str) -> String {
    framework_asset(path).map_or_else(String::new, |bytes| String::from_utf8_lossy(&bytes).into_owned())
}

#[test]
fn framework_assets_include_expected_build_content() {
    assert!(framework_asset("angular/deployment/build/02_run_build.sh").is_some());
    assert!(framework_asset("next/deployment/build/02_run_build.sh").is_some());
    let nuxt = asset_text("nuxt/deployment/build/02_run_build.sh");
    assert!(nuxt.contains("BONES_RUNTIME_IS_STATIC"));
    assert!(nuxt.contains("corepack pnpm \"$command\""));
    assert!(nuxt.contains("npm run \"$command\""));
    assert!(
        kit_asset("deployment/functions.sh")
            .is_some_and(|functions| { !String::from_utf8_lossy(&functions).contains("BONES_RUNTIME_NODE_VERSION") })
    );
}

#[test]
fn framework_builds_keep_runtime_outputs_and_prune_build_only_content() -> Result<()> {
    let django = asset_text("django/deployment/build/02_run_build.sh");
    assert!(!django.contains("-m venv"));
    assert!(django.contains("python_enable_toolchain"));
    assert!(django.contains("python -m pip install"));
    assert!(django.contains("--target \"$packages_dir\""));
    assert!(django.contains("python --version >.bonesdeploy-python-runtime"));
    assert!(django.contains(".bonesdeploy/runtimes/python\" -m gunicorn \"$@\""));
    assert!(!django.contains("BONES_RUNTIME_PYTHON_VERSION"));
    assert!(django.contains("rm -rf deployment/build"));

    let django_prepare = asset_text("django/deployment/prepare/01_prepare_django.sh");
    assert!(!django_prepare.contains("-m venv"));
    assert!(!django_prepare.contains("pip install"));
    assert!(django_prepare.contains(".bonesdeploy-python-runtime"));
    assert!(django_prepare.contains(".bonesdeploy/runtimes/python/bin/python"));

    let laravel = asset_text("laravel/deployment/build/03_build_frontend.sh");
    assert!(laravel.contains("rm -rf node_modules deployment/build"));
    let rails = asset_text("rails/deployment/build/02_run_build.sh");
    assert!(rails.contains("gem install bundler --version \"$bundler_version\""));
    assert!(rails.contains(".bonesdeploy-ruby-version"));
    assert!(rails.contains("\"$RUBY_BIN\" \"$packaged_bundler\" install"));
    assert!(rails.contains("BUNDLE_DEPLOYMENT=\"true\""));
    assert!(rails.contains("BUNDLE_WITHOUT=\"development:test\""));
    assert!(!rails.contains("--deployment"));
    assert!(!rails.contains("--without"));
    assert!(rails.contains("rm -rf node_modules tmp/cache deployment/build"));
    assert!(!rails.contains("rm -rf node_modules tmp/cache vendor/bundle"));

    let rails_prepare = asset_text("rails/deployment/prepare/01_prepare_rails.sh");
    let runtime = rails_prepare
        .rfind("\tvalidate_runtime")
        .context("Rails prepare must validate the provisioned Ruby before checking the bundle")?;
    let check = rails_prepare
        .find("\"$SITE_RUBY\" \"$PACKAGED_BUNDLER\" check")
        .context("Rails prepare must check the packaged bundle")?;
    let migration_skip =
        rails_prepare.find("BONES_RAILS_SKIP_MIGRATIONS").context("Rails prepare must support skipping migrations")?;
    let migrate = rails_prepare
        .find("\"$SITE_RUBY\" \"$PACKAGED_BUNDLER\" exec rails db:migrate")
        .context("Rails prepare must run migrations through the target bundle")?;
    assert!(runtime < check);
    assert!(check < migration_skip);
    assert!(check < migrate);
    assert!(!rails_prepare.contains("bundle install"));
    assert!(rails_prepare.contains(".bonesdeploy/runtimes/ruby/bin/ruby"));
    assert!(rails_prepare.contains("vendor/bundle/bundler/bin/bundle"));
    assert!(rails_prepare.contains("BUNDLE_DEPLOYMENT=\"true\""));
    assert!(rails_prepare.contains("BUNDLE_PATH=\"vendor/bundle\""));
    assert!(rails_prepare.contains("BUNDLE_WITHOUT=\"development:test\""));
    assert!(!rails_prepare.contains("--deployment"));
    assert!(!rails_prepare.contains("--without"));

    let next = asset_text("next/deployment/build/02_run_build.sh");
    assert!(next.contains(".next/standalone"));
    assert!(next.contains("rm -rf .next/cache .next/static node_modules"));
    assert!(next.contains("rm -rf .next node_modules"));

    let nuxt = asset_text("nuxt/deployment/build/02_run_build.sh");
    assert!(nuxt.contains("rm -rf .nuxt node_modules deployment/build"));
    let sveltekit = asset_text("sveltekit/deployment/build/02_run_build.sh");
    assert!(sveltekit.contains("rm -rf .svelte-kit deployment/build"));
    let vue = asset_text("vue/deployment/build/02_run_build.sh");
    assert!(vue.contains("rm -rf .vite node_modules deployment/build"));

    let angular = asset_text("angular/deployment/build/02_run_build.sh");
    assert!(angular.contains("./node_modules/.bin/ng build --configuration production --output-path dist"));
    assert!(angular.contains("npm ci --include=optional"));
    assert!(angular.contains("[ ! -f \"dist/browser/index.html\" ]"));
    assert!(angular.contains("rm -rf .angular/cache node_modules deployment/build"));
    assert!(!angular.contains("rm -rf dist"));
    Ok(())
}

#[test]
fn django_artifact_contract_matches_the_production_runtime() -> Result<()> {
    let build = asset_text("django/deployment/build/02_run_build.sh");
    assert!(build.contains("local packages_dir=\".python-packages\""));
    assert!(build.contains("local wrapper_dir=\".venv/bin\""));
    assert!(build.contains("python_enable_toolchain"));
    assert!(build.contains("python -m pip install"));
    assert!(build.contains("export PYTHONPATH=\"$release_root/.python-packages${PYTHONPATH:+:$PYTHONPATH}\""));
    assert!(build.contains(".bonesdeploy/runtimes/python\" \"$@\""));
    assert!(build.contains("python --version >.bonesdeploy-python-runtime"));
    assert!(!build.contains("BONES_RUNTIME_PYTHON_VERSION"));
    assert!(!build.contains("production_python"));

    let prepare = asset_text("django/deployment/prepare/01_prepare_django.sh");
    let require_artifact = prepare.find("PYTHON_RUNTIME_MARKER not found").context("missing artifact validation")?;
    let validate = prepare.find("manage.py check --deploy").context("Django prepare must validate the application")?;
    let migration_skip =
        prepare.find("BONES_DJANGO_SKIP_MIGRATIONS").context("Django prepare must support skipping migrations")?;
    assert!(prepare.contains("readonly PYTHON_BIN=\"$SITE_ROOT/.bonesdeploy/runtimes/python/bin/python\""));
    assert!(prepare.contains("export PYTHONPATH=\"$PWD/.python-packages${PYTHONPATH:+:$PYTHONPATH}\""));
    assert!(prepare.contains("does not match provisioned runtime"));
    assert!(require_artifact < validate);
    assert!(require_artifact < migration_skip);
    assert!(!prepare.contains("pip install"));
    assert!(!prepare.contains("-m venv"));
    Ok(())
}

#[test]
fn framework_assets_do_not_duplicate_canonical_infrastructure() {
    assert!(framework_asset_paths().iter().all(|path| !path.split('/').any(|part| part == "infra")));
}

#[test]
fn every_framework_scaffolds_deployment_assets() -> Result<()> {
    for framework in framework_names() {
        let temp = tempfile::tempdir()?;
        scaffold_framework_project(&framework, temp.path())?;
        let deployment = temp.path().join("deployment");
        assert!(deployment.join("functions.sh").is_file(), "{framework} is missing deployment functions");
        assert!(deployment.read_dir()?.count() > 1, "{framework} is missing framework deployment assets");
    }
    Ok(())
}

#[test]
fn every_framework_has_a_build_environment_example() -> Result<()> {
    for framework in framework_names() {
        let selected = Framework::parse(&framework)?;
        let content = selected
            .build_environment_example(&Runtime::default())
            .ok_or_else(|| anyhow::anyhow!("{framework} is missing .env.build"))?;
        assert!(content.contains("Committed, non-secret"), "{framework} must include build environment header");
        assert!(content.contains("# BonesDeploy Infra"), "{framework} must include the BonesDeploy build section");
        for variable in ["NODE_VERSION=", "PYTHON_VERSION=", "RUBY_VERSION="] {
            assert!(!content.contains(variable), "{framework} must derive {variable} from Bones config");
        }
    }
    Ok(())
}

#[test]
fn framework_build_environments_do_not_declare_managed_runtime_versions() -> Result<()> {
    let runtime = Runtime { node_version: "25.8.0".into(), ..Runtime::default() };
    for framework in [Framework::Next, Framework::Nuxt] {
        let content = framework
            .build_environment_example(&runtime)
            .ok_or_else(|| anyhow::anyhow!("{framework} is missing .env.build"))?;
        assert!(!content.contains("NODE_VERSION="));
    }
    Ok(())
}

#[test]
fn framework_build_environment_example_does_not_overwrite_existing_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    scaffold_framework_env_build("next", temp.path(), &Runtime::default())?;
    assert!(fs::read_to_string(temp.path().join(".env.build"))?.contains("NEXT_PUBLIC_API_URL="));
    fs::write(temp.path().join(".env.build"), "CUSTOM=value\n")?;
    scaffold_framework_env_build("next", temp.path(), &Runtime::default())?;
    assert_eq!(fs::read_to_string(temp.path().join(".env.build"))?, "CUSTOM=value\n");
    Ok(())
}

#[test]
fn framework_pnpm_installs_use_the_persistent_store() {
    for framework in framework_names() {
        let path = format!("{framework}/deployment/build/02_run_build.sh");
        let script = asset_text(&path);
        if script.contains("pnpm install") {
            assert!(script.contains("--store-dir \"$PNPM_STORE_DIR\""), "{path} must use the persistent pnpm store");
        }
    }
    let laravel = asset_text("laravel/deployment/build/03_build_frontend.sh");
    if laravel.contains("pnpm install") {
        assert!(laravel.contains("--store-dir \"$PNPM_STORE_DIR\""));
    }
}

#[test]
fn corepack_supports_modern_pnpm_package_layouts() -> Result<()> {
    let functions =
        kit_asset("deployment/functions.sh").ok_or_else(|| anyhow::anyhow!("missing deployment functions"))?;
    let functions = String::from_utf8_lossy(&functions);

    assert!(functions.contains("COREPACK_COMPAT_VERSION=\"0.31.0\""));
    assert!(functions.contains("COREPACK_MODERN_VERSION=\"0.34.5\""));
    assert!(functions.contains("[ \"$major\" -eq 18 ]"));
    assert!(functions.contains("COREPACK_HOME=\"$BUILD_CACHE_DIR/corepack/$target_version\""));
    Ok(())
}

#[test]
fn prepare_scripts_preserve_validation_and_mutation_order() -> Result<()> {
    let laravel = asset_text("laravel/deployment/prepare/01_prepare_laravel.sh");
    assert!(laravel.contains("php artisan optimize"));
    for command in ["optimize:clear", "package:discover", "queue:restart", "artisan up"] {
        assert!(!laravel.contains(command), "prepare must not run {command}");
    }
    let django = asset_text("django/deployment/prepare/01_prepare_django.sh");
    let check = django.find("manage.py check --deploy").ok_or_else(|| anyhow::anyhow!("missing deployment check"))?;
    let migrate = django.find("manage.py migrate").ok_or_else(|| anyhow::anyhow!("missing migration"))?;
    assert!(check < migrate);
    Ok(())
}

#[test]
fn framework_defaults_match_runtime_and_canonical_names() -> Result<()> {
    for framework in framework_names() {
        let defaults = framework_defaults(&framework)?;
        let config: Runtime = serde_json::from_value(serde_json::Value::Object(defaults))?;
        assert_eq!(config.template, framework);
    }
    let custom = framework_defaults("custom")?;
    assert_eq!(custom.get("template"), Some(&serde_json::Value::String("custom".into())));
    assert_eq!(custom.get("web_root"), base_framework_defaults()?.get("web_root"));

    for framework in Framework::ALL {
        let name = framework.to_string();
        let has_assets = framework_names().contains(&name);
        if *framework == Framework::Custom {
            assert!(!has_assets, "custom must not have embedded framework assets");
        } else {
            assert!(has_assets, "{name} must have embedded framework assets");
            Framework::parse(&name)?;
        }
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

#[test]
fn skill_doc_names_cover_the_expected_topics() {
    let names = doc_names();
    assert!(names.contains(&"commands".to_string()), "missing `commands` skill doc");
    assert!(names.contains(&"workflows".to_string()), "missing `workflows` skill doc");
    assert!(names.contains(&"methodology".to_string()), "missing `methodology` skill doc");
    assert!(!names.contains(&"SKILL".to_string()), "SKILL.md must be excluded from `skill list`");
}

#[test]
fn build_toolchains_use_verified_precompiled_mise_runtimes() -> Result<()> {
    let functions = kit_asset("deployment/functions.sh").ok_or_else(|| anyhow::anyhow!("missing functions.sh"))?;
    let functions = String::from_utf8_lossy(&functions);

    assert!(functions.contains("MISE_VERSION=\"2026.10.0\""));
    assert!(functions.contains("MISE_BINARY_SHA256="));
    assert!(functions.contains("MISE_DATA_DIR=\"$BUILD_CACHE_DIR/mise/data\""));
    assert!(functions.contains("MISE_ALL_COMPILE=\"false\""));
    assert!(functions.contains("MISE_NODE_COMPILE=\"false\""));
    assert!(functions.contains("MISE_PYTHON_COMPILE=\"false\""));
    assert!(functions.contains("MISE_RUBY_COMPILE=\"false\""));
    assert!(functions.contains("MISE_REGISTRY_FLOATING=\"false\""));
    assert!(functions.contains("MISE_AUTO_UPDATE=\"false\""));
    assert!(functions.contains("MISE_NO_HOOKS=\"true\""));
    assert!(functions.contains("MISE_OVERRIDE_CONFIG_FILENAMES=\".bonesdeploy-mise-disabled\""));
    assert!(functions.contains("MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES=\"none\""));
    assert!(functions.contains("mise_install_runtime node \"$version\""));
    assert!(functions.contains("mise_install_runtime ruby \"$version\""));
    assert!(functions.contains("mise_install_runtime python \"$version\""));
    let node_version = functions.find("if [ -n \"${NODE_VERSION:-}\" ]").context("missing NODE_VERSION resolver")?;
    let dot_node_version = functions.find("if [ -f .node-version ]").context("missing .node-version resolver")?;
    let nvmrc = functions.find("if [ -f .nvmrc ]").context("missing .nvmrc resolver")?;
    let tool_versions = functions.find("if [ -f .tool-versions ]").context("missing .tool-versions resolver")?;
    assert!(node_version < dot_node_version && dot_node_version < nvmrc && nvmrc < tool_versions);
    assert!(!functions.contains("node-v${version}-linux-${node_arch}"));
    assert!(!functions.contains("cache.ruby-lang.org"));
    assert!(!functions.contains("make -j"));
    Ok(())
}
