use std::fs;
use std::path::Path;

/// Creates the project root parent directory with traversable permissions before the placeholder release.
#[test]
fn setup_deploy_creates_project_root_parent_before_placeholder() {
    let deploy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/infra/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("project_root_parent") && content.contains("0711"),
        "setup deploy must create project root parent with traversable permissions\n{content}"
    );
    assert!(
        content.contains("placeholder_web_root") && content.contains("0750"),
        "setup deploy must create placeholder release directory after parent path\n{content}"
    );

    let parent_idx = content.find("project_root_parent");
    let placeholder_idx = content.find("placeholder_web_root");

    assert!(parent_idx.is_some(), "expected explicit project root parent task\n{content}");
    assert!(placeholder_idx.is_some(), "expected placeholder directory task\n{content}");
    assert!(parent_idx < placeholder_idx, "project root parent task must run before placeholder creation\n{content}");
}
