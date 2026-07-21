use std::path::Path;

use anyhow::Result;
use rust_embed::Embed;

use shared::paths;

use super::write_asset;

#[derive(Embed)]
#[folder = "./kit/"]
pub(super) struct Kit;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }

    Ok(())
}

pub(super) fn scaffold_deployment_functions(bones_dir: &Path) -> Result<()> {
    let path = format!("{}functions.sh", paths::KIT_DEPLOYMENT_DIR);
    let Some(asset) = Kit::get(&path) else {
        return Ok(());
    };
    write_asset(bones_dir, &path, asset.data.as_ref())
}

#[cfg(test)]
mod tests {
    use super::Kit;

    #[test]
    fn node_install_extracts_a_cold_cache_archive() -> anyhow::Result<()> {
        use std::env;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;

        let temp = tempfile::tempdir()?;
        let archive_root = temp.path().join("archive-root");
        let node_root = archive_root.join("node-v1.2.3-linux-x64/bin");
        fs::create_dir_all(&node_root)?;
        let node = node_root.join("node");
        fs::write(&node, "#!/bin/sh\nprintf 'v1.2.3\\n'\n")?;
        fs::set_permissions(&node, PermissionsExt::from_mode(0o755))?;

        let archive = temp.path().join("node-v1.2.3-linux-x64.tar.xz");
        let archive_status = Command::new("tar")
            .current_dir(temp.path())
            .args(["-cJf"])
            .arg(&archive)
            .args(["-C"])
            .arg(&archive_root)
            .arg("node-v1.2.3-linux-x64")
            .status()?;
        assert!(archive_status.success(), "failed to create Node archive fixture");

        let checksum = Command::new("sha256sum").current_dir(temp.path()).arg(&archive).output()?;
        assert!(checksum.status.success(), "failed to checksum Node archive fixture");
        let checksum_hash = String::from_utf8(checksum.stdout)?
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Node archive checksum fixture was empty"))?
            .to_string();
        let checksum_line = format!("{checksum_hash}  node-v1.2.3-linux-x64.tar.xz\n");
        let checksums = temp.path().join("SHASUMS256.txt");
        fs::write(&checksums, checksum_line)?;

        let fake_bin = temp.path().join("bin");
        fs::create_dir(&fake_bin)?;
        let fake_curl = fake_bin.join("curl");
        fs::write(
            &fake_curl,
            "#!/bin/sh\noutput=\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-o\" ]; then output=$2; shift 2; else shift; fi\ndone\ncase $output in\n  *SHASUMS256.txt) cp \"$FIXTURE_CHECKSUMS\" \"$output\" ;;\n  *) cp \"$FIXTURE_ARCHIVE\" \"$output\" ;;\nesac\n",
        )?;
        fs::set_permissions(&fake_curl, PermissionsExt::from_mode(0o755))?;

        let cache = temp.path().join("cache");
        let functions = Kit::get("deployment/functions.sh").ok_or_else(|| anyhow::anyhow!("missing functions.sh"))?;
        let functions_file = temp.path().join("functions.sh");
        fs::write(&functions_file, functions.data.as_ref())?;

        let current_path = env::var("PATH").unwrap_or_default();
        let script = "source \"$FUNCTIONS_FILE\"\nnode_install 1.2.3 x64\n";
        let status = Command::new("bash")
            .current_dir(temp.path())
            .arg("-c")
            .arg(script)
            .env("FUNCTIONS_FILE", &functions_file)
            .env("BUILD_CACHE_DIR", &cache)
            .env("FIXTURE_ARCHIVE", &archive)
            .env("FIXTURE_CHECKSUMS", &checksums)
            .env("PATH", format!("{}:{current_path}", fake_bin.display()))
            .status()?;
        assert!(status.success(), "Node fixture installation failed");
        assert!(cache.join("node/v1.2.3-linux-x64/bin/node").is_file());

        Ok(())
    }
}
