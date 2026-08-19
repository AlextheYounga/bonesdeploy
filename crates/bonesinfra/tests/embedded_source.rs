use std::collections::HashSet;

use bonesinfra::{embedded_source_paths, embedded_source_version};

#[test]
fn distribution_contains_the_python_package_and_install_metadata() {
    let paths = embedded_source_paths().collect::<HashSet<_>>();

    assert!(paths.contains("pyproject.toml"));
    assert!(paths.contains("README.md"), "pyproject readme reference must be embedded");
    assert!(paths.contains("src/bonesinfra/__main__.py"));
    assert!(paths.contains("src/bonesinfra/project.py"));
}

#[test]
fn distribution_excludes_development_and_derived_trees() {
    for file_path in embedded_source_paths() {
        assert!(
            !file_path.starts_with("docs/")
                && !file_path.starts_with("tests/")
                && !file_path.starts_with(".venv/")
                && !file_path.contains("__pycache__")
                && !file_path.contains(".egg-info"),
            "unexpected embedded file: {file_path}"
        );
    }
}

#[test]
fn distribution_version_is_stable_across_calls() {
    assert_eq!(embedded_source_version(), embedded_source_version());
}
