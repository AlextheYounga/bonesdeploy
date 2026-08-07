use std::fs;

use anyhow::{Context, Result, bail};

use bonesdeploy_core::paths;

pub fn run(file: Option<&str>, key: Option<&str>) -> Result<()> {
    print!("{}", render(file, key)?);
    Ok(())
}

pub fn render(file: Option<&str>, key: Option<&str>) -> Result<String> {
    let path = file.unwrap_or(paths::LOCAL_BONES_TOML);
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read config file: {path}"))?;
    let value: toml::Value = toml::from_str(&content).with_context(|| format!("Failed to parse TOML: {path}"))?;

    let Some(key) = key else {
        return Ok(content);
    };

    // Walk dotted paths like `app.server.host` so users can actually read a value,
    // since every real bones.toml field is nested under a table.
    let Some(toml_value) = lookup_dotted(&value, key) else {
        bail!("Key '{key}' not found in {path}");
    };

    match toml_value {
        toml::Value::String(s) => Ok(s.clone()),
        toml::Value::Boolean(b) => Ok(b.to_string()),
        toml::Value::Integer(i) => Ok(i.to_string()),
        toml::Value::Float(f) => Ok(f.to_string()),
        _ => bail!("Unsupported value type for key '{key}'"),
    }
}

fn lookup_dotted<'v>(mut value: &'v toml::Value, key: &str) -> Option<&'v toml::Value> {
    for part in key.split('.') {
        value = value.get(part)?;
    }
    Some(value)
}
