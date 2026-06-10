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
    "templates/django/.lib/remote/vars/setup.yml",
    "templates/laravel/.lib/remote/vars/setup.yml",
    "templates/next/.lib/remote/vars/setup.yml",
    "templates/nuxt/.lib/remote/vars/setup.yml",
    "templates/rails/.lib/remote/vars/setup.yml",
    "templates/sveltekit/.lib/remote/vars/setup.yml",
    "templates/vue/.lib/remote/vars/setup.yml",
];

const TEMPLATE_SETUP_PLAYBOOKS: [&str; 7] = [
    "templates/django/.lib/remote/playbooks/setup.yml",
    "templates/laravel/.lib/remote/playbooks/setup.yml",
    "templates/next/.lib/remote/playbooks/setup.yml",
    "templates/nuxt/.lib/remote/playbooks/setup.yml",
    "templates/rails/.lib/remote/playbooks/setup.yml",
    "templates/sveltekit/.lib/remote/playbooks/setup.yml",
    "templates/vue/.lib/remote/playbooks/setup.yml",
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
