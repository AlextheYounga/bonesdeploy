use std::path::Path;

use anyhow::Result;
use rust_embed::Embed;

use bonesdeploy_core::paths;

use super::write_asset;

#[derive(Embed)]
#[folder = "./assets/kit/"]
struct Kit;

pub fn kit_asset(path: &str) -> Option<Vec<u8>> {
    Kit::get(path).map(|asset| asset.data.into_owned())
}

pub(super) fn scaffold_deployment_functions(bones_dir: &Path) -> Result<()> {
    let path = format!("{}functions.sh", paths::KIT_DEPLOYMENT_DIR);
    let Some(asset) = Kit::get(&path) else {
        return Ok(());
    };
    write_asset(bones_dir, &path, asset.data.as_ref())
}
