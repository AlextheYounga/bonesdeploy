use std::fs;

#[cfg(unix)]
use std::env;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::process::Command;

use anyhow::Result;
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
    assert!(django.contains("python3 -m pip install"));
    assert!(django.contains("--target \"$packages_dir\""));
    assert!(django.contains("exec /usr/bin/python3 -m gunicorn \"$@\""));
    assert!(!django.contains("BONES_RUNTIME_PYTHON_VERSION"));
    assert!(django.contains("rm -rf deployment/build"));

    let django_prepare = asset_text("django/deployment/prepare/01_prepare_django.sh");
    assert!(!django_prepare.contains("-m venv"));
    assert!(!django_prepare.contains("pip install"));
    assert!(django_prepare.contains("$VENV_DIR/bin/python"));

    let laravel = asset_text("laravel/deployment/build/03_build_frontend.sh");
    assert!(laravel.contains("rm -rf node_modules deployment/build"));
    let rails = asset_text("rails/deployment/build/02_run_build.sh");
    assert!(rails.contains("bundle install"));
    assert!(rails.contains("BUNDLE_DEPLOYMENT=\"true\""));
    assert!(rails.contains("BUNDLE_WITHOUT=\"development:test\""));
    assert!(!rails.contains("--deployment"));
    assert!(!rails.contains("--without"));
    assert!(rails.contains("rm -rf node_modules tmp/cache deployment/build"));
    assert!(!rails.contains("rm -rf node_modules tmp/cache vendor/bundle"));

    let rails_prepare = asset_text("rails/deployment/prepare/01_prepare_rails.sh");
    let check = rails_prepare
        .find("/usr/bin/bundle check")
        .ok_or_else(|| anyhow::anyhow!("Rails prepare must check the packaged bundle"))?;
    let migration_skip = rails_prepare
        .find("BONES_RAILS_SKIP_MIGRATIONS")
        .ok_or_else(|| anyhow::anyhow!("Rails prepare must support skipping migrations"))?;
    let migrate = rails_prepare
        .find("/usr/bin/bundle exec rails db:migrate")
        .ok_or_else(|| anyhow::anyhow!("Rails prepare must run migrations through the target bundle"))?;
    assert!(check < migration_skip);
    assert!(check < migrate);
    assert!(!rails_prepare.contains("bundle install"));
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
    assert!(build.contains("python3 -m pip install"));
    assert!(build.contains("export PYTHONPATH=\"$release_root/.python-packages${PYTHONPATH:+:$PYTHONPATH}\""));
    assert!(build.contains("exec /usr/bin/python3 \"$@\""));
    assert!(!build.contains("BONES_RUNTIME_PYTHON_VERSION"));
    assert!(!build.contains("production_python"));

    let prepare = asset_text("django/deployment/prepare/01_prepare_django.sh");
    let require_artifact = prepare
        .find("build the Django artifact first")
        .ok_or_else(|| anyhow::anyhow!("missing artifact validation"))?;
    let validate = prepare
        .find("manage.py check --deploy")
        .ok_or_else(|| anyhow::anyhow!("Django prepare must validate the application"))?;
    let migration_skip = prepare
        .find("BONES_DJANGO_SKIP_MIGRATIONS")
        .ok_or_else(|| anyhow::anyhow!("Django prepare must support skipping migrations"))?;
    assert!(prepare.contains("readonly PYTHON_BIN=\"$VENV_DIR/bin/python\""));
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
        assert!(content.contains("# BonesDeploy Infra\nNODE_VERSION="), "{framework} must declare Node in .env.build");
    }
    Ok(())
}

#[test]
fn next_and_nuxt_build_environments_use_the_selected_node_version() -> Result<()> {
    let runtime = Runtime { node_version: "25.8.0".into(), ..Runtime::default() };
    for framework in [Framework::Next, Framework::Nuxt] {
        let content = framework
            .build_environment_example(&runtime)
            .ok_or_else(|| anyhow::anyhow!("{framework} is missing .env.build"))?;
        assert!(content.contains("NODE_VERSION=25.8.0"));
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

#[cfg(unix)]
#[test]
fn node_install_extracts_a_cold_cache_archive() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let archive_root = temp.path().join("archive-root");
    let node_root = archive_root.join("node-v1.2.3-linux-x64/bin");
    fs::create_dir_all(&node_root)?;
    let node = node_root.join("node");
    fs::write(&node, "#!/bin/sh\nprintf 'v1.2.3\\n'\n")?;
    fs::set_permissions(&node, fs::Permissions::from_mode(0o755))?;

    let archive = temp.path().join("node-v1.2.3-linux-x64.tar.xz");
    let archive_status = Command::new("tar")
        .current_dir(temp.path())
        .args(["-cJf"])
        .arg(&archive)
        .args(["-C"])
        .arg(&archive_root)
        .arg("node-v1.2.3-linux-x64")
        .status()?;
    assert!(archive_status.success(), "failed to create Node archive fixture");
    let checksum = Command::new("sha256sum").current_dir(temp.path()).arg(&archive).output()?;
    assert!(checksum.status.success(), "failed to checksum Node archive fixture");
    let checksum_hash = String::from_utf8(checksum.stdout)?
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Node archive checksum fixture was empty"))?
        .to_string();
    let checksums = temp.path().join("SHASUMS256.txt");
    fs::write(&checksums, format!("{checksum_hash}  node-v1.2.3-linux-x64.tar.xz\n"))?;

    let fake_bin = temp.path().join("bin");
    fs::create_dir(&fake_bin)?;
    let fake_curl = fake_bin.join("curl");
    fs::write(
        &fake_curl,
        "#!/bin/sh\noutput=\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-o\" ]; then output=$2; shift 2; else shift; fi\ndone\ncase $output in\n  *SHASUMS256.txt) cp \"$FIXTURE_CHECKSUMS\" \"$output\" ;;\n  *) cp \"$FIXTURE_ARCHIVE\" \"$output\" ;;\nesac\n",
    )?;
    fs::set_permissions(&fake_curl, fs::Permissions::from_mode(0o755))?;
    let functions = kit_asset("deployment/functions.sh").ok_or_else(|| anyhow::anyhow!("missing functions.sh"))?;
    let functions_file = temp.path().join("functions.sh");
    fs::write(&functions_file, functions)?;
    let status = Command::new("bash")
        .current_dir(temp.path())
        .arg("-c")
        .arg("source \"$FUNCTIONS_FILE\"\nnode_install 1.2.3 x64\n")
        .env("FUNCTIONS_FILE", &functions_file)
        .env("BUILD_CACHE_DIR", temp.path().join("cache"))
        .env("FIXTURE_ARCHIVE", &archive)
        .env("FIXTURE_CHECKSUMS", &checksums)
        .env("PATH", format!("{}:{}", fake_bin.display(), env::var("PATH").unwrap_or_default()))
        .status()?;
    assert!(status.success(), "Node fixture installation failed");
    assert!(temp.path().join("cache/node/v1.2.3-linux-x64/bin/node").is_file());
    Ok(())
}
