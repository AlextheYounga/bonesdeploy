use crate::{control_plane, privileges};
use anyhow::Result;

pub fn sync(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote config sync")?;
    let descriptor = control_plane::read_stdin_descriptor()?;
    control_plane::sync(site, &descriptor)
}
