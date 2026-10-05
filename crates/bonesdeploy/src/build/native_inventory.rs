use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::build_contract::SOURCE_MOUNT;

use super::{command, docker::DockerClient, inventory::BuildInventory};

pub(super) fn read(
    docker: &DockerClient,
    source: &Path,
    container: &str,
    timeout: Option<u64>,
) -> Result<BuildInventory> {
    let mut command = docker.command()?;
    command.current_dir(source).args([
        "exec",
        container,
        "find",
        "-P",
        SOURCE_MOUNT,
        "-mindepth",
        "1",
        "-printf",
        "%P\\0%y\\0%m\\0",
    ]);
    let output = command::output(command, "inspect Linux builder output metadata", timeout)?;
    BuildInventory::from_builder_output(&output.stdout)
}
