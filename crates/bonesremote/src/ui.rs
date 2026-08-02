pub(crate) fn success_marker() -> String {
    console::style("✓").green().bold().force_styling(true).to_string()
}

pub(crate) fn pending_marker() -> String {
    console::style("•").yellow().bold().force_styling(true).to_string()
}

pub(crate) fn failure_marker() -> String {
    console::style("✗").red().bold().force_styling(true).to_string()
}

#[cfg(test)]
mod tests {
    use super::{failure_marker, success_marker};

    #[test]
    fn markers_keep_color_when_output_is_piped_over_ssh() {
        assert!(success_marker().contains("\x1b["));
        assert!(failure_marker().contains("\x1b["));
    }
}
