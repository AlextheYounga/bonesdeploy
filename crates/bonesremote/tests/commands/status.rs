use std::{env, fs, process};

use anyhow::Result;

use bonesremote::commands::status::ssl_status;

#[test]
fn reads_ssl_domain_from_conventionally_named_nginx_config() -> Result<()> {
    let path = env::temp_dir().join(format!("bonesremote-status-{}.conf", process::id()));
    fs::write(&path, "server_name example.test;\nlisten 443 ssl;\n")?;
    let path = path.display().to_string();

    assert_eq!(ssl_status(&path).domain, "example.test");
    assert!(ssl_status(&path).enabled);
    fs::remove_file(path)?;
    Ok(())
}
