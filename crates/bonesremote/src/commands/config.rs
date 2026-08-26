use anyhow::Result;

use crate::{control_plane, privileges};

pub fn sync(site: &str, config_stdin: bool) -> Result<()> {
    privileges::ensure_root("bonesremote config sync")?;
    if !config_stdin {
        anyhow::bail!("bonesremote config sync requires --config-stdin");
    }
    let descriptor = control_plane::read_stdin_descriptor()?;
    control_plane::store(site, &descriptor)
}
