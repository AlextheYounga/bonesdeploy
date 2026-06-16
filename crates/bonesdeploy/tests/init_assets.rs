use std::path::PathBuf;

use std::path::Path;

#[path = "init_assets/apparmor.rs"]
mod apparmor;
#[path = "init_assets/firewall.rs"]
mod firewall;
#[path = "init_assets/removed_modules_tripwires.rs"]
mod removed_modules_tripwires;
#[path = "init_assets/paths.rs"]
mod paths;
#[path = "init_assets/setup_playbook.rs"]
mod setup_playbook;
#[path = "init_assets/templates.rs"]
mod templates;

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
