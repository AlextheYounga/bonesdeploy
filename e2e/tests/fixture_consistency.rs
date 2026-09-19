use std::fs;
use std::path::Path;

use anyhow::Result;
use e2e::unique_suffix;

#[test]
fn sveltekit_fixture_uses_the_node_adapter() -> Result<()> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/sveltekit.md");
    let output = e2e::target_dir().join(format!("fixture-consistency-{}", unique_suffix()));
    fs::create_dir_all(&output)?;

    let result = (|| {
        mdpack::unpack_from_path(&fixture, Some(&output), mdpack::UnpackOptions::default())
            .map_err(|error| anyhow::anyhow!("Failed to expand {}: {error}", fixture.display()))?;
        let package = fs::read_to_string(output.join("package.json"))?;
        let lock = fs::read_to_string(output.join("package-lock.json"))?;
        let config = fs::read_to_string(output.join("vite.config.ts"))?;

        assert!(package.contains("\"@sveltejs/adapter-node\""));
        assert!(!package.contains("\"@sveltejs/adapter-auto\""));
        assert!(lock.contains("node_modules/@sveltejs/adapter-node"));
        assert!(!lock.contains("node_modules/@sveltejs/adapter-auto"));
        assert!(config.contains("from '@sveltejs/adapter-node'"));
        assert!(!config.contains("from '@sveltejs/adapter-auto'"));
        Ok(())
    })();

    fs::remove_dir_all(&output)?;
    result
}
