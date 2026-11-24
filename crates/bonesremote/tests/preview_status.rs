use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use anyhow::{Result, ensure};
use serde_json::Value;

#[test]
fn status_reports_the_latest_valid_preview_url_from_cloudflared_logs() -> Result<()> {
    let root = tempfile::tempdir()?;
    for (name, script) in [
        ("systemctl", "#!/bin/sh\nprintf '%s\\n' active\n"),
        ("journalctl", "#!/bin/sh\nprintf '%s\\n' \"$BONES_TEST_JOURNAL\"\n"),
    ] {
        let path = root.path().join(name);
        fs::write(&path, script)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    for (journal, expected) in [
        (
            "old https://old.trycloudflare.com\nnoise https://new.trycloudflare.com",
            Some("https://new.trycloudflare.com"),
        ),
        (
            "https://example.com https://bad_host.trycloudflare.com https://valid-preview.trycloudflare.com",
            Some("https://valid-preview.trycloudflare.com"),
        ),
        ("cloudflared started", None),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bonesremote"))
            .args(["status", "--site", "test-preview-status"])
            .env("PATH", root.path())
            .env("BONES_TEST_JOURNAL", journal)
            .output()?;
        ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(report["preview"]["active"], true);
        assert_eq!(report["preview"]["url"].as_str(), expected);
    }
    Ok(())
}
