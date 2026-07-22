use anyhow::Result;
use e2e::project::SampleProject;

use super::harness::Harness;

const SITE: &str = "e2elaravel";
const MARKER: &str = "e2e-laravel-php";
const PHP_VERSION: &str = "8.5";

pub fn provision(harness: &Harness) -> Result<SampleProject> {
    let project = harness.provision(SITE, "laravel", &["php_version=8.5"])?;
    harness.write_laravel_probe(SITE, MARKER)?;
    Ok(project)
}

pub fn assert_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SITE)?;
    harness.assert_service(&format!("php{PHP_VERSION}-fpm.service"))?;
    harness.assert_route(SITE, MARKER)?;
    harness.assert_owner(&format!("/var/log/bonesdeploy/{SITE}/php-worker-error.log"), &format!("{SITE}:{SITE}"))
}

pub fn deploy(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(SITE, project)?;
    harness.assert_deployed(SITE)?;
    harness.assert_owner(&format!("/var/log/bonesdeploy/{SITE}/php-worker-error.log"), &format!("{SITE}:{SITE}"))
}
