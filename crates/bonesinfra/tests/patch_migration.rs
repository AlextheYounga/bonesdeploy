use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, bail};
use bonesdeploy_core::config::Bones;
use bonesinfra::patches::apply_local;

#[test]
fn migration_preserves_owned_files_and_refuses_ambiguous_trees() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "migration_child", "--nocapture"])
        .env("BONES_MIGRATION_TEST_ROOT", root.path())
        .env("XDG_DATA_HOME", root.path().join("data"))
        .output()?;
    if !output.status.success() {
        bail!("migration test failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

#[test]
fn migration_child() -> Result<()> {
    let Some(root) = env::var_os("BONES_MIGRATION_TEST_ROOT") else { return Ok(()) };
    let root = PathBuf::from(root);
    let project = root.join("project");
    let old = project.join(".bones");
    fs::create_dir_all(old.join("infra/nested"))?;
    fs::create_dir_all(old.join("deployment"))?;
    fs::create_dir_all(old.join("secrets"))?;
    fs::write(old.join("infra/nested/custom.py"), "VALUE = 1\n")?;
    fs::write(old.join("deployment/build.sh"), "#!/bin/sh\n")?;
    fs::write(old.join("secrets/app.age"), "encrypted\n")?;

    apply_local(&Bones::for_site("migration"), &project, "0.8.0-beta.1")?;
    assert_eq!(fs::read_to_string(project.join("infra/custom/nested/custom.py"))?, "VALUE = 1\n");
    assert_eq!(fs::read_to_string(project.join("infra/deployment/build.sh"))?, "#!/bin/sh\n");
    assert_eq!(fs::read_to_string(project.join("infra/secrets/app.age"))?, "encrypted\n");
    assert!(!old.exists());

    let other = root.join("ambiguous");
    fs::create_dir_all(other.join(".bones"))?;
    fs::write(other.join(".bones/unknown"), "keep me\n")?;
    let config = Bones::for_site("ambiguous");
    assert!(apply_local(&config, &other, "0.8.0").is_err());
    assert_eq!(fs::read_to_string(other.join(".bones/unknown"))?, "keep me\n");
    assert!(!other.join("infra").exists());
    assert!(apply_local(&config, &other, "0.8").is_err());
    assert!(apply_local(&config, &other, "0.8.x").is_err());
    Ok(())
}
