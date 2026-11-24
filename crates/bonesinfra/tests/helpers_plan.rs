use bonesdeploy_core::names;

use std::fs;

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesinfra::provisioning::prepare_helpers;
use bonesinfra::{ProvisioningRuntime, ServerContext, TemplateResolver};

#[test]
fn helper_plan_installs_packages_tools_and_records_etc() -> Result<()> {
    let project = tempfile::tempdir()?;
    for path in [
        "shared/scripts/install-debian-aliases.sh",
        "shared/scripts/install-neovim.sh",
        "shared/scripts/install-neovim-config.sh",
        "shared/starship.toml",
        "shared/profile.d/bonesinfra-shell.sh",
        names::ETCKEEPER_TEMPLATE,
    ] {
        let target = project.path().join("infra/templates").join(path);
        fs::create_dir_all(target.parent().ok_or_else(|| anyhow::anyhow!("asset has no parent"))?)?;
        fs::write(target, "asset")?;
    }
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    let server = ServerContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&server)?;

    let deploy = prepare_helpers(&runtime, &TemplateResolver::new(project.path()))?;
    let names = deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(names.first().map(String::as_str), Some("Install supplementary helper apt packages"));
    assert!(names.iter().any(|name| name == "Install starship prompt"));
    assert!(names.iter().any(|name| name == "Install Neovim > 0.11 from official release"));
    assert!(names.iter().any(|name| name == "Install rainfrog binary"));
    assert_eq!(names.last().map(String::as_str), Some(names::ETCKEEPER_RECORD_OPERATION));
    Ok(())
}
