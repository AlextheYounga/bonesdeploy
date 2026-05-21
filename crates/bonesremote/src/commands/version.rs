use crate::config;

pub fn run() {
    println!("{} {}", config::Constants::BINARY_NAME, env!("CARGO_PKG_VERSION"));
}
