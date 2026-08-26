use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config;

pub(super) const POSTGRES_PASSWORD: &str = "POSTGRES_PASSWORD";
pub(super) const MYSQL_PASSWORD: &str = "MYSQL_PASSWORD";
pub(super) const MONGODB_PASSWORD: &str = "MONGODB_PASSWORD";
pub(super) const VALKEY_PASSWORD: &str = "VALKEY_PASSWORD";
pub(super) const VALKEY_PORT: &str = "VALKEY_PORT";
pub(super) const REDIS_PASSWORD: &str = "REDIS_PASSWORD";
pub(super) const REDIS_PORT: &str = "REDIS_PORT";

pub(super) fn prepare(
    path: &Path,
    framework_content: &str,
    cfg: &config::Bones,
    local: &config::LoadedLocal,
) -> Result<String> {
    let framework_parsed = config::parse_dotenv(framework_content)?;
    let mut local_defaults = framework_parsed.applications.clone();
    for key in service_blank_keys(&cfg.services.services) {
        local_defaults.entry(key.into()).or_default();
    }
    merge_application_keys(path, &local_defaults)?;

    let mut production = framework_content.to_string();
    let mut values = config::production_application_keys(&framework_parsed)?;
    values.extend(local.applications.clone());
    for (key, value) in values {
        set_env_value(&mut production, &key, &value);
    }
    set_env_value(&mut production, "APP_KEY", "");
    let plaintext = inject_service_environment(production, cfg)?;
    config::validate_dotenv(&plaintext)?;
    Ok(plaintext)
}

fn service_blank_keys(services: &[String]) -> Vec<&'static str> {
    let mut keys = Vec::new();
    for service in services {
        keys.extend(match service.as_str() {
            "postgres" => [POSTGRES_PASSWORD, "POSTGRES_USER", "POSTGRES_DB"].as_slice(),
            "mysql" => [MYSQL_PASSWORD, "MYSQL_USER", "MYSQL_DB"].as_slice(),
            "mongodb" => [MONGODB_PASSWORD, "MONGODB_USER", "MONGODB_DB"].as_slice(),
            "valkey" => [VALKEY_PASSWORD, VALKEY_PORT].as_slice(),
            "redis" => [REDIS_PASSWORD, REDIS_PORT].as_slice(),
            _ => &[],
        });
    }
    keys
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

fn inject_service_environment(mut content: String, cfg: &config::Bones) -> Result<String> {
    let identifier = cfg
        .project_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>();
    for service in &cfg.services.services {
        match service.as_str() {
            "postgres" => {
                let pass = generated_password()?;
                let user = format!("{identifier}_postgres");
                for (key, value) in [
                    (POSTGRES_PASSWORD, pass.as_str()),
                    ("POSTGRES_USER", user.as_str()),
                    ("POSTGRES_DB", identifier.as_str()),
                ] {
                    set_env_value(&mut content, key, value);
                }
                set_env_value(
                    &mut content,
                    "POSTGRES_URL",
                    &format!("postgresql://{user}:{pass}@127.0.0.1:5432/{identifier}"),
                );
                for (key, value) in [
                    ("DB_CONNECTION", "pgsql"),
                    ("DB_HOST", "127.0.0.1"),
                    ("DB_PORT", "5432"),
                    ("DB_DATABASE", &identifier),
                    ("DB_USERNAME", &user),
                    ("DB_PASSWORD", &pass),
                ] {
                    set_env_value(&mut content, key, value);
                }
            }
            "mysql" => {
                let pass = generated_password()?;
                let user = format!("{identifier}_mysql");
                for (key, value) in
                    [(MYSQL_PASSWORD, pass.as_str()), ("MYSQL_USER", user.as_str()), ("MYSQL_DB", identifier.as_str())]
                {
                    set_env_value(&mut content, key, value);
                }
                set_env_value(&mut content, "MYSQL_URL", &format!("mysql://{user}:{pass}@127.0.0.1:3306/{identifier}"));
            }
            "mongodb" => {
                let pass = generated_password()?;
                let user = format!("{identifier}_mongodb");
                for (key, value) in [
                    (MONGODB_PASSWORD, pass.as_str()),
                    ("MONGODB_USER", user.as_str()),
                    ("MONGODB_DB", identifier.as_str()),
                ] {
                    set_env_value(&mut content, key, value);
                }
                set_env_value(
                    &mut content,
                    "MONGODB_URI",
                    &format!("mongodb://{user}:{pass}@127.0.0.1:27017/{identifier}?authSource={identifier}"),
                );
            }
            "valkey" | "redis" => {
                let pass = generated_password()?;
                let password_key = if service == "valkey" { VALKEY_PASSWORD } else { REDIS_PASSWORD };
                let port_key = if service == "valkey" { VALKEY_PORT } else { REDIS_PORT };
                let url_key = if service == "valkey" { "VALKEY_URL" } else { "REDIS_URL" };
                set_env_value(&mut content, password_key, &pass);
                set_env_value(&mut content, port_key, "6379");
                set_env_value(&mut content, url_key, &format!("redis://:{pass}@127.0.0.1:6379/0"));
            }
            _ => {}
        }
    }
    Ok(content)
}

fn generated_password() -> Result<String> {
    let mut bytes = [0_u8; 24];
    getrandom::fill(&mut bytes).map_err(|error| anyhow::anyhow!("Failed to generate service password: {error}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}
