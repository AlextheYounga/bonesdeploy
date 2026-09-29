use std::fs;

use anyhow::Result;
use bonesdeploy_core::build_contract;
use bonesdeploy_core::build_contract::TargetPlatform;
use bonesdeploy_core::config::Bones;
use tempfile::tempdir;

#[test]
fn numbered_scripts_are_discovered_in_lexical_order() -> Result<()> {
    let dir = tempdir()?;
    for name in ["10_second.sh", "02_first.sh", "bad.sh", "100_too-large.sh"] {
        fs::write(dir.path().join(name), "")?;
    }

    let scripts = build_contract::numbered_scripts(dir.path())?;
    let names: Vec<_> = scripts.iter().filter_map(|path| path.file_name()).collect();
    assert_eq!(names, ["02_first.sh", "10_second.sh"]);
    Ok(())
}

#[test]
fn target_platform_has_a_canonical_name_and_json_value() -> Result<()> {
    assert_eq!(TargetPlatform::default().name(), build_contract::TARGET_PLATFORM_NAME);
    assert_eq!(serde_json::to_string(&TargetPlatform::default())?, "\"linux/amd64\"");
    assert!(serde_json::from_str::<TargetPlatform>("\"linux/arm64\"").is_err());
    assert!(serde_json::from_str::<TargetPlatform>("null").is_err());
    Ok(())
}

#[test]
fn environment_projection_excludes_sensitive_and_controlled_values() -> Result<()> {
    let dir = tempdir()?;
    fs::write(dir.path().join(".env.build"), "PUBLIC_VALUE=ok\nPROJECT_NAME=override\n")?;
    let mut config = Bones::for_site("atlas");
    config.host = "secret.example".into();

    let error = build_contract::environment(&config, dir.path()).expect_err("controlled variable");
    assert!(error.to_string().contains("PROJECT_NAME"));

    fs::write(dir.path().join(".env.build"), "PUBLIC_VALUE=ok\n")?;
    let values = build_contract::environment(&config, dir.path())?;
    assert!(values.iter().any(|(key, value)| key == "PUBLIC_VALUE" && value == "ok"));
    assert!(!values.iter().any(|(key, _)| key == "BONES_APP_HOST"));
    Ok(())
}

#[test]
fn environment_projection_excludes_the_entire_backup_section() -> Result<()> {
    let dir = tempdir()?;
    let mut config = Bones::for_site("atlas");
    config.backup.schedule = "0 3 * * *".into();
    config.backup.retention_days = 14;
    config.backup.passphrase = "secret-backup-passphrase".into();

    let derived = build_contract::derived_environment(&config)?;
    assert!(derived.iter().all(|(key, _)| !key.starts_with("BONES_BACKUP")));
    assert!(derived.iter().all(|(_, value)| value != "secret-backup-passphrase"));

    let merged = build_contract::environment(&config, dir.path())?;
    assert!(merged.iter().all(|(key, _)| !key.starts_with("BONES_BACKUP")));
    assert!(merged.iter().all(|(_, value)| value != "secret-backup-passphrase"));
    Ok(())
}
