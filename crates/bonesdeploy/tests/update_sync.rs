use std::fs;
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
fn refresh_local_infrastructure_rejects_existing_custom_provisioning_without_touching_files() -> Result<()> {
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
    write(&infra_dir.join("custom/runtime.py"), "def deploy(ctx):\n    custom(ctx)\n")?;

    let Err(error) = refresh_local_infrastructure(&source_dir, &project_root, Some("laravel")) else {
        return Err(anyhow::anyhow!("existing custom provisioning did not stop update"));
    };
    assert_eq!(fs::read_to_string(project_root.join(".env"))?, "TEMPLATE=laravel\n");
    assert!(error.to_string().contains("temporarily unsupported"));
    assert!(!infra_dir.join("deployment").exists());
    assert_eq!(fs::read_to_string(infra_dir.join("custom/runtime.py"))?, "def deploy(ctx):\n    custom(ctx)\n");
    Ok(())
}

#[test]
fn refresh_local_infrastructure_does_not_touch_project_owned_files() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let source_dir = temp.path().join("source");
    let project_root = temp.path().join("project");
    let infra_dir = project_root.join("infra");
    let deployment_file = project_root.join("infra/deployment/build/01_build.sh");
    write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/functions.sh"), "new functions")?;
    write(&source_dir.join("crates/bonesdeploy/assets/kit/deployment/build/01_build.sh"), "new deployment")?;
    write(&project_root.join(".env"), "TEMPLATE=custom\n")?;
    write(&infra_dir.join("templates/custom.txt"), "project template")?;
    write(&deployment_file, "locally preserved")?;
    refresh_local_infrastructure(&source_dir, &project_root, Some("custom"))?;
    assert_eq!(fs::read_to_string(infra_dir.join("templates/custom.txt"))?, "project template");
    assert_eq!(fs::read_to_string(deployment_file)?, "new deployment");
    Ok(())
}
