use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub(super) fn read_package_version(manifest: &Path) -> Result<String> {
    let content = fs::read_to_string(manifest).with_context(|| format!("Failed to read {}", manifest.display()))?;
    parse_package_version(&content)
        .with_context(|| format!("Failed to parse package version from {}", manifest.display()))
}

pub(super) fn parse_package_version(manifest: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(manifest)?;
    value
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .filter(|version| !version.is_empty())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("missing [package] version"))
}
