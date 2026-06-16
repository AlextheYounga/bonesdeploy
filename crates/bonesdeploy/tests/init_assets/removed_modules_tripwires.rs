use std::fs;

use super::project_root;

/// The old embeds/runtimes directory must not exist — runtime templates live in infra/ now.
#[test]
fn old_embeds_runtimes_directory_is_removed() {
    let old_path = project_root().join("crates/bonesdeploy/embeds/runtimes");
    assert!(!old_path.exists(), "old embeds/runtimes/ must be removed: {old_path:?}");
}

/// The old embeds/kit directory must not exist — kit lives in crates/bonesdeploy/kit/ now.
#[test]
fn old_embeds_kit_directory_is_removed() {
    let old_path = project_root().join("crates/bonesdeploy/embeds/kit");
    assert!(!old_path.exists(), "old embeds/kit/ must be removed: {old_path:?}");
}

/// embedded.rs must not define a Runtimes struct (removed in migration).
#[test]
fn embedded_rs_no_runtimes_struct() {
    let path = project_root().join("crates/bonesdeploy/src/embedded.rs");
    let content = fs::read_to_string(&path).unwrap_or_default();
    assert!(!content.contains("struct Runtimes"), "embedded.rs must not define Runtimes struct");
}

/// embedded.rs must not define scaffold_runtime_template (removed in migration).
#[test]
fn embedded_rs_no_scaffold_runtime_template() {
    let path = project_root().join("crates/bonesdeploy/src/embedded.rs");
    let content = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        !content.contains("fn scaffold_runtime_template"),
        "embedded.rs must not define scaffold_runtime_template"
    );
}

/// embedded.rs must not define read_template_runtime_config (removed in migration).
#[test]
fn embedded_rs_no_read_template_runtime_config() {
    let path = project_root().join("crates/bonesdeploy/src/embedded.rs");
    let content = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        !content.contains("fn read_template_runtime_config"),
        "embedded.rs must not define read_template_runtime_config"
    );
}

/// embedded.rs must not define available_templates (removed in migration).
#[test]
fn embedded_rs_no_available_templates() {
    let path = project_root().join("crates/bonesdeploy/src/embedded.rs");
    let content = fs::read_to_string(&path).unwrap_or_default();
    assert!(!content.contains("fn available_templates"), "embedded.rs must not define available_templates");
}

/// The old operations.py path (runtime entrypoint) must not exist in the scaffold.
#[test]
fn no_operations_py_runtime_entrypoint() {
    let old_paths = [
        project_root().join("infra/src/operations.py"),
        project_root().join("infra/runtime/operations.py"),
    ];
    for path in &old_paths {
        assert!(!path.exists(), "old operations.py runtime entrypoint must not exist: {path:?}");
    }
}
