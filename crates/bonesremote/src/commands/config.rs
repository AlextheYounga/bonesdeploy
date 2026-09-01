use anyhow::Result;

use crate::{control_plane, privileges};

pub fn sync(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote config sync")?;
    let descriptor = control_plane::read_stdin_descriptor()?;
    control_plane::store(site, &descriptor)
}
