use std::fs;

use anyhow::Result;

#[test]
fn materialized_framework_contains_complete_distribution_and_preserves_custom() -> Result<()> {
    let project = tempfile::tempdir()?;
    let framework = bonesinfra::materialize_project_framework(project.path())?;
    assert!(framework.join("pyproject.toml").is_file());
    assert!(framework.join("uv.lock").is_file());
    assert!(framework.join("src/bonesinfra/__main__.py").is_file());

    let custom = project.path().join("infra/custom/runtime.py");
    let custom_parent = custom.parent().ok_or_else(|| anyhow::anyhow!("custom runtime path has no parent"))?;
    fs::create_dir_all(custom_parent)?;
    fs::write(&custom, "project owned")?;
    fs::write(framework.join("stale.py"), "stale")?;

    bonesinfra::materialize_project_framework(project.path())?;

    assert_eq!(fs::read_to_string(custom)?, "project owned");
    assert!(!framework.join("stale.py").exists());
    Ok(())
}
