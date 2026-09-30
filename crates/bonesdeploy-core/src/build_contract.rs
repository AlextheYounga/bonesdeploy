//! Shared local native build contract.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::de::Error as DeError;
use serde_json::Value;

use crate::config::{Bones, build_env, is_numbered_shell_script, variables};
use crate::paths;

/// The immutable builder image used by local builds.
pub const BUILDER_IMAGE: &str = paths::IMAGE_STORE_BASE_IMAGE;
pub const BUILDER_IMAGE_DIGEST: &str = paths::IMAGE_STORE_BASE_IMAGE_DIGEST;

/// The only build target supported by the shared contract.
pub const TARGET_PLATFORM_NAME: &str = "linux/amd64";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TargetPlatform {
    #[default]
    LinuxAmd64,
}

impl serde::Serialize for TargetPlatform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(TARGET_PLATFORM_NAME)
    }
}

impl<'de> serde::Deserialize<'de> for TargetPlatform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        if value == TARGET_PLATFORM_NAME {
            return Ok(Self::LinuxAmd64);
        }
        Err(DeError::unknown_variant(&value, &[TARGET_PLATFORM_NAME]))
    }
}

impl TargetPlatform {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LinuxAmd64 => TARGET_PLATFORM_NAME,
        }
    }
}

pub const SOURCE_MOUNT: &str = "/workspace/source";
pub const CACHE_MOUNT: &str = "/workspace/cache";
pub const WORKSPACE_ROOT: &str = "/workspace";

/// Lists executable build scripts in lexical order by their two-digit prefix.
/// Files must use the established `NN_name.sh` convention.
///
/// # Errors
/// Returns an error when the scripts directory cannot be read.
pub fn numbered_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.file_name().and_then(|name| name.to_str()).is_some_and(is_numbered_shell_script) {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

const DERIVED_ENV_DENYLIST: &[&str] = &[
    "app.ssh_user",
    "app.host",
    "app.port",
    "app.branch",
    "app.project_root",
    "runtime.permissions",
    "runtime.backend",
    "runtime.node_version",
    "app.server.host",
    "app.server.port",
    "app.dns",
    "backup",
    "build.timeout_seconds",
];

/// Projects safe derived configuration and committed `.env.build` values.
/// Container-controlled variables cannot be overridden by project files.
///
/// # Errors
/// Returns an error when `.env.build` cannot be loaded or attempts to replace
/// a variable owned by the container contract.
pub fn environment(cfg: &Bones, source_context: &Path) -> Result<Vec<(String, String)>> {
    let mut values = derived_environment(cfg)?;
    for (key, value) in build_env::load(source_context)? {
        if variables::CONTAINER_CONTROLLED.contains(&key.as_str()) {
            bail!(".env.build variable `{key}` is reserved for the build container contract");
        }
        values.push((key, value));
    }
    Ok(values)
}

pub fn derived_environment(cfg: &Bones) -> Result<Vec<(String, String)>> {
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
    values.push((format!("BONES_{}", path.join("_").to_ascii_uppercase()), value.to_string()));
}

/// The cache location that keeps sites, platforms, and builder identities isolated.
#[must_use]
pub fn cache_path(cache_root: &Path, site: &str) -> PathBuf {
    let builder_key = BUILDER_IMAGE_DIGEST.replace(':', "-");
    cache_root.join(site).join(TargetPlatform::LinuxAmd64.name()).join(builder_key)
}
