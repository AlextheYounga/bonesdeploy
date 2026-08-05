use std::process::Command;

use crate::commands::doctor::security::types::{PolicyDecision, SudoEvidence};

pub(crate) fn collect_sudo_policy(user: &str) -> SudoEvidence {
    let mut sudo = Command::new("sudo");
    sudo.args(["-n", "-U", user, "-ll"]);
    sudo.env("LC_ALL", "C");

    let decision = match sudo.output() {
        Ok(output) if output.status.success() && sudo_listing_denies(&output.stdout, &output.stderr) => {
            PolicyDecision::Denied
        }
        Ok(output) if output.status.success() => {
            PolicyDecision::Allowed(command_output(&output.stdout, &output.stderr))
        }
        Ok(output) => classify_sudo_denial(output.status.code(), &output.stdout, &output.stderr),
        Err(error) => PolicyDecision::Unverified(format!("could not execute sudo policy check: {error}")),
    };
    SudoEvidence { user: user.to_string(), decision }
}

fn sudo_listing_denies(stdout: &[u8], stderr: &[u8]) -> bool {
    command_output(stdout, stderr).to_ascii_lowercase().contains("is not allowed to run sudo on")
}

fn classify_sudo_denial(status: Option<i32>, stdout: &[u8], stderr: &[u8]) -> PolicyDecision {
    let stderr = String::from_utf8_lossy(stderr);
    let normalized = stderr.to_ascii_lowercase();
    let inspection_failure = normalized.contains("password is required")
        || normalized.contains("parse error")
        || normalized.contains("syntax error")
        || normalized.contains("unable to initialize")
        || normalized.contains("no valid sudoers")
        || normalized.contains("fatal");
    if status == Some(1) && !inspection_failure {
        PolicyDecision::Denied
    } else {
        let detail = command_output(stdout, stderr.as_bytes());
        PolicyDecision::Unverified(if detail.is_empty() {
            format!("sudo policy check exited with status {status:?} without a policy decision")
        } else {
            format!("sudo policy check failed:\n{detail}")
        })
    }
}

fn command_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("stdout:\n{stdout}\nstderr:\n{stderr}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_sudo_denial, command_output, sudo_listing_denies};
    use crate::commands::doctor::security::types::PolicyDecision;

    #[test]
    fn sudo_policy_denial_is_distinct_from_collector_failure() {
        assert_eq!(
            classify_sudo_denial(Some(1), b"", b"Sorry, user atlas is not allowed to execute '/bin/sh' as root"),
            PolicyDecision::Denied
        );
        assert_eq!(classify_sudo_denial(Some(1), b"", b""), PolicyDecision::Denied);
        assert!(matches!(
            classify_sudo_denial(Some(1), b"", b"sudo: a password is required"),
            PolicyDecision::Unverified(_)
        ));
        assert!(matches!(classify_sudo_denial(Some(2), b"", b"sudo: parse error"), PolicyDecision::Unverified(_)));
    }

    #[test]
    fn sudo_policy_output_keeps_stdout_and_stderr() {
        assert_eq!(
            command_output(b"allowed command", b"audit detail"),
            "stdout:\nallowed command\nstderr:\naudit detail"
        );
    }

    #[test]
    fn sudo_listing_denial_is_not_authority() {
        assert!(sudo_listing_denies(b"User e2evue is not allowed to run sudo on bones-e2e-487602-30de0aa3.\n", b""));
        assert!(!sudo_listing_denies(b"User e2evue may run the following commands on host:\n    (ALL) ALL\n", b""));
    }
}
