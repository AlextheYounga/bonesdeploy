//! Integration test: production Rust files contain no test modules or seams.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_code_lives_outside_production_source_directories() {
    let root = workspace_root();
    let mut violations = Vec::new();

    if let Ok(crates) = fs::read_dir(root.join("crates")) {
        for crate_dir in crates.flatten() {
            scan_source_tree(&crate_dir.path().join("src"), &mut violations);
        }
    }
    scan_source_tree(&root.join("e2e/src"), &mut violations);

    assert!(violations.is_empty(), "Test code found in production sources:\n{}", violations.join("\n"));
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scan_source_tree(directory: &Path, violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_source_tree(&path, violations);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "rs")
            && fs::read_to_string(&path)
                .is_ok_and(|source| source.contains("#[cfg(test)]") || source.contains("#[test]"))
        {
            violations.push(path.display().to_string());
        }
    }
}
