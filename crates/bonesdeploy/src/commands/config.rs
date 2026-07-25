use std::fs;

use anyhow::{Context, Result, bail};

use shared::paths;

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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use anyhow::{Result, bail};
    use tempfile::TempDir;

    use super::render;

    fn write_temp(contents: &str) -> Result<(TempDir, PathBuf)> {
        let dir = TempDir::new()?;
        let path = dir.path().join("bones.toml");
        fs::write(&path, contents)?;
        Ok((dir, path))
    }

    #[test]
    fn dumps_full_file_when_key_omitted() -> Result<()> {
        let toml = "[app]\nproject_name = \"atlas\"\n";
        let (_dir, path) = write_temp(toml)?;
        assert_eq!(render(path.to_str(), None)?, toml);
        Ok(())
    }

    #[test]
    fn reads_string_key() -> Result<()> {
        let (_dir, path) = write_temp("[app]\nproject_name = \"atlas\"\n")?;
        assert_eq!(render(path.to_str(), Some("app.project_name"))?, "atlas");
        Ok(())
    }

    #[test]
    fn reads_integer_key() -> Result<()> {
        let (_dir, path) = write_temp("[app.deploy]\nreleases = 5\n")?;
        assert_eq!(render(path.to_str(), Some("app.deploy.releases"))?, "5");
        Ok(())
    }

    #[test]
    fn missing_key_bails() -> Result<()> {
        let (_dir, path) = write_temp("[app]\nproject_name = \"atlas\"\n")?;
        let result = render(path.to_str(), Some("nope"));
        let Err(err) = result else {
            bail!("expected error for missing key, got: {result:?}");
        };
        assert!(err.to_string().contains("not found"), "{}", err);
        Ok(())
    }
}
