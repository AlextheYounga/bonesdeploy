use bonesdeploy_core::names;

use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesinfra::provisioning::prepare_site_setup;
use bonesinfra::{ProvisioningContext, ProvisioningRuntime, TemplateResolver};

fn config() -> Bones {
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    config.ssh_user = "deploy".to_owned();
    config.runtime.template = "laravel".to_owned();
    config.backup.passphrase = "not-exposed-in-plan-metadata".to_owned();
    config
}

fn write_templates(project: &Path) -> Result<()> {
    for (path, content) in [
        (names::INDEX_TEMPLATE, "{{ project_name }}"),
        ("shared/cron/backup.cron.j2", "{{ backup_schedule }} root {{ bonesremote_path }}"),
        (names::ETCKEEPER_TEMPLATE, "#!/bin/sh\netckeeper commit '{{ message }}' || true\n"),
    ] {
        let target = project.join("infra/templates").join(path);
        fs::create_dir_all(target.parent().ok_or_else(|| anyhow::anyhow!("template has no parent"))?)?;
        fs::write(target, content)?;
    }
    Ok(())
}

#[test]
fn site_setup_plan_registers_the_complete_base_site_sequence() -> Result<()> {
    let project = tempfile::tempdir()?;
    write_templates(project.path())?;
    let context = ProvisioningContext::from_bones(&config())?;
    let runtime = ProvisioningRuntime::new(&context.server)?;

    let deploy = prepare_site_setup(&context, &runtime, &TemplateResolver::new(project.path()))?;
    let names = deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(names.first().map(String::as_str), Some("Ensure runtime group exists"));
    assert!(names.iter().any(|name| name == "Seed placeholder index page"));
    assert!(names.iter().any(|name| name == "Install root-only Borg passphrase file"));
    assert!(names.iter().any(|name| name == "Install scheduled backup cron entry"));
    assert_eq!(names.last().map(String::as_str), Some(names::ETCKEEPER_RECORD_OPERATION));
    assert!(names.iter().all(|name| !name.contains(&config().backup.passphrase)));
    Ok(())
}

#[test]
fn site_setup_plan_omits_backup_operations_when_unconfigured() -> Result<()> {
    let project = tempfile::tempdir()?;
    write_templates(project.path())?;
    let mut config = config();
    config.backup.passphrase.clear();
    let context = ProvisioningContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;

    let deploy = prepare_site_setup(&context, &runtime, &TemplateResolver::new(project.path()))?;
    let names = deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect::<Vec<_>>();

    assert!(!names.iter().any(|name| name == "Install root-only Borg passphrase file"));
    assert!(!names.iter().any(|name| name == "Install scheduled backup cron entry"));
    Ok(())
}
