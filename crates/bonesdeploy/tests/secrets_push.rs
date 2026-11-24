mod common;

use std::fs;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::infra::ssh::shell_quote;
use bonesdeploy_core::paths;
use common::{TestEnv, ssh::SshFixture};

#[test]
fn secrets_push_transmits_valid_decrypted_environment_without_modification() -> Result<()> {
    let env = TestEnv::new()?;
    let plaintext = env.repo().join("decrypted-fixture");
    let received = env.repo().join("received-environment");
    let command = env.repo().join("received-command");
    let script = format!(
        "#!/bin/sh\nprintf '%s' \"$SSH_ORIGINAL_COMMAND\" > {}\ncat > {}\n",
        shell_quote(&command.display().to_string()),
        shell_quote(&received.display().to_string())
    );
    let ssh = SshFixture::start(&script)?;
    let init = env.run(&[
        "init",
        "--non-interactive",
        "--project-name",
        "atlas",
        "--host",
        "127.0.0.1",
        "--port",
        &ssh.port.to_string(),
    ])?;
    ensure!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let username = String::from_utf8(Command::new("id").arg("-un").output()?.stdout)?.trim().to_owned();
    let dotenv = env.repo().join(".env");
    fs::write(&dotenv, fs::read_to_string(&dotenv)?.replace("SSH_USER=root", &format!("SSH_USER={username}")))?;
    ssh.install_command(
        "gpg",
        &format!(
            "#!/bin/sh\nif [ \"$1\" = --version ]; then exit 0; fi\ncat {}\n",
            shell_quote(&plaintext.display().to_string())
        ),
    )?;
    let encrypted = env.repo().join(paths::LOCAL_INFRA_ENV_SECRET);
    fs::create_dir_all(encrypted.parent().ok_or_else(|| anyhow::anyhow!("missing secrets directory"))?)?;
    fs::write(&encrypted, b"encrypted bytes")?;
    let contents = "APP_KEY=base64:abc123\nDATABASE_URL=postgres://localhost/app\n";
    fs::write(&plaintext, contents)?;

    let output = ssh.run(&env, &["secrets", "push"])?;
    ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(&received)?, contents);
    assert_eq!(fs::read(&encrypted)?, b"encrypted bytes");
    assert!(fs::read_to_string(command)?.contains("bonesremote shared install-environment --site"));

    fs::write(&plaintext, "INVALID LINE\n")?;
    let output = ssh.run(&env, &["secrets", "push"])?;
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(received)?, contents);
    Ok(())
}
