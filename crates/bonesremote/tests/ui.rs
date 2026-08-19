use bonesremote::ui::{failure_marker, success_marker};

#[test]
fn markers_keep_color_when_output_is_piped_over_ssh() {
    assert!(success_marker().contains("\x1b["));
    assert!(failure_marker().contains("\x1b["));
}
