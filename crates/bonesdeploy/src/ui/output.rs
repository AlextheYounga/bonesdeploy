use console::style;

pub fn green_command(command: &str) -> String {
    style(command).green().to_string()
}

pub fn success_marker() -> String {
    style("✓").green().bold().to_string()
}

pub fn pending_marker() -> String {
    style("•").yellow().bold().to_string()
}

pub fn failure_marker() -> String {
    style("✗").red().bold().to_string()
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

pub fn render_remote_doctor_output(report: &str, verbose: bool) -> bool {
    let pending = report.contains("has not been pushed yet");
    if verbose {
        print!("{report}");
        if !report.is_empty() && !report.ends_with('\n') {
            println!();
        }
    } else if pending {
        for line in report.lines().filter(|line| line.contains("has not been pushed yet")) {
            let clean = strip_ansi(line);
            let clean = clean.trim().strip_prefix('•').map_or(clean.trim(), str::trim_start);
            println!("{} {}", pending_marker(), clean);
        }
    }
    pending
}

pub fn strip_ansi(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\x1b' {
            index += 1;
            if index < chars.len() && chars[index] == '[' {
                index += 1;
                while index < chars.len() && !('@'..='~').contains(&chars[index]) {
                    index += 1;
                }
                index += 1;
            }
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}
