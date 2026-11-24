use bonesdeploy_core::names;

use std::fs;

use anyhow::Result;
use bonesdeploy_core::config::Bones;
use bonesinfra::provisioning::prepare_ssl;
use bonesinfra::{ProvisioningContext, ProvisioningRuntime, TemplateResolver};

#[test]
fn ssl_plan_stages_the_challenge_before_enabling_tls() -> Result<()> {
    let project = tempfile::tempdir()?;
    for path in [
        names::DEFAULT_CERTIFICATE_TEMPLATE,
        names::DEFAULT_DENY_TEMPLATE,
        names::ROUTER_TEMPLATE,
        names::NGINX_VALIDATION_SCRIPT,
        names::ETCKEEPER_TEMPLATE,
    ] {
        let target = project.path().join("infra/templates").join(path);
        fs::create_dir_all(target.parent().ok_or_else(|| anyhow::anyhow!("asset has no parent"))?)?;
        fs::write(target, "asset")?;
    }
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    config.domain = "app.example.com".to_owned();
    config.email = "ops@example.com".to_owned();
    let context = ProvisioningContext::from_bones(&config)?;
    let runtime = ProvisioningRuntime::new(&context.server)?;

    let deploy = prepare_ssl(&context, &runtime, &TemplateResolver::new(project.path()))?;
    let names = deploy
        .plan()
        .operations
        .iter()
        .filter_map(|operation| deploy.operation(&operation.id))
        .map(|operation| operation.display_name().to_owned())
        .collect::<Vec<_>>();

    let challenge = names.iter().position(|name| name == "Deploy router nginx config (certbot challenge)");
    let certificate = names.iter().position(|name| name == "Obtain or renew certificate");
    let enable = names.iter().position(|name| name == "Deploy router nginx config (SSL enable)");
    assert!(matches!((challenge, certificate, enable), (Some(a), Some(b), Some(c)) if a < b && b < c));
    assert_eq!(names.last().map(String::as_str), Some(names::ETCKEEPER_RECORD_OPERATION));
    Ok(())
}

#[test]
fn ssl_plan_rejects_untrusted_domain_and_email_values() {
    let mut config = Bones::for_site("example");
    config.host = "server.example.com".to_owned();
    config.domain = "example.com;touch-/tmp/bad".to_owned();
    config.email = "ops@example.com".to_owned();
    assert!(ProvisioningContext::from_bones(&config).is_err());
}
