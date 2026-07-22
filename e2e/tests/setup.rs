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

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn laravel() -> Result<()> {
    let h = harness::shared_harness()?;
    laravel::provision(&h)?;
    laravel::assert_running(&h)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn next_server() -> Result<()> {
    let h = harness::shared_harness()?;
    next::provision_server(&h)?;
    next::assert_server_running(&h)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn next_static() -> Result<()> {
    let h = harness::shared_harness()?;
    next::provision_static(&h)?;
    next::assert_static_running(&h)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn nuxt_server() -> Result<()> {
    let h = harness::shared_harness()?;
    nuxt::provision_server(&h)?;
    nuxt::assert_server_running(&h)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn nuxt_static() -> Result<()> {
    let h = harness::shared_harness()?;
    nuxt::provision_static(&h)?;
    nuxt::assert_static_running(&h)
}

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn vue() -> Result<()> {
    let h = harness::shared_harness()?;
    vue::provision(&h)?;
    vue::assert_running(&h)
}
