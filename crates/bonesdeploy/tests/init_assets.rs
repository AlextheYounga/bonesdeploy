use std::path::PathBuf;

use std::path::Path;

const TEMPLATES: [&str; 7] = [
    "templates/django/bones.yaml",
    "templates/laravel/bones.yaml",
    "templates/next/bones.yaml",
    "templates/nuxt/bones.yaml",
    "templates/rails/bones.yaml",
    "templates/sveltekit/bones.yaml",
    "templates/vue/bones.yaml",
];

// These probably aren't necessary anymore
const TEMPLATE_SETUP_VARS_FILES: [&str; 7] = [
    "templates/django/runtime/vars/setup.yml",
    "templates/laravel/runtime/vars/setup.yml",
    "templates/next/runtime/vars/setup.yml",
    "templates/nuxt/runtime/vars/setup.yml",
    "templates/rails/runtime/vars/setup.yml",
    "templates/sveltekit/runtime/vars/setup.yml",
    "templates/vue/runtime/vars/setup.yml",
];

#[path = "init_assets/apparmor.rs"]
mod apparmor;
#[path = "init_assets/firewall.rs"]
mod firewall;
#[path = "init_assets/paths.rs"]
mod paths;
#[path = "init_assets/setup_playbook.rs"]
mod setup_playbook;
#[path = "init_assets/templates.rs"]
mod templates;

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
