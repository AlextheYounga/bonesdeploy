//! Command-level test support for the `bonesremote` binary.
//!
//! Executes the compiled binary as a child process and captures its result, so
//! argument-validation behavior is asserted at the real CLI boundary.

// Each integration-test target builds this module standalone, so helpers that
// some targets do not use would otherwise look like dead code.
#![allow(dead_code)]

use std::process::{Command, Output};

use anyhow::{Context, Result};

/// Absolute path to the compiled `bonesremote` binary.
pub fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bonesremote")
}

/// Runs `bonesremote` with the given arguments, returning its raw output.
pub fn run(args: &[&str]) -> Result<Output> {
    Command::new(binary()).args(args).output().context("failed to run bonesremote")
}
