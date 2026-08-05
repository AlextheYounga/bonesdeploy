use anyhow::Result;
use e2e::project::SampleProject;

use super::harness::Harness;

const SITE: &str = "e2erails";

pub fn provision(harness: &Harness) -> Result<SampleProject> {
    harness.provision(SITE, "rails", &["ruby_version=3.4"])
}

pub fn assert_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SITE)?;
    harness.assert_service("e2erails-puma.service")?;
    harness.assert_route(SITE, SITE)
}

pub fn deploy(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(project)?;
    harness.assert_deployed(SITE)
}
