use anyhow::Result;

use super::harness::Harness;

const STATIC_SITE: &str = "e2enextstatic";
const SERVER_SITE: &str = "e2enextserver";

pub fn provision_static(harness: &Harness) -> Result<()> {
    harness.provision(STATIC_SITE, "next", &["is_static=true"])
}

pub fn provision_server(harness: &Harness) -> Result<()> {
    harness.provision(SERVER_SITE, "next", &["is_static=false"])
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
