use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::paths;

const ENV_BUILD_HEADER: &str = "\
# Committed, non-secret values used while building this project.
# Do not place passwords, tokens, or private keys here.
";

/// Returns the default content for a fresh `.env.build` file.
#[must_use]
pub fn default_content() -> &'static str {
    ENV_BUILD_HEADER
}

/// Parses a `.env.build` file from the given directory.
///
/// Returns key-value pairs for all entries. Validates that keys are
/// valid POSIX environment variable names, that no key starts with
/// `BONES_` (reserved for derived values), and that no key is duplicated.
///
/// Returns an empty map when the file does not exist.
///
/// # Errors
/// Returns an error when the file exists but cannot be read, contains
/// invalid keys, duplicate keys, or reserved `BONES_*` keys.
pub fn load(dir: &Path) -> Result<BTreeMap<String, String>> {
    let path = dir.join(paths::ENV_BUILD_FILE);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    parse(&content)
}

/// Parses `.env.build` content without shell evaluation.
///
/// # Errors
/// Returns an error on invalid keys, duplicate keys, or reserved `BONES_*` keys.
pub fn parse(content: &str) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (key, value) =
            trimmed.split_once('=').with_context(|| format!("Line {line_num}: expected KEY=VALUE format"))?;

        let key = key.trim();
        validate_key(key, line_num)?;

        if map.contains_key(key) {
            bail!("Line {}: duplicate key `{key}`", line_num + 1);
        }

        let value = strip_quotes(value.trim());
        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
}

fn validate_key(key: &str, line_num: usize) -> Result<()> {
    if key.is_empty() {
        bail!("Line {line_num}: empty variable name");
    }

    if !is_valid_env_name(key) {
        bail!("Line {line_num}: invalid variable name `{key}`");
    }

    if key.starts_with("BONES_") {
        bail!("Line {line_num}: `BONES_*` names are reserved, found `{key}`");
    }

    Ok(())
}

fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn strip_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unquoted_values() {
        let map = parse("KEY=hello\nOTHER=world").unwrap();
        assert_eq!(map.get("KEY").map(String::as_str), Some("hello"));
        assert_eq!(map.get("OTHER").map(String::as_str), Some("world"));
    }

    #[test]
    fn parses_quoted_values() {
        let map = parse("KEY=\"hello world\"\nOTHER='single quotes'").unwrap();
        assert_eq!(map.get("KEY").map(String::as_str), Some("hello world"));
        assert_eq!(map.get("OTHER").map(String::as_str), Some("single quotes"));
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let map = parse("# comment\n\nKEY=val\n  \n# another\nOTHER=other").unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("KEY").map(String::as_str), Some("val"));
    }

    #[test]
    fn rejects_invalid_keys() {
        assert!(parse("1BAD=value").is_err());
        assert!(parse("BAD-KEY=value").is_err());
        assert!(parse("=value").is_err());
    }

    #[test]
    fn rejects_bones_reserved_prefix() {
        assert!(parse("BONES_SOMETHING=value").is_err());
        assert!(parse("BONES_X=1").is_err());
    }

    #[test]
    fn rejects_duplicate_keys() {
        assert!(parse("KEY=one\nKEY=two").is_err());
    }

    #[test]
    fn allows_valid_underscore_names() {
        let map = parse("_UNDERSCORE=1\nNEXT_PUBLIC_API_URL=https://api.example.com").unwrap();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn load_returns_empty_map_when_file_missing() {
        let dir = std::env::temp_dir().join("bones-env-build-missing-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let map = load(&dir).unwrap();
        assert!(map.is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_reads_env_build_from_directory() {
        let dir = std::env::temp_dir().join("bones-env-build-load-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(".env.build"), "API_URL=https://api.example.com\nSITE_NAME=Test\n").unwrap();
        let map = load(&dir).unwrap();
        assert_eq!(map.get("API_URL").map(String::as_str), Some("https://api.example.com"));
        assert_eq!(map.get("SITE_NAME").map(String::as_str), Some("Test"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn derived_bones_values_cannot_be_overridden() {
        let result = parse("BONES_RUNTIME_TEMPLATE=evil");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("reserved"), "error should mention reserved: {err}");
    }
}
