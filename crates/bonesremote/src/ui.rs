pub(crate) fn success_marker() -> String {
    console::style("✓").green().bold().to_string()
}

pub(crate) fn pending_marker() -> String {
    console::style("•").yellow().bold().to_string()
}

pub(crate) fn failure_marker() -> String {
    console::style("✗").red().bold().to_string()
}
