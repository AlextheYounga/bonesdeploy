pub fn success_marker() -> String {
    console::style("✓").green().bold().force_styling(true).to_string()
}

pub fn pending_marker() -> String {
    console::style("•").yellow().bold().force_styling(true).to_string()
}

pub fn failure_marker() -> String {
    console::style("✗").red().bold().force_styling(true).to_string()
}
