use anyhow::Result;
use e2e::project::SampleProject;

use super::harness::{BuildMode, Harness};

const SITE: &str = "e2edjango";

pub fn provision(harness: &Harness, build_mode: BuildMode) -> Result<SampleProject> {
    harness.provision(SITE, "django", &["wsgi_module=djangotest.wsgi:application"], build_mode)
}

pub fn assert_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SITE)?;
    harness.assert_service("e2edjango-gunicorn.service")?;
    harness.assert_route(SITE, SITE)
}

pub fn deploy(harness: &Harness, project: &SampleProject) -> Result<()> {
    harness.deploy(project)?;
    harness.assert_deployed(SITE)
}
