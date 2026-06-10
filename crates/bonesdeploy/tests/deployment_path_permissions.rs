use std::fs;
use std::path::Path;

/// Creates the project root parent directory with traversable permissions before the placeholder release.
#[test]
fn common_role_creates_project_root_parent_before_placeholder_release() {
    let tasks = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/common/tasks/main.yml");
    let content = fs::read_to_string(&tasks);
    assert!(content.is_ok(), "failed to read {}", tasks.display());
    let content = content.unwrap_or_default();

    let expected_task = "- name: Ensure project root parent directory is traversable by deploy hooks\n  ansible.builtin.file:\n    path: \"{{ paths.project_root_parent }}\"\n    state: directory\n    owner: root\n    group: root\n    mode: \"0711\"";

    assert!(
        content.contains(expected_task),
        "common role must pin the shared project root parent before creating placeholder paths\n{content}"
    );

    let parent_idx = content.find("Ensure project root parent directory is traversable by deploy hooks");
    let placeholder_idx = content.find("Ensure placeholder release directory exists");

    assert!(parent_idx.is_some(), "expected explicit project root parent task\n{content}");
    assert!(placeholder_idx.is_some(), "expected placeholder directory task\n{content}");
    assert!(parent_idx < placeholder_idx, "project root parent task must run before placeholder creation\n{content}");
}
