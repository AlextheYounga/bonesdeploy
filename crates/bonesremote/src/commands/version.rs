use bonesdeploy_core::paths;

pub fn run() {
    println!("{} {}", paths::bonesremote_binary(), env!("CARGO_PKG_VERSION"));
}
