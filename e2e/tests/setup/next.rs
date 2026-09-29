use anyhow::Result;
use e2e::project::SampleProject;

use super::harness::{BuildMode, Harness};

const STATIC_SITE: &str = "e2enextstatic";
const SERVER_SITE: &str = "e2enextserver";

pub fn provision_static(harness: &Harness, build_mode: BuildMode) -> Result<SampleProject> {
    harness.provision(STATIC_SITE, "next", &["is_static=true"], build_mode)
}

pub fn provision_server(harness: &Harness, build_mode: BuildMode) -> Result<SampleProject> {
    harness.provision(SERVER_SITE, "next", &["is_static=false"], build_mode)
}

pub fn assert_static_running(harness: &Harness) -> Result<()> {
    harness.assert_site(STATIC_SITE)?;
    harness.assert_route(STATIC_SITE, STATIC_SITE)
}

pub fn assert_server_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SERVER_SITE)?;
    harness.assert_service("e2enextserver-next.service")?;
    harness.assert_route(SERVER_SITE, SERVER_SITE)
}

pub fn deploy_static(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(project)?;
    harness.assert_deployed(STATIC_SITE)
}

pub fn deploy_server(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(project)?;
    harness.assert_deployed(SERVER_SITE)
}
