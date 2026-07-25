//! Cached base image: an apt-based system container with sshd ready, so every
//! test starts from "fresh VPS with SSH access" without paying the apt setup
//! per run. Rebuild with `BONES_E2E_REBUILD=1`.

use std::env;

use anyhow::Result;

use crate::container::Container;
use crate::incus::incus;
use crate::{IMAGE_ENV, REBUILD_ENV};

pub const BASE_ALIAS: &str = "bonesdeploy-e2e-base";
const DEFAULT_UPSTREAM: &str = "images:debian/13";

/// Returns a launchable image reference for the prepared base, building and
/// caching it on first use.
pub fn ensure_base() -> Result<String> {
    let exists = image_exists(BASE_ALIAS)?;
    if env::var_os(REBUILD_ENV).is_some() && exists {
        incus(&["image", "delete", BASE_ALIAS])?;
    } else if exists {
        return Ok(BASE_ALIAS.to_string());
    }
    build_base()?;
    Ok(BASE_ALIAS.to_string())
}

fn image_exists(alias: &str) -> Result<bool> {
    // `image show` resolves aliases directly; `image list <alias>` filter
    // matching is unreliable for this.
    Ok(incus(&["image", "show", &format!("local:{alias}")]).is_ok())
}

fn build_base() -> Result<()> {
    let upstream = env::var(IMAGE_ENV).unwrap_or_else(|_| DEFAULT_UPSTREAM.to_string());
    eprintln!("Building e2e base image from {upstream} (cached as {BASE_ALIAS})...");

    let container = Container::launch(&upstream)?;
    container.wait_ready()?;
    container.exec(
        "apt-get update \
         && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends openssh-server \
         && systemctl enable ssh",
    )?;
    container.stop()?;
    incus(&["publish", container.name(), "--alias", BASE_ALIAS])?;
    Ok(())
}
