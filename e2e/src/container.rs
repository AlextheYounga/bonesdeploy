//! Incus container lifecycle with cleanup on drop.

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::incus::incus;
use crate::{CONTAINER_PREFIX, keep_artifacts, unique_suffix};

/// A running Incus container, deleted on drop unless `BONES_E2E_KEEP=1`.
pub struct Container {
    name: String,
    keep: bool,
}

impl Container {
    /// Launches a fresh container from `image` with a unique harness name.
    pub fn launch(image: &str) -> Result<Self> {
        let name = format!("{CONTAINER_PREFIX}-{}", unique_suffix());
        incus(&[
            "launch", image, &name,
            "--config", "limits.memory=1GiB",
            "--config", "limits.cpu=1",
        ])?;
        Ok(Self { name, keep: keep_artifacts() })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Runs a bash script inside the container, returning stdout.
    pub fn exec(&self, script: &str) -> Result<String> {
        incus(&["exec", &self.name, "--", "bash", "-c", script])
            .with_context(|| format!("Command failed inside container {}: {script}", self.name))
    }

    /// Pushes a local file into the container at an absolute path.
    pub fn push_file(&self, local: &std::path::Path, remote: &str, mode: &str) -> Result<()> {
        let Some(local) = local.to_str() else {
            bail!("Non-UTF-8 path: {}", local.display());
        };
        incus(&["file", "push", "--mode", mode, local, &format!("{}{remote}", self.name)])?;
        Ok(())
    }

    /// Waits for systemd to finish booting. `degraded` is accepted: minimal
    /// container images routinely have one cosmetic unit failure.
    pub fn wait_ready(&self) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(120);
        let mut last_state = String::new();
        while Instant::now() < deadline {
            let output = Command::new("incus")
                .args(["exec", &self.name, "--", "systemctl", "is-system-running"])
                .output()
                .context("Failed to run incus exec")?;
            last_state = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if last_state == "running" || last_state == "degraded" {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(500));
        }
        bail!("Container {} did not finish booting (last systemd state: {last_state:?})", self.name)
    }

    /// Waits for a systemd unit to report active.
    pub fn wait_active(&self, unit: &str) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            if self.exec(&format!("systemctl is-active --quiet {unit}")).is_ok() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(500));
        }
        bail!("Unit {unit} never became active in container {}", self.name)
    }

    /// Waits for and returns the container's IPv4 address.
    pub fn ipv4(&self) -> Result<String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline {
            if let Some(address) = self.try_ipv4()? {
                return Ok(address);
            }
            thread::sleep(Duration::from_millis(500));
        }
        bail!("Container {} never obtained an IPv4 address", self.name)
    }

    fn try_ipv4(&self) -> Result<Option<String>> {
        let json = incus(&["list", &self.name, "--format", "json"])?;
        let instances: serde_json::Value = serde_json::from_str(&json).context("Failed to parse incus list output")?;

        let networks = instances
            .get(0)
            .and_then(|i| i.get("state"))
            .and_then(|s| s.get("network"))
            .and_then(|n| n.as_object());
        let Some(networks) = networks else { return Ok(None) };

        for (interface, details) in networks {
            if interface == "lo" {
                continue;
            }
            let addresses = details.get("addresses").and_then(|a| a.as_array());
            for address in addresses.into_iter().flatten() {
                if address.get("family").and_then(|f| f.as_str()) == Some("inet")
                    && let Some(ip) = address.get("address").and_then(|a| a.as_str())
                {
                    return Ok(Some(ip.to_string()));
                }
            }
        }
        Ok(None)
    }

    /// Installs an SSH public key for root. Bootstrap later copies root's
    /// authorized_keys to the deploy user, so this key unlocks both.
    pub fn authorize_root_key(&self, public_key: &str) -> Result<()> {
        let key = public_key.trim();
        self.exec(&format!(
            "mkdir -p /root/.ssh && chmod 700 /root/.ssh \
             && printf '%s\\n' '{key}' > /root/.ssh/authorized_keys \
             && chmod 600 /root/.ssh/authorized_keys"
        ))?;
        Ok(())
    }

    /// Stops the container (required before `incus publish`).
    pub fn stop(&self) -> Result<()> {
        incus(&["stop", &self.name])?;
        Ok(())
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        if self.keep {
            eprintln!("{}: keeping container {} for inspection", crate::KEEP_ENV, self.name);
            return;
        }
        if let Err(err) = incus(&["delete", "--force", &self.name]) {
            eprintln!("Failed to clean up container {}: {err}", self.name);
        }
    }
}
