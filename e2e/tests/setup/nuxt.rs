use anyhow::Result;

use super::harness::Harness;

const STATIC_SITE: &str = "e2enuxtstatic";
const SERVER_SITE: &str = "e2enuxtserver";

pub fn provision_static(harness: &Harness) -> Result<()> {
    harness.provision(STATIC_SITE, "nuxt", &["is_static=true"])
}

pub fn provision_server(harness: &Harness) -> Result<()> {
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
