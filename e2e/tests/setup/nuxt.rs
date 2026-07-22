use anyhow::Result;
use e2e::project::SampleProject;

use super::harness::Harness;

const STATIC_SITE: &str = "e2enuxtstatic";
const SERVER_SITE: &str = "e2enuxtserver";

pub fn provision_static(harness: &Harness) -> Result<SampleProject> {
    harness.provision(STATIC_SITE, "nuxt", &["is_static=true"])
}

pub fn provision_server(harness: &Harness) -> Result<SampleProject> {
    harness.provision(SERVER_SITE, "nuxt", &["is_static=false"])
}

pub fn assert_static_running(harness: &Harness) -> Result<()> {
    harness.assert_site(STATIC_SITE)?;
    harness.assert_route(STATIC_SITE, STATIC_SITE)
}

pub fn assert_server_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SERVER_SITE)?;
    harness.assert_service("e2enuxtserver-nuxt.service")?;
    harness.assert_route(SERVER_SITE, SERVER_SITE)
}

pub fn deploy_static(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(STATIC_SITE, project)?;
    harness.assert_deployed(STATIC_SITE)
}

pub fn deploy_server(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(SERVER_SITE, project)?;
    harness.assert_deployed(SERVER_SITE)
}
