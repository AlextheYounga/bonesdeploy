use anyhow::Result;

use super::harness::Harness;

const SITE: &str = "e2evue";
const MARKER: &str = "e2e-vue-static";

pub fn provision(harness: &Harness) -> Result<()> {
    harness.provision(SITE, "vue", &[])?;
    harness.write_placeholder(SITE, "dist", MARKER)
}

pub fn assert_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SITE)?;
    harness.assert_route(SITE, MARKER)
}
