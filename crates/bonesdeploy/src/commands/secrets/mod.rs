use std::env;
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config;
use crate::frameworks;
use crate::infra::{self, ssh};
use crate::ui::output;
use bonesdeploy_core::config as shared_config;
use bonesdeploy_core::config::parse_port;
use bonesdeploy_core::paths;

use crate::platform;

mod environment;
pub mod gpg;

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
    let encrypted_path = Path::new(paths::LOCAL_INFRA_ENV_SECRET);
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
    let plaintext = environment::prepare(env_path, &framework_content, &loaded)?;

    gpg::ensure_installed()?;
    let key_fingerprint = gpg::ensure_project_key(&cfg.project_name)?;
    fs::create_dir_all(paths::LOCAL_INFRA_SECRETS_DIR)
        .with_context(|| format!("Failed to create {}", paths::LOCAL_INFRA_SECRETS_DIR))?;

    let temp_path = create_temp_edit_path()?;
    fs::write(&temp_path, plaintext)
        .with_context(|| format!("Failed to write default secrets to {}", temp_path.display()))?;
    #[cfg(unix)]
    platform::set_mode(&temp_path, 0o600)?;
    #[cfg(not(unix))]
    platform::set_mode(&temp_path, 0o600);

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
    #[cfg(unix)]
    platform::set_mode(encrypted_path, 0o640)?;
    #[cfg(not(unix))]
    platform::set_mode(encrypted_path, 0o640);
    Ok(())
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

    let encrypted_path = Path::new(paths::LOCAL_INFRA_ENV_SECRET);

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
        let result = gpg::run(&[
            "--batch",
            "--yes",
            "--output",
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
            "--encrypt",
            "--recipient",
            &key_fingerprint,
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
        ]);
        #[cfg(unix)]
        let encrypt_result = result.and_then(|()| platform::set_mode(encrypted_path, 0o640));
        #[cfg(not(unix))]
        let encrypt_result = result.map(|()| {
            platform::set_mode(encrypted_path, 0o640);
        });
        encrypt_result
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
    let encrypted_path = Path::new(paths::LOCAL_INFRA_ENV_SECRET);
    if !encrypted_path.is_file() {
        bail!("Missing encrypted secrets\n\n{}", output::next_step("bonesdeploy secrets edit"));
    }

    let plaintext =
        String::from_utf8(gpg::decrypt(encrypted_path)?).context("Decrypted secrets are not valid UTF-8")?;
    let environment = environment_to_push(&plaintext)?;

    let ssh_user = config::bootstrap_ssh_user(&cfg);
    let port = parse_port(&cfg.port)?;
    let session = ssh::SshTransport::connect_as(&ssh_user, &cfg.host, port).await?;
    let command = infra::shared_install_environment_command(&cfg.project_name);
    session.run_cmd_with_stdin(&command, environment.as_bytes()).await?;
    println!("{} Secrets pushed.", output::success_marker());
    Ok(())
}

pub async fn production_secrets_exist(cfg: &config::Bones) -> Result<bool> {
    let ssh_user = config::bootstrap_ssh_user(cfg);
    let port = parse_port(&cfg.port)?;
    let session = ssh::SshTransport::connect_as(&ssh_user, &cfg.host, port).await?;
    let target = Path::new(&cfg.project_root).join(paths::SHARED_DIR).join(paths::DOT_ENV);
    let command = format!(
        "if test -f {}; then printf present; else printf missing; fi",
        ssh::shell_quote(&target.display().to_string())
    );
    Ok(session.run_cmd(&command).await?.trim() == "present")
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use std::path::Path;

    use crate::test_support::with_env;

    use super::environment_to_push;

    #[test]
    fn environment_push_uses_the_encrypted_file_without_modification() -> Result<()> {
        let secrets = "APP_KEY=base64:abc123\nDATABASE_URL=postgres://localhost/app\n";

        let environment = environment_to_push(secrets)?;

        assert_eq!(environment, secrets);
        Ok(())
    }

    #[test]
    fn missing_configured_editor_names_the_editor_and_windows_executable_contract() -> Result<()> {
        let editor = if cfg!(windows) { "missing-editor.exe" } else { "missing-editor" };
        let result = with_env("EDITOR", Some(editor), || super::open_editor(Path::new("secret file.txt")));
        let error = result.err().context("missing configured editor should fail")?;
        let message = format!("{error:#}");
        assert!(message.contains("Configured editor"), "{message}");
        assert!(message.contains(editor), "{message}");
        Ok(())
    }
}

fn open_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("$EDITOR is not set. Set it before running `bonesdeploy secrets edit`."))?;

    let status = {
        #[cfg(windows)]
        {
            Command::new("cmd.exe")
                .args(["/D", "/S", "/C"])
                .arg(format!("\"{}\"", windows_editor_command_line(&editor, path)))
                .status()
        }
        #[cfg(not(windows))]
        {
            Command::new("sh")
                .arg("-c")
                .arg("${EDITOR:?EDITOR is not set} \"$1\"")
                .arg("sh")
                .arg(path)
                .env("EDITOR", &editor)
                .status()
        }
    }
    .with_context(|| {
        format!("Failed to launch configured editor `{editor}`; install it and ensure its executable is available")
    })?;

    if !status.success() {
        bail!("Configured editor `{editor}` exited with status {status}");
    }

    Ok(())
}

#[cfg(any(windows, test))]
fn windows_editor_command_line(editor: &str, path: &Path) -> String {
    let (program, arguments) = if let Some(unquoted) = editor.strip_prefix('"') {
        let end = unquoted.find('"').map_or(editor.len(), |index| index + 2);
        (&editor[..end], editor[end..].trim())
    } else if let Some(end) = editor.to_ascii_lowercase().find(".exe") {
        let end = end + ".exe".len();
        (&editor[..end], editor[end..].trim())
    } else {
        editor.split_once(char::is_whitespace).unwrap_or((editor, ""))
    };
    let program = if program.starts_with('"') || !program.chars().any(char::is_whitespace) {
        program.to_string()
    } else {
        format!("\"{program}\"")
    };
    let arguments = arguments.trim();
    if arguments.is_empty() {
        format!("{program} \"{}\"", path.display())
    } else {
        format!("{program} {arguments} \"{}\"", path.display())
    }
}

#[cfg(test)]
mod editor_tests {
    use std::path::Path;

    use super::windows_editor_command_line;

    #[test]
    fn windows_editor_command_line_quotes_executable_and_file_paths_with_spaces() {
        let command = windows_editor_command_line(
            r#"C:\Program Files\Editor\editor.exe --wait"#,
            Path::new(r#"C:\Users\Test User\secrets file.env"#),
        );

        assert_eq!(command, r#""C:\Program Files\Editor\editor.exe" --wait "C:\Users\Test User\secrets file.env""#);
    }

    #[test]
    fn windows_editor_command_line_preserves_command_arguments() {
        let command = windows_editor_command_line("code --wait", Path::new("secrets.env"));

        assert_eq!(command, r#"code --wait "secrets.env""#);
    }
}

fn create_temp_edit_path() -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesdeploy-env-{}-{nonce}", process::id()));

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("Failed to create temp file {}", path.display()))?;
    #[cfg(unix)]
    platform::set_mode(&path, 0o600)?;
    #[cfg(not(unix))]
    platform::set_mode(&path, 0o600);

    Ok(path)
}
