use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

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
