use std::fs;

use anyhow::Result;
use bonesdeploy_core::config::{Runtime, RuntimeBackend};

const FRAMEWORKS: &[&str] = &["angular", "custom", "django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue"];

#[test]
fn native_materialization_keeps_only_the_selected_framework_paths() -> Result<()> {
    for framework in FRAMEWORKS {
        let project = tempfile::tempdir()?;
        let wheel = bonesinfra::materialize_project_artifacts(project.path(), &native_runtime(framework))?;
        let infra = project.path().join("infra");

        assert_eq!(fs::read(wheel)?, bonesinfra::embedded_wheel()?);
        assert!(infra.join("templates/shared/nginx/index.html.j2").is_file());
        for candidate in FRAMEWORKS {
            assert_eq!(
                infra.join("templates/frameworks").join(candidate).is_dir(),
                candidate == framework,
                "unexpected framework path for {candidate} in {framework} project"
            );
        }
    }
    Ok(())
}

#[test]
fn docker_materialization_removes_every_framework_path() -> Result<()> {
    let project = tempfile::tempdir()?;
    let runtime = Runtime { backend: RuntimeBackend::Docker, ..Runtime::default() };
    let wheel = bonesinfra::materialize_project_artifacts(project.path(), &runtime)?;
    let infra = project.path().join("infra");

    assert_eq!(fs::read(wheel)?, bonesinfra::embedded_wheel()?);
    assert!(infra.join("templates/shared/nginx/index.html.j2").is_file());
    for framework in FRAMEWORKS {
        assert!(!infra.join("templates/frameworks").join(framework).exists());
    }
    Ok(())
}

#[test]
fn rematerialization_restores_newly_selected_paths_and_preserves_project_files() -> Result<()> {
    let project = tempfile::tempdir()?;
    let custom = project.path().join("infra/custom/runtime.py");
    fs::create_dir_all(custom.parent().ok_or_else(|| anyhow::anyhow!("custom runtime path has no parent"))?)?;
    fs::write(&custom, "project owned")?;

    bonesinfra::materialize_project_artifacts(project.path(), &native_runtime("laravel"))?;
    let frameworks = project.path().join("infra/templates/frameworks");
    assert!(frameworks.join("laravel").is_dir());
    assert!(!frameworks.join("django").exists());

    bonesinfra::materialize_project_artifacts(project.path(), &native_runtime("django"))?;
    assert!(frameworks.join("django").is_dir());
    assert!(!frameworks.join("laravel").exists());
    assert_eq!(fs::read_to_string(custom)?, "project owned");
    Ok(())
}

#[test]
fn invalid_framework_is_rejected_before_framework_paths_are_removed() -> Result<()> {
    let project = tempfile::tempdir()?;
    let Err(error) = bonesinfra::materialize_project_artifacts(project.path(), &native_runtime("unknown")) else {
        return Err(anyhow::anyhow!("unknown framework was accepted"));
    };

    assert!(error.to_string().contains("unknown BonesInfra framework"));
    for framework in FRAMEWORKS {
        assert!(project.path().join("infra/templates/frameworks").join(framework).is_dir());
    }
    Ok(())
}

#[test]
fn materialization_removes_the_obsolete_framework_directory() -> Result<()> {
    let project = tempfile::tempdir()?;
    let obsolete = project.path().join("infra/.framework");
    fs::create_dir_all(&obsolete)?;
    fs::write(obsolete.join("stale.py"), "stale")?;

    bonesinfra::materialize_project_artifacts(project.path(), &native_runtime("custom"))?;

    assert!(!obsolete.exists());
    Ok(())
}

fn native_runtime(framework: &str) -> Runtime {
    Runtime {
        template: if framework == "custom" { String::new() } else { framework.to_string() },
        ..Runtime::default()
    }
}
