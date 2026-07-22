use anyhow::Result;

use super::harness::Harness;

const STATIC_SITE: &str = "e2enuxtstatic";
const STATIC_MARKER: &str = "e2e-nuxt-static";
const SERVER_SITE: &str = "e2enuxtserver";
const SERVER_MARKER: &str = "e2e-nuxt-server";

pub fn provision_static(harness: &Harness) -> Result<()> {
    harness.provision(STATIC_SITE, "nuxt", &["is_static=true"])?;
    harness.write_placeholder(STATIC_SITE, ".output/public", STATIC_MARKER)
}

pub fn provision_server(harness: &Harness) -> Result<()> {
    harness.provision(SERVER_SITE, "nuxt", &["is_static=false"])?;
    harness.write_placeholder(SERVER_SITE, "public", SERVER_MARKER)
}

pub fn assert_static_running(harness: &Harness) -> Result<()> {
    harness.assert_site(STATIC_SITE)?;
    harness.assert_route(STATIC_SITE, STATIC_MARKER)
}

pub fn assert_server_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SERVER_SITE)?;
    harness.assert_service("e2enuxtserver-nuxt.service")?;
    harness.assert_route(SERVER_SITE, SERVER_MARKER)
}
