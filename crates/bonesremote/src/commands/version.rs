use shared::paths;

pub fn run() {
    println!("{} {}", paths::BONESREMOTE_BINARY, env!("CARGO_PKG_VERSION"));
}
