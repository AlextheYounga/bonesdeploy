//! End-to-end test harness for bonesdeploy against real Incus system containers.
//!
//! Incus containers boot a real systemd as PID 1, so `systemd-run`, `systemctl`,
//! AppArmor, and the rest of the remote surface behave like an actual VPS —
//! the things Docker containers get wrong.
//!
//! Tests are `#[ignore]`d so `cargo test --workspace` stays fast; run them with:
//!
//! ```text
//! cargo test -p e2e -- --ignored --test-threads=1
//! ```
//!
//! Environment knobs:
//! - `BONES_E2E_KEEP=1`    — keep containers and session homes after a run (debugging)
//! - `BONES_E2E_REBUILD=1` — rebuild the cached base image
//! - `BONES_E2E_IMAGE=...` — override the upstream image the base is built from

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod build;
pub mod container;
pub mod image;
pub mod incus;
pub mod project;
pub mod session;

pub const KEEP_ENV: &str = "BONES_E2E_KEEP";
pub const REBUILD_ENV: &str = "BONES_E2E_REBUILD";
pub const IMAGE_ENV: &str = "BONES_E2E_IMAGE";

/// All harness-created containers share this prefix so strays are easy to
/// find and delete: `incus list bones-e2e`.
pub const CONTAINER_PREFIX: &str = "bones-e2e";

pub fn keep_artifacts() -> bool {
    env::var_os(KEEP_ENV).is_some()
}

/// Unique-enough suffix for container names and scratch directories.
pub fn unique_suffix() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
    format!("{}-{nanos:08x}", std::process::id())
}

/// Workspace root (this crate lives at `<root>/e2e`).
pub fn workspace_root() -> &'static Path {
    // The manifest dir always has a parent; the fallback is unreachable.
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap_or_else(|| Path::new("."))
}

/// Cargo target directory, honoring `CARGO_TARGET_DIR`.
pub fn target_dir() -> PathBuf {
    env::var_os("CARGO_TARGET_DIR").map_or_else(|| workspace_root().join("target"), PathBuf::from)
}

/// Scratch space for session homes and sample projects; survives in `target/`
/// for post-mortem inspection when `BONES_E2E_KEEP=1`.
pub fn scratch_dir() -> PathBuf {
    target_dir().join("e2e")
}

pub fn status_ok(status: ExitStatus, what: &str) -> anyhow::Result<()> {
    if status.success() { Ok(()) } else { anyhow::bail!("{what} failed ({status})") }
}
