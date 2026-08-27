use std::fs;

use anyhow::Result;

#[test]
fn materialized_artifacts_include_templates_and_preserve_custom() -> Result<()> {
    let project = tempfile::tempdir()?;
    bonesinfra::materialize_project_artifacts(project.path())?;
    let infra = project.path().join("infra");
    assert!(infra.join("bonesinfra.whl").is_file());
    assert!(infra.join("templates/shared/nginx/index.html.j2").is_file());
    assert!(infra.join("templates/frameworks/laravel/queue-worker.service.j2").is_file());
    assert!(!infra.join(".framework").exists());

    let custom = project.path().join("infra/custom/runtime.py");
    let custom_parent = custom.parent().ok_or_else(|| anyhow::anyhow!("custom runtime path has no parent"))?;
    fs::create_dir_all(custom_parent)?;
    fs::write(&custom, "project owned")?;
    fs::create_dir_all(infra.join(".framework"))?;
    fs::write(infra.join(".framework/stale.py"), "stale")?;

    bonesinfra::materialize_project_artifacts(project.path())?;

    assert_eq!(fs::read_to_string(custom)?, "project owned");
    assert!(!infra.join(".framework").exists());
    Ok(())
}
