use console::style;

pub fn green_command(command: &str) -> String {
    style(command).green().to_string()
}

pub fn run_command(command: &str) -> String {
    format!("run {}", green_command(command))
}

pub fn next_step(command: &str) -> String {
    format!("Next: {}.", run_command(command))
}

pub fn next_step_with_detail(command: &str, detail: &str) -> String {
    format!("Next: {} {detail}.", run_command(command))
}
