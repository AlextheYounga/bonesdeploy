use anyhow::Result;

use super::harness::Harness;

const SITE: &str = "e2elaravel";
const MARKER: &str = "e2e-laravel-php";
const PHP_VERSION: &str = "8.5";

pub fn provision(harness: &Harness) -> Result<()> {
    harness.provision(SITE, "laravel", &["php_version=8.5"])?;
    harness.write_laravel_probe(SITE, MARKER)
}

pub fn assert_running(harness: &Harness) -> Result<()> {
    harness.assert_site(SITE)?;
    harness.assert_service(&format!("php{PHP_VERSION}-fpm.service"))?;
    harness.assert_route(SITE, MARKER)?;
    harness.assert_owner(&format!("/var/log/bonesdeploy/{SITE}/php-worker-error.log"), &format!("{SITE}:{SITE}"))
}
