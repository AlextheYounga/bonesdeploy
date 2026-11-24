use bonesdeploy_core::names;

use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesinfra::provisioning::prepare_server_setup;
use bonesinfra::{ProvisioningRuntime, ServerContext, TemplateResolver};

const TEMPLATES: &[&str] = &[
    "shared/modprobe/disable-algif.conf.j2",
    "shared/fail2ban/jail.local.j2",
    "shared/unattended-upgrades/20auto-upgrades.j2",
    "shared/unattended-upgrades/50unattended-upgrades.j2",
    "shared/scripts/copy-ssh-authorized-keys.sh.j2",
    "shared/scripts/install-bonesremote.sh.j2",
    "shared/sshd/99-bonesdeploy-root-login.conf.j2",
    "shared/sudoers/bonesdeploy.j2",
    names::ETCKEEPER_TEMPLATE,
];

fn write_templates(project: &Path) -> Result<()> {
    for path in TEMPLATES {
        let target = project.join("infra/templates").join(path);
        fs::create_dir_all(target.parent().ok_or_else(|| anyhow::anyhow!("template has no parent"))?)?;
        fs::write(target, "{{ ssh_port }}{{ bonesremote_version }}{{ message }}")?;
    }
    Ok(())
}

#[test]
fn server_setup_plan_covers_the_baseline_and_finishes_with_etckeeper() -> Result<()> {
    let project = tempfile::tempdir()?;
    write_templates(project.path())?;
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    config.ssh_user = "root".to_owned();
    config.port = "2222".to_owned();
    let server = ServerContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&server)?;

    let deploy = prepare_server_setup(&server, &runtime, &TemplateResolver::new(project.path()), "1.2.3")?;
    let names = deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(names.first().map(String::as_str), Some("Validate supported production host"));
    assert!(names.iter().any(|name| name == "Install setup apt packages"));
    for expected in [
        "Ensure apparmor service is enabled and started",
        "Apply UFW configuration",
        "Install fail2ban jail.local",
        "Ensure deploy user exists",
        "Disable password login for root",
        "Install bonesremote binary",
        "Install BonesDeploy sudoers drop-in",
    ] {
        assert!(names.iter().any(|name| name == expected), "missing {expected}");
    }
    assert_eq!(names.last().map(String::as_str), Some(names::ETCKEEPER_RECORD_OPERATION));
    Ok(())
}
