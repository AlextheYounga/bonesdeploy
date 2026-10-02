//! Full first-time setup against a fresh Incus container, covering several
//! framework sites sharing one server.

use anyhow::{Context, Result, bail};
use e2e::project::SampleProject;

#[path = "setup/angular.rs"]
mod angular;
#[path = "setup/compose.rs"]
mod compose;
#[path = "setup/django.rs"]
mod django;
#[path = "setup/harness.rs"]
mod harness;
#[path = "setup/laravel.rs"]
mod laravel;
#[path = "setup/next.rs"]
mod next;
#[path = "setup/nuxt.rs"]
mod nuxt;
#[path = "setup/rails.rs"]
mod rails;
#[path = "setup/sveltekit.rs"]
mod sveltekit;
#[path = "setup/vue.rs"]
mod vue;

use harness::Harness;

fn run_native(
    site: &str,
    provision: fn(&Harness) -> Result<SampleProject>,
    assert_running: fn(&Harness) -> Result<()>,
    deploy: fn(&Harness, &SampleProject) -> Result<()>,
) -> Result<()> {
    let h = harness::shared_harness()?;
    let project = provision(&h)?;
    h.assert_artifact_only(site)?;
    assert_running(&h)?;
    deploy(&h, &project)?;
    h.assert_artifact_only(site)?;

    let first_release = h.current_release(site)?;
    project.write("e2e-release-marker.txt", "second release\n")?;
    h.commit(&project, "second artifact release")?;
    h.deploy(&project)?;
    let second_release = h.current_release(site)?;
    if second_release == first_release {
        bail!("{site} second release did not change current");
    }

    project.write("e2e-release-marker.txt", "failed release\n")?;
    h.commit(&project, "failed artifact release")?;
    let failed_deploy = {
        let _sabotage = h.sabotage_nginx(site)?;
        h.deploy(&project)
    };
    if failed_deploy.is_ok() {
        bail!("{site} deployment with a sabotaged nginx unit unexpectedly succeeded");
    }

    let active_release =
        h.current_release(site).context("failed activation did not leave a readable current release")?;
    if active_release != second_release {
        bail!("failed {site} activation restored {active_release:?}, expected second release {second_release:?}");
    }
    h.prune_releases(site, 1)?;
    h.assert_release_removed(&first_release)?;
    h.assert_artifact_only(site)?;
    Ok(())
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn angular() -> Result<()> {
    run_native("e2eangular", angular::provision, angular::assert_running, angular::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn django() -> Result<()> {
    run_native("e2edjango", django::provision, django::assert_running, django::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon with nested Docker; see e2e/README.md"]
fn docker_compose() -> Result<()> {
    let h = harness::shared_harness()?;
    compose::run(&h)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn laravel() -> Result<()> {
    run_native("e2elaravel", laravel::provision, laravel::assert_running, laravel::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn next_server() -> Result<()> {
    run_native("e2enextserver", next::provision_server, next::assert_server_running, next::deploy_server)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn next_static() -> Result<()> {
    run_native("e2enextstatic", next::provision_static, next::assert_static_running, next::deploy_static)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn nuxt_server() -> Result<()> {
    run_native("e2enuxtserver", nuxt::provision_server, nuxt::assert_server_running, nuxt::deploy_server)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn nuxt_static() -> Result<()> {
    run_native("e2enuxtstatic", nuxt::provision_static, nuxt::assert_static_running, nuxt::deploy_static)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn rails() -> Result<()> {
    run_native("e2erails", rails::provision, rails::assert_running, rails::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn sveltekit() -> Result<()> {
    run_native("e2esveltekit", sveltekit::provision, sveltekit::assert_running, sveltekit::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn vue() -> Result<()> {
    run_native("e2evue", vue::provision, vue::assert_running, vue::deploy)
}
