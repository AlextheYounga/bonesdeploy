use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

#[path = "../../build_support.rs"]
mod build_support;

fn main() {
    if let Err(error) = build_wheel() {
        eprintln!("{error}");
        exit(1);
    }
}

fn build_wheel() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let python = root.join("python");
    let assets = root.join("assets");

    remove_python_caches(&python)?;
    remove_old_wheels(&assets)?;

    let status = Command::new("uv")
        .args(["build", "--wheel", "--out-dir"])
        .arg(&assets)
        .arg(&python)
        .status()
        .map_err(|error| format!("failed to run uv: {error}"))?;
    if !status.success() {
        return Err("uv failed to build the BonesInfra wheel".to_owned());
    }

    let wheels = generated_wheels(&assets)?;
    let [wheel] = wheels.as_slice() else {
        return Err(format!("expected exactly one generated wheel in {}", assets.display()));
    };
    println!("Built {}", wheel.display());
    Ok(())
}

fn remove_old_wheels(assets: &Path) -> Result<(), String> {
    let previous_wheel = assets.join("bonesinfra.whl");
    if previous_wheel.is_file() {
        fs::remove_file(&previous_wheel)
            .map_err(|error| format!("failed to remove {}: {error}", previous_wheel.display()))?;
    }
    for wheel in generated_wheels(assets)? {
        fs::remove_file(&wheel).map_err(|error| format!("failed to remove {}: {error}", wheel.display()))?;
    }
    Ok(())
}

fn generated_wheels(assets: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(assets).map_err(|error| format!("failed to read {}: {error}", assets.display()))?;
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("bonesinfra-"))
                && path.extension().is_some_and(|extension| extension == "whl")
        })
        .collect())
}

fn remove_python_caches(directory: &Path) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| format!("failed to read {}: {error}", directory.display()))? {
        let path = entry.map_err(|error| format!("failed to read Python build input: {error}"))?.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == build_support::PYTHON_CACHE_DIR) {
                fs::remove_dir_all(&path).map_err(|error| format!("failed to remove {}: {error}", path.display()))?;
            } else {
                remove_python_caches(&path)?;
            }
        }
    }
    Ok(())
}
