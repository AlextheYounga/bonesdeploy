use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesinfra::manifest::DeletionPlan;
use bonesinfra::provisioning::prepare_site_delete;
use bonesinfra::{ProvisioningContext, ProvisioningRuntime};
use rustinfra::operations::PreparedDeploy;

fn names(deploy: &PreparedDeploy) -> Vec<String> {
    deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect()
}

#[test]
fn deletion_stops_services_before_removing_deepest_artifacts() -> Result<()> {
    let mut config = Bones::for_site("atlas");
    config.host = "server.example.com".to_owned();
    config.runtime.template = "laravel".to_owned();
    let context = ProvisioningContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let deploy = prepare_site_delete(&runtime, &DeletionPlan::resolve(&context)?)?;
    let names = names(&deploy);

    let first_remove = names
        .iter()
        .position(|name| name.starts_with("Remove "))
        .ok_or_else(|| anyhow::anyhow!("deletion plan did not contain removal operations"))?;
    assert!(names[..first_remove].iter().all(|name| name.starts_with("Stop and disable ")));
    assert!(names.iter().any(|name| name == "Reload systemd after Quick Tunnel removal"));
    assert_eq!(names.last().map(String::as_str), Some("Reload nginx after Quick Tunnel removal"));
    Ok(())
}
