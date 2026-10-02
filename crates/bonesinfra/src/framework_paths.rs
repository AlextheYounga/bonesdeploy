use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{Result, bail};
use bonesdeploy_core::config::{Runtime, RuntimeBackend};

static FRAMEWORK_PATHS: LazyLock<HashMap<&'static str, &'static [&'static str]>> = LazyLock::new(|| {
    HashMap::from([
        ("angular", &["templates/frameworks/angular"][..]),
        ("custom", &["templates/frameworks/custom"][..]),
        ("django", &["templates/frameworks/django"][..]),
        ("laravel", &["templates/frameworks/laravel"][..]),
        ("next", &["templates/frameworks/next"][..]),
        ("nuxt", &["templates/frameworks/nuxt"][..]),
        ("rails", &["templates/frameworks/rails"][..]),
        ("sveltekit", &["templates/frameworks/sveltekit"][..]),
        ("vue", &["templates/frameworks/vue"][..]),
    ])
});

pub(super) fn prune(infra: &Path, runtime: &Runtime) -> Result<()> {
    let selected = selected_framework(runtime);
    if let Some(framework) = selected
        && !FRAMEWORK_PATHS.contains_key(framework)
    {
        bail!("unknown BonesInfra framework: {framework}");
    }

    for (framework, paths) in FRAMEWORK_PATHS.iter() {
        if selected == Some(*framework) {
            continue;
        }
        for relative_path in *paths {
            super::remove_path(&infra.join(relative_path))?;
        }
    }
    Ok(())
}

fn selected_framework(runtime: &Runtime) -> Option<&str> {
    match runtime.backend {
        RuntimeBackend::Native if runtime.template.is_empty() => Some("custom"),
        RuntimeBackend::Native => Some(&runtime.template),
        RuntimeBackend::Docker => None,
    }
}
