use anyhow::{Context, Result, bail};

use super::harness::Harness;

const SITE: &str = "composee2e";
const PORT: u16 = 18080;

pub fn run(h: &Harness) -> Result<()> {
    let project = h.provision_compose(SITE, PORT)?;
    h.assert_artifact_only(SITE)?;
    h.deploy(&project)?;
    h.assert_service(&format!("{SITE}-compose.service"))?;
    h.assert_route(SITE, "compose-v1")?;
    h.assert_compose_release_images(SITE)?;
    compose(h, "config --quiet")?;
    compose(h, "exec -T state test -s /data/marker")?;

    let first_release = h.current_release(SITE)?;
    let first_image = h.compose_image_tag(&first_release)?;
    project.write("index.html", "compose-v2\n")?;
    h.commit(&project, "second Compose release")?;
    h.deploy(&project)?;
    h.assert_route(SITE, "compose-v2")?;
    h.assert_compose_release_images(SITE)?;
    compose(h, "exec -T state test -s /data/marker")?;

    let previous = h.exec(&format!("readlink -f /srv/sites/{SITE}/current"))?;
    project.write("compose.override.yaml", FAILED_OVERRIDE)?;
    h.commit(&project, "unhealthy Compose release")?;
    if h.deploy(&project).is_ok() {
        bail!("unhealthy Compose deployment unexpectedly succeeded");
    }
    let active = h.exec(&format!("readlink -f /srv/sites/{SITE}/current"))?;
    if active.trim() != previous.trim() {
        bail!("failed Compose activation did not restore the previous release");
    }
    h.assert_route(SITE, "compose-v2").context("previous Compose release was not restored")?;
    h.prune_releases(SITE, 1)?;
    h.assert_release_removed(&first_release)?;
    h.assert_image_removed(&first_image)?;
    h.assert_artifact_only(SITE)
}

fn compose(h: &Harness, operation: &str) -> Result<String> {
    h.exec(&format!(
        "docker compose --project-name bonesdeploy-{SITE} --project-directory /srv/sites/{SITE}/current \
         --env-file /srv/sites/{SITE}/shared/.env --file /srv/sites/{SITE}/current/compose.yaml \
         --file /srv/sites/{SITE}/current/compose.override.yaml {operation}"
    ))
}

const FAILED_OVERRIDE: &str = r#"services:
  web:
    healthcheck:
      test: ["CMD", "false"]
      interval: 1s
      timeout: 1s
      retries: 2
"#;
