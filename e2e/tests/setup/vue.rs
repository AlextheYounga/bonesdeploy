use anyhow::Result;

use super::harness::Harness;

const SITE: &str = "e2evue";

pub fn provision(harness: &Harness) -> Result<()> {
    harness.provision(SITE, "vue", &[])
}

pub fn assert_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SITE)?;
    harness.assert_route(SITE, SITE)
}
