use std::env;
use std::fs;
use std::process;

use anyhow::{Result, anyhow};
use bonesdeploy_core::config::load;

use bonesremote::release::lifecycle::build::run_scripts::{derived_config_env, list_scripts, resolve_build_env};

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
        root.join(".env"),
        "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=nuxt\nWEB_ROOT=.output/public\n",
    )?;
    let cfg = load(&root.join(".env"))?;

    let env = derived_config_env(&cfg)?;

    assert!(env.contains(&("BONES_RUNTIME_TEMPLATE".to_string(), "nuxt".to_string())));
    assert!(env.contains(&("BONES_APP_PROJECT_NAME".to_string(), "demo".to_string())), "{env:?}");
    assert!(!env.iter().any(|(key, _)| key == "BONES_APP_SERVER_HOST"));
    assert!(!env.iter().any(|(key, _)| key == "BONES_APP_SERVER_PORT"));
    assert!(!env.iter().any(|(key, _)| key == "BONES_APP_REPO_PATH"));
    assert!(!env.iter().any(|(key, _)| key == "BONES_APP_PROJECT_ROOT"));
    assert!(!env.iter().any(|(key, _)| key == "BONES_APP_REMOTE_NAME"));
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
    fs::write(site_root.join(".env"), "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=nuxt\n")?;
    fs::write(source.join(".env.build"), "NEXT_PUBLIC_API_URL=https://api.example.com\n")?;
    let cfg = load(&site_root.join(".env"))?;

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
    fs::write(site_root.join(".env"), "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=next\n")?;
    let cfg = load(&site_root.join(".env"))?;

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
        site_root.join(".env"),
        "PROJECT_NAME=demo\nHOST=deploy.example.com\nPORT=22\nDOMAIN=app.example.com\nTEMPLATE=nuxt\n",
    )?;
    let cfg = load(&site_root.join(".env"))?;

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
fn build_timeout_setting_is_denied_in_build_env() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-env-build-timeout-{}", process::id()));
    fs::create_dir_all(&root)?;
    fs::write(root.join(".env"), "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=nuxt\n")?;
    let cfg = load(&root.join(".env"))?;

    let env = derived_config_env(&cfg)?;

    assert!(
        !env.iter().any(|(key, _)| key == "BONES_BUILD_TIMEOUT_SECONDS"),
        "build timeout should not leak into the build container env: {env:?}"
    );
    fs::remove_dir_all(root).ok();
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
    fs::write(site_root.join(".env"), "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=next\n")?;
    fs::write(source.join(".env.build"), "BONES_RUNTIME_TEMPLATE=evil\n")?;
    let cfg = load(&site_root.join(".env"))?;

    let result = resolve_build_env(&cfg, &source);

    assert!(result.is_err(), "BONES_* in .env.build should be rejected");
    let err = match result {
        Ok(_) => return Err(anyhow::anyhow!("reserved BONES_* variable unexpectedly succeeded")),
        Err(error) => error.to_string(),
    };
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
    fs::write(site_root.join(".env"), "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=nuxt\n")?;
    let cfg = load(&site_root.join(".env"))?;

    let env = resolve_build_env(&cfg, &source)?;

    assert!(env.contains(&("BONES_RUNTIME_TEMPLATE".to_string(), "nuxt".to_string())));
    assert!(!env.iter().any(|(key, _)| key == "NEXT_PUBLIC_API_URL"), "no .env.build means no extra vars");
    fs::remove_dir_all(site_root).ok();
    fs::remove_dir_all(source).ok();
    Ok(())
}

#[test]
fn container_contract_values_cannot_be_overridden_by_env_build() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-env-build-contract-{}", process::id()));
    let source = env::temp_dir().join(format!("bonesremote-env-build-contract-src-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&source);
    fs::create_dir_all(&root)?;
    fs::create_dir_all(&source)?;
    fs::write(root.join(".env"), "PROJECT_NAME=demo\nHOST=deploy.example.com\nTEMPLATE=next\n")?;
    fs::write(source.join(".env.build"), "PROJECT_NAME=attacker\n")?;
    let cfg = load(&root.join(".env"))?;

    let result = resolve_build_env(&cfg, &source);

    assert!(result.is_err());
    let error = result.err().ok_or_else(|| anyhow!("reserved container variables must be rejected"))?;
    assert!(error.to_string().contains("reserved"));
    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(source).ok();
    Ok(())
}
