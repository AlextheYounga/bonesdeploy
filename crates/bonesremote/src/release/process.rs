use std::fs;
use std::path::Path;
use std::process;

use crate::release::state::DeploymentRecord;

pub fn matches(active: &DeploymentRecord) -> bool {
    let stat = Path::new("/proc").join(active.pid().to_string()).join("stat");
    fs::read_to_string(stat)
        .ok()
        .and_then(|content| start_ticks(&content))
        .is_some_and(|ticks| ticks == active.process_start_ticks())
}

pub fn start_ticks(stat: &str) -> Option<u64> {
    let (_, fields) = stat.rsplit_once(") ")?;
    fields.split_whitespace().nth(19)?.parse().ok()
}

pub fn current_start_ticks() -> Option<u64> {
    let stat = fs::read_to_string(Path::new("/proc").join(process::id().to_string()).join("stat")).ok()?;
    start_ticks(&stat)
}
