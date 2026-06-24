use console::style;

pub fn command(command: &str) -> String {
    style(command).cyan().bold().to_string()
}

pub fn next(command: &str) {
    println!("Next: run {}.", self::command(command));
}

pub fn next_raw(command: &str) {
    println!("Next: {}", self::command(command));
}

pub fn then_raw(command: &str) {
    println!("Then: {}", self::command(command));
}

pub fn indented_next(command: &str) {
    println!("  Next: run {}", self::command(command));
}

pub fn indented_next_text(text: &str) {
    println!("  Next: {text}");
}
