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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::parse_package_version;

    #[test]
    fn parses_package_version_from_manifest_package_section() -> Result<()> {
        let manifest = r#"
[package]
name = "bonesdeploy"
version = "0.2.8"
edition = "2024"

[dependencies]
version = "not-this"
"#;

        assert_eq!(parse_package_version(manifest)?, "0.2.8");
        Ok(())
    }

    #[test]
    fn rejects_manifest_without_package_version() {
        let result = parse_package_version("[dependencies]\nversion = \"0.2.8\"\n");
        assert!(result.is_err());
    }
}
