//! Full first-time setup against a fresh Incus container: `bonesdeploy init`
//! followed by `bonesdeploy setup --yes` (bootstrap + runtime + push + doctor),
//! then direct assertions on the provisioned box.

use anyhow::Result;

use e2e::container::Container;
use e2e::project::SampleProject;
use e2e::session::Session;
use e2e::{build, image, incus};

const PROJECT_NAME: &str = "e2edemo";

#[test]
#[ignore = "requires a running Incus daemon; see e2e/README.md"]
fn full_setup_provisions_a_fresh_server() -> Result<()> {
    incus::check_server()?;
    let artifacts = build::artifacts()?;
    let base = image::ensure_base()?;
    let session = Session::create()?;

    let container = Container::launch(&base)?;
    container.wait_ready()?;
    container.authorize_root_key(&session.public_key()?)?;
    container.wait_active("ssh")?;
    // Pre-seed the locally built bonesremote: bootstrap's `command -v bonesremote`
    // guard then skips its cargo-install-from-GitHub path, so the container runs
    // this working tree instead of the published repo.
    container.push_file(&artifacts.bonesremote, "/usr/local/bin/bonesremote", "0755")?;
    let host = container.ipv4()?;

    let project = SampleProject::create(&session)?;
    project.bonesdeploy(
        &session,
        &artifacts.bonesdeploy,
        &[
            "init",
            "--non-interactive",
            "--project-name",
            PROJECT_NAME,
            "--branch",
            "main",
            "--host",
            &host,
            "--template",
            "vue",
        ],
    )?;
    project.bonesdeploy(&session, &artifacts.bonesdeploy, &["setup", "--yes"])?;

    // The box now looks like a provisioned server.
    container.exec("id git")?;
    container.exec("bonesremote version")?;
    container.exec("systemctl is-active --quiet nginx")?;
    container.exec(&format!("test -d /srv/sites/{PROJECT_NAME}"))?;
    Ok(())
}
