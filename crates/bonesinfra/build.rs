use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::exit;

use zip::ZipArchive;

mod build_support;

fn main() {
    println!("cargo:rerun-if-changed=assets/bonesinfra.whl");
    println!("cargo:rerun-if-changed=python/pyproject.toml");
    println!("cargo:rerun-if-changed=python/src");

    if env::var_os("CARGO_FEATURE_WHEEL_BUILDER").is_some() {
        return;
    }

    if let Err(error) = verify_wheel() {
        eprintln!("{error}");
        exit(1);
    }
}

fn verify_wheel() -> io::Result<()> {
    let wheel_path = Path::new("assets/bonesinfra.whl");
    let wheel_bytes = fs::read(wheel_path).map_err(|error| {
        io::Error::new(error.kind(), format!("failed to read {}: {error}; run cargo build-wheel", wheel_path.display()))
    })?;
    let mut archive = ZipArchive::new(io::Cursor::new(wheel_bytes))
        .map_err(|error| io::Error::other(format!("invalid BonesInfra wheel: {error}")))?;

    let packaged_files = packaged_python_files(&mut archive)?;
    let source_files = source_python_files(Path::new("python/src/bonesinfra"))?;
    if packaged_files != source_files {
        return Err(io::Error::other(
            "BonesInfra wheel is stale: packaged files differ from python/src/bonesinfra; run cargo build-wheel",
        ));
    }

    let expected_version = project_version(Path::new("python/pyproject.toml"))?;
    let wheel_version = wheel_metadata_version(&mut archive)?;
    if expected_version != wheel_version {
        return Err(io::Error::other(format!(
            "BonesInfra wheel version {wheel_version} does not match pyproject.toml version {expected_version}; run cargo build-wheel"
        )));
    }

    Ok(())
}

fn packaged_python_files<R: Read + io::Seek>(archive: &mut ZipArchive<R>) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| io::Error::other(format!("failed to read wheel entry: {error}")))?;
        let name = entry.name().to_string();
        if name.starts_with("bonesinfra/") && !name.ends_with('/') {
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents)?;
            files.insert(name, contents);
        }
    }
    Ok(files)
}

fn source_python_files(root: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    collect_source_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_source_files(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == build_support::PYTHON_CACHE_DIR) {
                continue;
            }
            collect_source_files(root, &path, files)?;
        } else if path.extension().is_none_or(|extension| extension != "pyc" && extension != "pyo") {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| io::Error::other(format!("failed to resolve source path: {error}")))?;
            files.insert(format!("bonesinfra/{}", relative.to_string_lossy().replace('\\', "/")), fs::read(path)?);
        }
    }
    Ok(())
}

fn project_version(path: &Path) -> io::Result<String> {
    let contents = fs::read_to_string(path)?;
    contents
        .lines()
        .find_map(|line| line.strip_prefix("version = \"")?.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| io::Error::other(format!("no project version found in {}", path.display())))
}

fn wheel_metadata_version<R: Read + io::Seek>(archive: &mut ZipArchive<R>) -> io::Result<String> {
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| io::Error::other(format!("failed to read wheel entry: {error}")))?;
        if !entry.name().ends_with(".dist-info/METADATA") {
            continue;
        }
        let mut contents = String::new();
        entry.read_to_string(&mut contents)?;
        if let Some(version) = contents.lines().find_map(|line| line.strip_prefix("Version: ")) {
            return Ok(version.to_owned());
        }
    }
    Err(io::Error::other("BonesInfra wheel has no metadata version"))
}
