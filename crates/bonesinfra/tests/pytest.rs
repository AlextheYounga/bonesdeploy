//! Runs the embedded Python package's pytest suite as part of `cargo test`.
//!
//! The venv lives under the cargo target directory so it never collides with a
//! developer's own venv in `python/` and never leaks into the rust-embed assets.
//! Set `BONES_SKIP_PYTEST=1` to skip during tight Rust-only iteration loops.

use std::env;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{Context, Result, bail};

const SKIP_ENV: &str = "BONES_SKIP_PYTEST";

#[test]
fn python_test_suite_passes() -> Result<()> {
    if env::var_os(SKIP_ENV).is_some() {
        eprintln!("skipping Python test suite: {SKIP_ENV} is set");
        return Ok(());
    }

    let python_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("python");
    let venv = venv_dir();
    ensure_venv(&python_dir, &venv)?;

    let mut pytest = Command::new(venv.join("bin/python"));
    pytest.current_dir(&python_dir).args(["-m", "pytest", "--color=yes"]);
    let output = run(pytest, "pytest suite for bonesinfra/python")?;
    report_summary(&output);
    Ok(())
}

/// Writes pytest's final summary line ("N passed in Xs") straight to the stderr
/// file descriptor: libtest only captures Rust's print macros, so this stays
/// visible on success while the full report is reserved for failures.
fn report_summary(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(summary) = stdout.lines().rev().find(|line| !line.trim().is_empty()) else {
        return;
    };
    let summary = strip_ansi(summary);
    let _ = writeln!(io::stderr(), "bonesinfra pytest: {}", summary.trim_matches(['=', ' ']));
}

fn strip_ansi(line: &str) -> String {
    let mut plain = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for follower in chars.by_ref() {
                if follower.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            plain.push(c);
        }
    }
    plain
}

/// Venv location under the cargo target directory, so `cargo clean` resets it.
fn venv_dir() -> PathBuf {
    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"),
        PathBuf::from,
    );
    target.join("bonesinfra-pytest-venv")
}

fn ensure_venv(python_dir: &Path, venv: &Path) -> Result<()> {
    let stamp_file = venv.join(".stamp");
    let stamp = dependency_stamp(python_dir);
    if fs::read_to_string(&stamp_file).is_ok_and(|existing| existing == stamp) {
        return Ok(());
    }

    if venv.exists() {
        fs::remove_dir_all(venv)
            .with_context(|| format!("Failed to reset the pytest venv at {}", venv.display()))?;
    }

    let mut create = Command::new("python3");
    create.arg("-m").arg("venv").arg(venv);
    run(create, "python3 -m venv (python3 >= 3.12 is required to test bonesinfra)")?;

    let mut install = Command::new(venv.join("bin/python"));
    install.current_dir(python_dir).args(["-m", "pip", "install", "--quiet", "-e", ".", "pytest"]);
    run(install, "pip install of bonesinfra and pytest into the test venv")?;

    // Written last so an interrupted setup rebuilds from scratch on the next run.
    fs::write(&stamp_file, stamp)
        .with_context(|| format!("Failed to write the pytest venv stamp at {}", stamp_file.display()))?;
    Ok(())
}

/// Stamp over the dependency manifests; source changes need no rebuild because
/// the package is installed editable.
fn dependency_stamp(python_dir: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    for name in ["pyproject.toml", "uv.lock"] {
        if let Ok(bytes) = fs::read(python_dir.join(name)) {
            name.hash(&mut hasher);
            bytes.hash(&mut hasher);
        }
    }
    format!("{:016x}", hasher.finish())
}

fn run(mut command: Command, description: &str) -> Result<Output> {
    let output = command.output().with_context(|| format!("Failed to start {description}"))?;
    if !output.status.success() {
        bail!(
            "{description} failed ({}).\n\n--- stdout ---\n{}\n--- stderr ---\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(output)
}
