//! Full first-time setup against a fresh Incus container, covering several
//! framework sites sharing one server.

use anyhow::{Context, Result, bail};
use e2e::project::SampleProject;

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

use harness::{BuildMode, Harness};

fn run_local(
    site: &str,
    provision: fn(&Harness, BuildMode) -> Result<SampleProject>,
    assert_running: fn(&Harness) -> Result<()>,
    deploy: fn(&Harness, &SampleProject) -> Result<()>,
) -> Result<()> {
    let h = harness::shared_harness()?;
    let project = provision(&h, BuildMode::Local)?;
    assert_running(&h)?;
    deploy(&h, &project)?;

    let first_release = h.current_release(site)?;
    project.write("e2e-release-marker.txt", "second release\n")?;
    h.commit(&project, "second local artifact release")?;
    h.deploy(&project)?;
    let second_release = h.current_release(site)?;
    if second_release == first_release {
        bail!("{site} second release did not change current");
    }

    project.write("e2e-release-marker.txt", "failed release\n")?;
    h.commit(&project, "failed local artifact release")?;
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
    Ok(())
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn django() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = django::provision(&h, BuildMode::Remote)?;
    django::assert_running(&h)?;
    django::deploy(&h, &project)
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
    let h = harness::shared_harness()?;
    let project = laravel::provision(&h, BuildMode::Remote)?;
    laravel::assert_running(&h)?;
    laravel::deploy(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn next_server() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = next::provision_server(&h, BuildMode::Remote)?;
    next::assert_server_running(&h)?;
    next::deploy_server(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn next_static() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = next::provision_static(&h, BuildMode::Remote)?;
    next::assert_static_running(&h)?;
    next::deploy_static(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn nuxt_server() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = nuxt::provision_server(&h, BuildMode::Remote)?;
    nuxt::assert_server_running(&h)?;
    nuxt::deploy_server(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn nuxt_static() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = nuxt::provision_static(&h, BuildMode::Remote)?;
    nuxt::assert_static_running(&h)?;
    nuxt::deploy_static(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn rails() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = rails::provision(&h, BuildMode::Remote)?;
    rails::assert_running(&h)?;
    rails::deploy(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn sveltekit() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = sveltekit::provision(&h, BuildMode::Remote)?;
    sveltekit::assert_running(&h)?;
    sveltekit::deploy(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn vue() -> Result<()> {
    let h = harness::shared_harness()?;
    let project = vue::provision(&h, BuildMode::Remote)?;
    vue::assert_running(&h)?;
    vue::deploy(&h, &project)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn django_local() -> Result<()> {
    run_local("e2edjango", django::provision, django::assert_running, django::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn laravel_local() -> Result<()> {
    run_local("e2elaravel", laravel::provision, laravel::assert_running, laravel::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn next_server_local() -> Result<()> {
    run_local("e2enextserver", next::provision_server, next::assert_server_running, next::deploy_server)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn next_static_local() -> Result<()> {
    run_local("e2enextstatic", next::provision_static, next::assert_static_running, next::deploy_static)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn nuxt_server_local() -> Result<()> {
    run_local("e2enuxtserver", nuxt::provision_server, nuxt::assert_server_running, nuxt::deploy_server)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn nuxt_static_local() -> Result<()> {
    run_local("e2enuxtstatic", nuxt::provision_static, nuxt::assert_static_running, nuxt::deploy_static)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn rails_local() -> Result<()> {
    run_local("e2erails", rails::provision, rails::assert_running, rails::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn sveltekit_local() -> Result<()> {
    run_local("e2esveltekit", sveltekit::provision, sveltekit::assert_running, sveltekit::deploy)
}

#[test]
#[ignore = "requires a running Incus daemon and local-build Podman; see e2e/README.md"]
fn vue_local() -> Result<()> {
    run_local("e2evue", vue::provision, vue::assert_running, vue::deploy)
}
