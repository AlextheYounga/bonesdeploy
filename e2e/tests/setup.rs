//! Full first-time setup against a fresh Incus container, covering several
//! framework sites sharing one server.

use anyhow::Result;

#[path = "setup/harness.rs"]
mod harness;
#[path = "setup/laravel.rs"]
mod laravel;
#[path = "setup/next.rs"]
mod next;
#[path = "setup/nuxt.rs"]
mod nuxt;
#[path = "setup/vue.rs"]
mod vue;

use harness::Harness;

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn full_setup_provisions_framework_sites_together() -> Result<()> {
    let harness = Harness::create()?;

    vue::provision(&harness)?;
    nuxt::provision_static(&harness)?;
    nuxt::provision_server(&harness)?;
    next::provision_static(&harness)?;
    next::provision_server(&harness)?;
    laravel::provision(&harness)?;

    vue::assert_running(&harness)?;
    nuxt::assert_static_running(&harness)?;
    nuxt::assert_server_running(&harness)?;
    next::assert_static_running(&harness)?;
    next::assert_server_running(&harness)?;
    laravel::assert_running(&harness)?;
    Ok(())
}
