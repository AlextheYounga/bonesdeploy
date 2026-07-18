use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str;

use anyhow::{Context, Result, anyhow, bail};
use rust_embed::Embed;
use serde_json::{Map, Value};

use shared::config::{Buildtime, Runtime};
use shared::paths;

#[derive(Embed)]
#[folder = "./kit/"]
struct Kit;

#[derive(Embed)]
#[folder = "./runtimes/"]
struct RuntimeAssets;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }

    Ok(())
}

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
    scaffold_kit_deployment_functions(bones_dir)?;
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_DEPLOYMENT_DIR)
}

pub fn scaffold_runtime_secrets(runtime: &str, bones_dir: &Path) -> Result<()> {
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_SECRETS_DIR)
}

pub fn runtime_buildtime_defaults(runtime: &str) -> Result<Buildtime> {
    let asset_path = format!("{runtime}/bones.toml");
    let Some(asset) = RuntimeAssets::get(&asset_path) else { return Ok(Buildtime::default()) };
    let value: toml::Value = toml::from_str(str::from_utf8(asset.data.as_ref())?)
        .with_context(|| format!("Failed to parse embedded build-time defaults at {asset_path}"))?;
    value
        .get("build")
        .cloned()
        .map(toml::Value::try_into)
        .transpose()
        .with_context(|| format!("Failed to parse [build] defaults at {asset_path}"))
        .map(Option::unwrap_or_default)
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

fn scaffold_kit_deployment_functions(bones_dir: &Path) -> Result<()> {
    let path = format!("{}functions.sh", paths::KIT_DEPLOYMENT_DIR);
    let Some(asset) = Kit::get(&path) else {
        return Ok(());
    };
    write_asset(bones_dir, &path, asset.data.as_ref())
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

fn write_asset(bones_dir: &Path, relative_path: &str, bytes: &[u8]) -> Result<()> {
    let dest = bones_dir.join(relative_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(&dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if relative_path.starts_with(paths::KIT_DEPLOYMENT_DIR) {
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Kit, RuntimeAssets, runtime_defaults, runtime_names};

    #[test]
    fn next_runtime_includes_the_build_script() {
        assert!(RuntimeAssets::get("next/deployment/build/02_run_build.sh").is_some());
    }

    #[test]
    fn shared_deployment_functions_include_cache_and_node_entry_points() {
        let Some(functions) = Kit::get("deployment/functions.sh") else {
            assert!(false, "shared functions should be embedded");
            return;
        };
        let functions = String::from_utf8_lossy(functions.data.as_ref());
        assert!(functions.contains("configure_build_cache"));
        assert!(functions.contains("install_node_dependencies"));
        assert!(functions.contains("COMPOSER_CACHE_DIR"));
        assert!(functions.contains("BUNDLE_USER_CACHE"));
        assert!(
            functions.contains("local extracted=\"$BUILD_NODE_TMP_DIR/extracted/node-v${version}-linux-${node_arch}\"")
        );
    }

    #[test]
    fn node_install_extracts_a_cold_cache_archive() -> anyhow::Result<()> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;

        let temp = tempfile::tempdir()?;
        let archive_root = temp.path().join("archive-root");
        let node_root = archive_root.join("node-v1.2.3-linux-x64/bin");
        fs::create_dir_all(&node_root)?;
        let node = node_root.join("node");
        fs::write(&node, "#!/bin/sh\nprintf 'v1.2.3\\n'\n")?;
        fs::set_permissions(&node, PermissionsExt::from_mode(0o755))?;

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
        let checksum_line = format!("{checksum_hash}  node-v1.2.3-linux-x64.tar.xz\n");
        let checksums = temp.path().join("SHASUMS256.txt");
        fs::write(&checksums, checksum_line)?;

        let fake_bin = temp.path().join("bin");
        fs::create_dir(&fake_bin)?;
        let fake_curl = fake_bin.join("curl");
        fs::write(
            &fake_curl,
            "#!/bin/sh\noutput=\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-o\" ]; then output=$2; shift 2; else shift; fi\ndone\ncase $output in\n  *SHASUMS256.txt) cp \"$FIXTURE_CHECKSUMS\" \"$output\" ;;\n  *) cp \"$FIXTURE_ARCHIVE\" \"$output\" ;;\nesac\n",
        )?;
        fs::set_permissions(&fake_curl, PermissionsExt::from_mode(0o755))?;

        let cache = temp.path().join("cache");
        let functions = Kit::get("deployment/functions.sh").ok_or_else(|| anyhow::anyhow!("missing functions.sh"))?;
        let functions_file = temp.path().join("functions.sh");
        fs::write(&functions_file, functions.data.as_ref())?;

        let current_path = std::env::var("PATH").unwrap_or_default();
        let script = "source \"$FUNCTIONS_FILE\"\nnode_install 1.2.3 x64\n";
        let status = Command::new("bash")
            .current_dir(temp.path())
            .arg("-c")
            .arg(script)
            .env("FUNCTIONS_FILE", &functions_file)
            .env("BUILD_CACHE_DIR", &cache)
            .env("FIXTURE_ARCHIVE", &archive)
            .env("FIXTURE_CHECKSUMS", &checksums)
            .env("PATH", format!("{}:{current_path}", fake_bin.display()))
            .status()?;
        assert!(status.success(), "Node fixture installation failed");
        assert!(cache.join("node/v1.2.3-linux-x64/bin/node").is_file());

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

        let kit = Kit::get("deployment/build/02_run_build copy.sh")
            .map(|asset| String::from_utf8_lossy(asset.data.as_ref()).into_owned())
            .unwrap_or_default();
        if kit.contains("pnpm install") {
            assert!(kit.contains("--store-dir \"$PNPM_STORE_DIR\""));
        }
    }

    #[test]
    fn runtime_defaults_fit_the_single_file_schema() {
        for runtime in runtime_names() {
            let defaults = runtime_defaults(&runtime).expect("runtime defaults should parse");
            let config: shared::config::Runtime = serde_json::from_value(serde_json::Value::Object(defaults))
                .expect("runtime defaults should deserialize");
            assert_eq!(config.template, runtime);
        }
    }

    #[test]
    fn runtime_answers_accept_boolean_template_settings() {
        let mut answers = runtime_defaults("nuxt").expect("Nuxt defaults should parse");
        answers.insert("static".into(), serde_json::Value::Bool(true));

        let config: shared::config::Runtime = serde_json::from_value(serde_json::Value::Object(answers))
            .expect("boolean template settings should deserialize");
        assert_eq!(config.extra.get("static").and_then(|value| value.as_bool()), Some(true));
        assert!(toml::to_string(&config).expect("runtime should serialize").contains("static = true"));
    }
}
