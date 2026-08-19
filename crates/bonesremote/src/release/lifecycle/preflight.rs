use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::release::SiteMutation;
use crate::release::state::release_dir;

/// Verifies the new release is safe to cut over to, while the old release still
/// serves traffic. Any failure here aborts the deployment (Phase A gate): it
/// runs the `nginx -t` reload gate and confirms the new release's web root
/// exists, so we never cut over to a release that cannot be reached or whose
/// nginx configuration would reload a broken daemon.
pub fn validate_ready(mutation: &SiteMutation, release: &str, nginx_test: impl Fn() -> Result<()>) -> Result<()> {
    let web_root = release_web_root(mutation, release);
    if !web_root.is_dir() {
        bail!("New release web root does not exist: {}", web_root.display());
    }
    nginx_test()
}

/// The configured web root for a release, i.e. `<root>/<release>/<web_root>`.
fn release_web_root(mutation: &SiteMutation, release: &str) -> PathBuf {
    release_dir(&mutation.config().project_root, release).join(&mutation.config().runtime.web_root)
}

/// Runs `nginx -t` against the configuration used by the site's nginx service.
/// Its root resolves through the `current` symlink, so a syntax/config failure
/// is caught here before the cut-over restart re-loads nginx.
pub fn run_nginx_test(site: &str) -> Result<()> {
    let config_path = site_nginx_config(site);
    let output = Command::new("nginx")
        .args(["-t", "-c"])
        .arg(&config_path)
        .output()
        .with_context(|| format!("Failed to run `nginx -t -c {}`", config_path.display()))?;
    if output.status.success() {
        println!("nginx -t passed: {}", config_path.display());
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    bail!("nginx -t failed before reload for {}:\n{}", config_path.display(), detail.trim())
}

pub fn site_nginx_config(site: &str) -> PathBuf {
    PathBuf::from(paths::DEFAULT_CONF_ROOT_PARENT).join(site).join(paths::NGINX_CONF)
}
