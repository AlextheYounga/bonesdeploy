#[path = "ssh_transport/fake_process.rs"]
mod fake_process;
#[cfg(unix)]
#[path = "ssh_transport/sshd.rs"]
mod sshd;

use std::time::Duration;

#[cfg(unix)]
use anyhow::Result;
use bonesdeploy::infra::ssh::TransportPolicy;

const TEST_DEADLINE: Duration = Duration::from_millis(250);

fn short_policy() -> TransportPolicy {
    TransportPolicy {
        connect_deadline: TEST_DEADLINE,
        command_deadline: TEST_DEADLINE,
        transfer_deadline: TEST_DEADLINE,
        remote_output_tail_limit: 64 * 1024,
        command_output_limit: 64 * 1024,
    }
}

#[cfg(unix)]
fn expected_error<T>(result: Result<T>, description: &str) -> Result<anyhow::Error> {
    result.err().ok_or_else(|| anyhow::anyhow!("{description}"))
}
