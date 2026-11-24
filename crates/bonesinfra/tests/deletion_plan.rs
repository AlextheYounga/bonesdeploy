use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesinfra::ProvisioningContext;
use bonesinfra::manifest::{ArtifactKind, DeletionArtifact, DeletionPlan, DeletionService};

fn context() -> Result<ProvisioningContext> {
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    config.runtime.template = "laravel".to_owned();
    config.backup.passphrase = "retained".to_owned();
    ProvisioningContext::from_bones(&config)
}

#[test]
fn deletion_plan_keeps_backup_data_and_includes_tunnel_cleanup() -> Result<()> {
    let context = context()?;
    let plan = DeletionPlan::resolve(&context)?;

    assert!(!plan.artifacts.iter().any(|artifact| artifact.path == context.paths.backup_repository));
    assert!(!plan.artifacts.iter().any(|artifact| artifact.path == context.paths.backup_passphrase_file));
    assert!(plan.artifacts.iter().any(|artifact| artifact.path == context.paths.backup_cron_file));
    assert!(plan.artifacts.iter().any(|artifact| artifact.owner == "tunnel"));
    assert!(plan.services.iter().any(|service| service.unit == "example-cloudflared.service"));
    Ok(())
}

#[test]
fn persisted_plan_rejects_unknown_fields_unsafe_paths_and_services() -> Result<()> {
    let context = context()?;
    let unknown = r#"{"artifacts":[],"services":[],"extra":true}"#;
    assert!(DeletionPlan::parse(unknown, &context).is_err());

    let unsafe_path = DeletionPlan {
        artifacts: vec![DeletionArtifact {
            name: "bad".to_owned(),
            path: "/etc/passwd".to_owned(),
            kind: ArtifactKind::File,
            owner: "bad".to_owned(),
        }],
        services: Vec::new(),
    };
    assert!(DeletionPlan::parse(&unsafe_path.to_json()?, &context).is_err());

    let unsafe_service = DeletionPlan {
        artifacts: Vec::new(),
        services: vec![DeletionService {
            name: "bad".to_owned(),
            unit: "nginx.service".to_owned(),
            owner: "bad".to_owned(),
        }],
    };
    assert!(DeletionPlan::parse(&unsafe_service.to_json()?, &context).is_err());
    Ok(())
}

#[test]
fn persisted_plan_round_trips_after_revalidation() -> Result<()> {
    let context = context()?;
    let plan = DeletionPlan::resolve(&context)?;

    assert_eq!(DeletionPlan::parse(&plan.to_json()?, &context)?, plan);
    Ok(())
}
