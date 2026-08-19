use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use bonesdeploy::commands::update::sync::refresh_local_infrastructure;

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn refresh_local_infrastructure_updates_managed_files_without_touching_custom() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let source_dir = temp.path().join("source");
    let project_root = temp.path().join("project");
    let infra_dir = project_root.join("infra");
    fs::create_dir_all(&infra_dir)?;
    write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/build/01_build.sh"), "generic deploy")?;
    write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/functions.sh"), "shared functions")?;
    write(
        &source_dir.join("crates/bonesdeploy/assets/frameworks/laravel/deployment/build/01_build.sh"),
        "laravel deploy",
    )?;
    write(&project_root.join(".env"), "TEMPLATE=laravel\n")?;
    write(&infra_dir.join("provision/custom/runtime.py"), "def deploy(ctx):\n    custom(ctx)\n")?;

    refresh_local_infrastructure(&source_dir, &project_root, Some("laravel"))?;
    assert_eq!(fs::read_to_string(project_root.join(".env"))?, "TEMPLATE=laravel\n");
    assert_eq!(fs::read_to_string(infra_dir.join("deployment/build/01_build.sh"))?, "laravel deploy");
    assert_eq!(fs::read_to_string(infra_dir.join("deployment/functions.sh"))?, "shared functions");
    assert_eq!(
        fs::read_to_string(infra_dir.join("provision/custom/runtime.py"))?,
        "def deploy(ctx):\n    custom(ctx)\n"
    );
    let mode = fs::metadata(infra_dir.join("deployment/build/01_build.sh"))?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o755);
    Ok(())
}

#[test]
fn refresh_local_infrastructure_leaves_managed_core_untouched() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let source_dir = temp.path().join("source");
    let project_root = temp.path().join("project");
    let core_file = project_root.join("infra/provision/core/runtime.py");
    let deployment_file = project_root.join("infra/deployment/build/01_build.sh");
    write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/functions.sh"), "new functions")?;
    write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/build/01_build.sh"), "new deployment")?;
    write(&project_root.join(".env"), "TEMPLATE=custom\n")?;
    write(&core_file, "locally changed")?;
    write(&deployment_file, "locally preserved")?;
    refresh_local_infrastructure(&source_dir, &project_root, Some("custom"))?;
    assert_eq!(fs::read_to_string(core_file)?, "locally changed");
    assert_eq!(fs::read_to_string(deployment_file)?, "new deployment");
    Ok(())
}
