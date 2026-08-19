use std::fs;

use anyhow::Result;

#[test]
fn materialized_core_contains_complete_distribution_and_preserves_custom() -> Result<()> {
    let project = tempfile::tempdir()?;
    let core = bonesinfra::materialize_project_core(project.path())?;
    assert!(core.join("pyproject.toml").is_file());
    assert!(core.join("uv.lock").is_file());
    assert!(core.join("src/bonesinfra/__main__.py").is_file());

    let custom = project.path().join("infra/provision/custom/runtime.py");
    let custom_parent = custom.parent().ok_or_else(|| anyhow::anyhow!("custom runtime path has no parent"))?;
    fs::create_dir_all(custom_parent)?;
    fs::write(&custom, "project owned")?;
    fs::write(core.join("stale.py"), "stale")?;

    bonesinfra::materialize_project_core(project.path())?;

    assert_eq!(fs::read_to_string(custom)?, "project owned");
    assert!(!core.join("stale.py").exists());
    Ok(())
}
