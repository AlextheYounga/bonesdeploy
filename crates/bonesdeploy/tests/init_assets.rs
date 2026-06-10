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

const TEMPLATE_SETUP_VARS_FILES: [&str; 7] = [
    "templates/django/remote/vars/setup.yml",
    "templates/laravel/remote/vars/setup.yml",
    "templates/next/remote/vars/setup.yml",
    "templates/nuxt/remote/vars/setup.yml",
    "templates/rails/remote/vars/setup.yml",
    "templates/sveltekit/remote/vars/setup.yml",
    "templates/vue/remote/vars/setup.yml",
];

const TEMPLATE_SETUP_PLAYBOOKS: [&str; 7] = [
    "templates/django/remote/playbooks/setup.yml",
    "templates/laravel/remote/playbooks/setup.yml",
    "templates/next/remote/playbooks/setup.yml",
    "templates/nuxt/remote/playbooks/setup.yml",
    "templates/rails/remote/playbooks/setup.yml",
    "templates/sveltekit/remote/playbooks/setup.yml",
    "templates/vue/remote/playbooks/setup.yml",
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
