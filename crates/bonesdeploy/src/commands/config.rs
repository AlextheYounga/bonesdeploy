use std::fs;

use anyhow::{Context, Result, bail};

pub fn run(file: &str, key: &str) -> Result<()> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read config file: {file}"))?;
    let value: toml::Value =
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML: {file}"))?;

    let result = value.get(key);
    let Some(toml_value) = result else {
        bail!("Key '{key}' not found in {file}");
    };

    match toml_value {
        toml::Value::String(s) => print!("{s}"),
        toml::Value::Boolean(b) => print!("{b}"),
        toml::Value::Integer(i) => print!("{i}"),
        toml::Value::Float(f) => print!("{f}"),
        _ => bail!("Unsupported value type for key '{key}'"),
    }

    Ok(())
}
