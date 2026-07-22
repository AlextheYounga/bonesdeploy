//! Builds the binaries under test from the local working tree.
//!
//! `bonesdeploy` runs on the host, so a plain debug build works. `bonesremote`
//! runs inside the (Debian) container, so it is built as a static musl binary
//! to sidestep host/container glibc version skew.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::{status_ok, target_dir, workspace_root};

const MUSL_TARGET: &str = "x86_64-unknown-linux-musl";

pub struct Artifacts {
    pub bonesdeploy: PathBuf,
    pub bonesremote: PathBuf,
}

/// Compiles both binaries, streaming cargo output to the terminal.
pub fn artifacts() -> Result<Artifacts> {
    ensure_musl_target()?;

    let root = workspace_root();
    let build = Command::new("cargo")
        .current_dir(root)
        .args(["build", "-p", "bonesdeploy"])
        .status()
        .context("Failed to run cargo build")?;
    status_ok(build, "cargo build -p bonesdeploy")?;

    let build = Command::new("cargo")
        .current_dir(root)
        .args(["build", "-p", "bonesremote", "--target", MUSL_TARGET])
        .status()
        .context("Failed to run cargo build")?;
    status_ok(build, "cargo build -p bonesremote (musl)")?;

    let target = target_dir();
    Ok(Artifacts {
        bonesdeploy: target.join("debug/bonesdeploy"),
        bonesremote: target.join(MUSL_TARGET).join("debug/bonesremote"),
    })
}

fn ensure_musl_target() -> Result<()> {
    let installed =
        Command::new("rustup").args(["target", "list", "--installed"]).output().context("Failed to run rustup")?;
    if String::from_utf8_lossy(&installed.stdout).lines().any(|line| line.trim() == MUSL_TARGET) {
        return Ok(());
    }
    eprintln!("Installing rust target {MUSL_TARGET} (one-time)...");
    let add = Command::new("rustup")
        .args(["target", "add", MUSL_TARGET])
        .status()
        .context("Failed to run rustup target add")?;
    status_ok(add, "rustup target add")
}
