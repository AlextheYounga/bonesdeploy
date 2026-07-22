//! Integration test: detect legacy-language smell terms in Rust source.
//!
//! Flags comments, docstrings, identifiers, attributes, and string literals
//! that describe code as `legacy`, `hack`, `workaround`,
//! `backcompat`, or `deprecated`.
//!
//! This is intentionally a high-signal smell check, not a semantic analysis.

use std::fs;
use std::path::{Path, PathBuf};

const IGNORE_DIRS: &[&str] = &["target", "vendor", "node_modules", ".git"];
const LEGACY_TERMS: &[&str] = &["legacy", "hack", "workaround", "backcompat", "deprecated"];

fn collect_source_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = vec![project_root.join("crates")];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !IGNORE_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }

    files
}

fn terms_in_line(line: &str) -> Vec<&'static str> {
    let mut matches = Vec::new();
    let lower = line.to_ascii_lowercase();

    for term in LEGACY_TERMS {
        if lower.contains(term) {
            matches.push(*term);
        }
    }
    matches
}

/// Detects legacy-language smell terms like legacy, hack, or workaround in source code.
#[test]
fn no_legacy_terms() {
    let project_root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let files = collect_source_files(project_root);

    assert!(!files.is_empty(), "No source files found in crates/.");

    let mut all_violations = Vec::new();
    let mut files_scanned = 0;

    for file in &files {
        let Ok(code) = fs::read_to_string(file) else {
            continue;
        };
        files_scanned += 1;

        for (line_number, line) in code.lines().enumerate() {
            let terms = terms_in_line(line);
            if terms.is_empty() {
                continue;
            }

            let relative = file.strip_prefix(project_root).unwrap_or(file);
            all_violations.push(format!(
                "  {}:{}: {} [{}]",
                relative.display(),
                line_number + 1,
                line.trim(),
                terms.join(", ")
            ));
        }
    }

    assert!(files_scanned > 0, "No readable Rust files were found to check.");

    assert!(all_violations.is_empty(), "Legacy-language terms found:\n{}", all_violations.join("\n"));
}
