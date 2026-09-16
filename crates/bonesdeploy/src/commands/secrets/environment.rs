use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config;

pub(super) fn prepare(path: &Path, framework_content: &str, local: &config::LoadedLocal) -> Result<String> {
    let framework_parsed = config::parse_dotenv(framework_content)?;
    merge_application_keys(path, &framework_parsed.applications)?;

    let mut production = framework_content.to_string();
    let mut values = config::production_application_keys(&framework_parsed)?;
    values.extend(local.applications.clone());
    for (key, value) in values {
        set_env_value(&mut production, &key, &value);
    }
    set_env_value(&mut production, "APP_KEY", "");
    config::validate_dotenv(&production)?;
    Ok(production)
}

fn merge_application_keys(path: &Path, keys: &BTreeMap<String, String>) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let parsed = config::parse_dotenv(&content)?;
    let missing = keys.iter().filter(|(key, _)| !parsed.applications.contains_key(*key)).collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    let marker = "# >>> BonesDeploy managed configuration >>>";
    let insertion = content.find(marker).unwrap_or(content.len());
    let mut output = content[..insertion].to_string();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    for (key, value) in missing {
        output.push_str(&format!("{key}={value}\n"));
    }
    output.push_str(&content[insertion..]);
    fs::write(path, output)?;
    Ok(())
}

fn set_env_value(content: &mut String, key: &str, value: &str) {
    let replacement = format!("{key}={value}");
    let mut found = false;
    let mut output = String::new();
    for line in content.lines() {
        if line.split_once('=').is_some_and(|(name, _)| name.trim() == key) {
            output.push_str(&replacement);
            found = true;
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }
    if !found {
        output.push_str(&replacement);
        output.push('\n');
    }
    *content = output;
}
