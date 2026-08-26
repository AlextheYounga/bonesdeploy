use std::env;
use std::fs::{self, OpenOptions, Permissions};
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config;
use crate::frameworks;
use crate::infra::ssh;
use crate::ui::output;
use bonesdeploy_core::config as shared_config;
use bonesdeploy_core::config::{KeyValueCredentials, ServiceCredentials, ServicesRequest, parse_port};
use bonesdeploy_core::paths;

mod environment;
pub mod gpg;

const LOCAL_ENV_SECRET: &str = "infra/secrets/.env.gpg";
const DEFAULT_SECRET_MODE: &str = "640";

fn environment_to_push(plaintext: &str) -> Result<&str> {
    shared_config::validate_dotenv(plaintext)?;
    Ok(plaintext)
}

pub fn init() -> Result<()> {
    if !Path::new(paths::LOCAL_INFRA_DIR).is_dir() {
        bail!("Missing infra/ directory\n\n{}", output::next_step("bonesdeploy init"));
    }

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    initialize_defaults(&cfg)?;

    println!("{} Secrets initialized.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy secrets edit"));
    Ok(())
}

pub fn initialize_defaults(cfg: &config::Bones) -> Result<()> {
    let encrypted_path = Path::new(LOCAL_ENV_SECRET);
    if encrypted_path.is_file() {
        return Ok(());
    }

    let mut effective_config = cfg.clone();
    shared_config::apply_derived_defaults(&mut effective_config);
    let framework = framework_for_secrets(&effective_config.runtime.template)?;

    let env_path = Path::new(paths::DOT_ENV);
    let loaded = config::load_local(env_path)?;
    let framework_content =
        framework.environment_example(&effective_config.project_name, &effective_config.domain).unwrap_or_default();
    let plaintext = environment::prepare(env_path, &framework_content, &effective_config, &loaded)?;

    gpg::ensure_installed()?;
    let key_fingerprint = gpg::ensure_project_key(&cfg.project_name)?;
    fs::create_dir_all(paths::LOCAL_INFRA_SECRETS_DIR)
        .with_context(|| format!("Failed to create {}", paths::LOCAL_INFRA_SECRETS_DIR))?;

    let temp_path = create_temp_edit_path()?;
    fs::write(&temp_path, plaintext)
        .with_context(|| format!("Failed to write default secrets to {}", temp_path.display()))?;
    fs::set_permissions(&temp_path, Permissions::from_mode(0o600))?;

    let encrypted_result = gpg::run(&[
        "--batch",
        "--yes",
        "--output",
        encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
        "--encrypt",
        "--recipient",
        &key_fingerprint,
        temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
    ]);
    let cleanup_result = fs::remove_file(&temp_path);
    encrypted_result?;
    cleanup_result.with_context(|| format!("Failed to remove temporary secrets file {}", temp_path.display()))?;
    fs::set_permissions(encrypted_path, Permissions::from_mode(0o640))?;
    Ok(())
}

pub(super) fn read_service_credentials(cfg: &config::Bones) -> Result<ServicesRequest> {
    let path = Path::new(LOCAL_ENV_SECRET);
    if !path.is_file() {
        bail!("Missing encrypted secrets; run `bonesdeploy secrets edit` first")
    }
    let plaintext = String::from_utf8(gpg::decrypt(path)?).context("Decrypted secrets are not valid UTF-8")?;
    let values = shared_config::parse_dotenv(&plaintext)?.applications;
    let required = |key: &str| -> Result<String> {
        values
            .get(key)
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Missing {key} in encrypted secrets; run `bonesdeploy secrets edit`"))
    };
    let project = &cfg.project_name;
    let database = sanitize_identifier(project);
    let selected = |name: &str| cfg.services.services.iter().any(|service| service == name);
    let port =
        |key: &str| values.get(key).filter(|value| !value.trim().is_empty()).cloned().unwrap_or_else(|| "6379".into());
    Ok(ServicesRequest {
        postgres: selected("postgres")
            .then(|| {
                required(environment::POSTGRES_PASSWORD).map(|password| ServiceCredentials {
                    password,
                    username: format!("{project}_postgres"),
                    database: database.clone(),
                })
            })
            .transpose()?,
        mysql: selected("mysql")
            .then(|| {
                required(environment::MYSQL_PASSWORD).map(|password| ServiceCredentials {
                    password,
                    username: format!("{project}_mysql"),
                    database: database.clone(),
                })
            })
            .transpose()?,
        mongodb: selected("mongodb")
            .then(|| {
                required(environment::MONGODB_PASSWORD).map(|password| ServiceCredentials {
                    password,
                    username: format!("{project}_mongodb"),
                    database: database.clone(),
                })
            })
            .transpose()?,
        valkey: selected("valkey")
            .then(|| {
                required(environment::VALKEY_PASSWORD)
                    .map(|password| KeyValueCredentials { password, port: port(environment::VALKEY_PORT) })
            })
            .transpose()?,
        redis: selected("redis")
            .then(|| {
                required(environment::REDIS_PASSWORD)
                    .map(|password| KeyValueCredentials { password, port: port(environment::REDIS_PORT) })
            })
            .transpose()?,
    })
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

pub fn framework_for_secrets(template: &str) -> Result<frameworks::Framework> {
    if template.trim().is_empty() {
        return Ok(frameworks::Framework::Custom);
    }

    frameworks::Framework::parse(template).with_context(|| format!("Invalid TEMPLATE value: {template}"))
}

pub fn edit() -> Result<()> {
    gpg::ensure_installed()?;

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let key_fingerprint = gpg::ensure_project_key(&cfg.project_name)?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);

    if let Some(parent) = encrypted_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let temp_path = create_temp_edit_path()?;

    if encrypted_path.is_file() {
        gpg::run(&[
            "--batch",
            "--yes",
            "--decrypt",
            "--output",
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
        ])?;
    }

    let edit_result = open_editor(&temp_path);
    let encrypt_result = if edit_result.is_ok() {
        gpg::run(&[
            "--batch",
            "--yes",
            "--output",
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
            "--encrypt",
            "--recipient",
            &key_fingerprint,
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
        ])
    } else {
        Ok(())
    };

    let cleanup_result = fs::remove_file(&temp_path);

    edit_result?;
    encrypt_result?;
    if let Err(error) = cleanup_result
        && error.kind() != ErrorKind::NotFound
    {
        eprintln!("Warning: could not remove temporary secret file: {}", temp_path.display());
    }

    println!("{} Secrets updated.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy secrets push"));
    Ok(())
}

pub async fn push() -> Result<()> {
    gpg::ensure_installed()?;

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let runtime_group = shared_config::runtime_group_for(&cfg.project_name);

    let ssh_user = config::bootstrap_ssh_user(&cfg);
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as(&ssh_user, &cfg.host, port).await?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);
    if !encrypted_path.is_file() {
        bail!("Missing encrypted secrets\n\n{}", output::next_step("bonesdeploy secrets edit"));
    }

    let plaintext =
        String::from_utf8(gpg::decrypt(encrypted_path)?).context("Decrypted secrets are not valid UTF-8")?;
    let shared = Path::new(&cfg.project_root).join(paths::SHARED_DIR);
    let target = shared.join(paths::DOT_ENV);
    let parent = target.parent().ok_or_else(|| anyhow::anyhow!("Remote target has no parent: {}", target.display()))?;
    let parent_s = ssh::shell_quote(&parent.display().to_string());
    let target_s = ssh::shell_quote(&target.display().to_string());
    let group_s = ssh::shell_quote(&runtime_group);
    let environment = environment_to_push(&plaintext)?;
    let cmd = format!(
        "tmp=; trap 'rm -f \"$tmp\"' EXIT; mkdir -p {parent_s} && tmp=$(mktemp {target_s}.XXXXXX) && cat > \"$tmp\" && chown root:{group_s} \"$tmp\" && chmod {DEFAULT_SECRET_MODE} \"$tmp\" && mv \"$tmp\" {target_s} && tmp=",
    );

    ssh::run_cmd_with_stdin(&session, &cmd, environment.as_bytes()).await?;
    session.close().await?;
    println!("{} Secrets pushed.", output::success_marker());
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::environment_to_push;

    #[test]
    fn environment_push_uses_the_encrypted_file_without_modification() -> Result<()> {
        let secrets = "APP_KEY=base64:abc123\nDATABASE_URL=postgres://localhost/app\n";

        let environment = environment_to_push(secrets)?;

        assert_eq!(environment, secrets);
        Ok(())
    }
}

fn open_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("$EDITOR is not set. Set it before running `bonesdeploy secrets edit`."))?;

    let status = Command::new("sh")
        .arg("-c")
        .arg("${EDITOR:?EDITOR is not set} \"$1\"")
        .arg("sh")
        .arg(path)
        .env("EDITOR", editor)
        .status()
        .context("Failed to launch editor")?;

    if !status.success() {
        bail!("Editor exited with status {status}");
    }

    Ok(())
}

fn create_temp_edit_path() -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesdeploy-env-{}-{nonce}", process::id()));

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("Failed to create temp file {}", path.display()))?;

    Ok(path)
}
