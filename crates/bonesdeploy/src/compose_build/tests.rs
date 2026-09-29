use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use super::{
    ComposeCommand, ComposeFiles, image_save_command, image_tag_command, inventory, persist_no_clobber,
    reserve_artifact_paths, source_images, write_new_file,
};
use bonesdeploy_core::artifact;
use bonesdeploy_core::config::Bones;

const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

fn config() -> Bones {
    let mut config = Bones::default();
    config.project_name = String::from("atlas");
    config
}

#[test]
fn compose_commands_use_a_stable_project_name_clean_environment_and_linux_amd64() -> anyhow::Result<()> {
    let source = tempfile::tempdir()?;
    let environment = tempfile::NamedTempFile::new()?;
    let files = ComposeFiles { paths: vec![source.path().join("compose.yaml")] };
    let command =
        ComposeCommand { project_directory: source.path(), environment_file: environment.path(), files: &files }
            .command(&config(), ["build"])?;
    let args: Vec<_> = command.get_args().map(|value| value.to_string_lossy().into_owned()).collect();

    assert_eq!(command.get_program().to_string_lossy(), "docker");
    assert!(args.windows(2).any(|pair| pair == ["--project-name", "bonesdeploy-atlas"]));
    assert!(args.windows(2).any(|pair| pair == ["--env-file", environment.path().to_string_lossy().as_ref()]));
    assert_eq!(
        command.get_envs().find(|(key, _)| *key == "DOCKER_DEFAULT_PLATFORM").and_then(|(_, value)| value),
        Some("linux/amd64".as_ref())
    );
    assert!(command.get_envs().all(|(key, value)| key == "DOCKER_DEFAULT_PLATFORM" || value.is_none()));
    Ok(())
}

#[test]
fn inventory_includes_built_and_pulled_services_with_immutable_tags() -> anyhow::Result<()> {
    let config = config();
    let images =
        inventory(&config, REVISION, br#"{"services":{"api":{"image":"local-api"},"redis":{"image":"redis:7"}}}"#)?;

    assert_eq!(images.iter().map(|image| image.service.as_str()).collect::<Vec<_>>(), ["api", "redis"]);
    assert_eq!(images[0].tag, format!("bonesdeploy-atlas-api-{REVISION}"));
    Ok(())
}

#[test]
fn inventory_rejects_services_that_override_the_linux_amd64_target() {
    let result = inventory(&config(), REVISION, br#"{"services":{"web":{"image":"web","platform":"linux/arm64"}}}"#);

    assert!(result.is_err());
}

#[test]
fn source_images_uses_compose_generated_identity_for_build_only_services() -> anyhow::Result<()> {
    let images = source_images(&config(), br#"{"services":{"web":{"build":{}},"redis":{"image":"redis:7"}}}"#)?;

    assert_eq!(images.get("web"), Some(&String::from("bonesdeploy-atlas-web")));
    assert_eq!(images.get("redis"), Some(&String::from("redis:7")));
    Ok(())
}

#[test]
fn compose_file_discovery_rejects_symlinks() -> anyhow::Result<()> {
    let source = tempfile::tempdir()?;
    let target = tempfile::NamedTempFile::new()?;
    symlink(target.path(), source.path().join("compose.yaml"))?;

    let error = ComposeFiles::discover(source.path())
        .err()
        .ok_or_else(|| anyhow::anyhow!("symlinked Compose files must be rejected"))?;

    assert!(error.to_string().contains("must not be a symlink"));
    Ok(())
}

#[test]
fn generated_override_selects_each_immutable_release_image() -> anyhow::Result<()> {
    let images = inventory(&config(), REVISION, br#"{"services":{"web":{"image":"web"}}}"#)?;

    assert_eq!(
        artifact::compose_override_contents(&images),
        format!("services:\n  web:\n    image: bonesdeploy-atlas-web-{REVISION}\n")
    );
    Ok(())
}

#[test]
fn image_commands_tag_and_save_exact_inventory_tags() -> anyhow::Result<()> {
    let images = inventory(&config(), REVISION, br#"{"services":{"web":{"image":"web"}}}"#)?;
    let tag = image_tag_command("web", &images[0].tag);
    let save = image_save_command(Path::new(artifact::COMPOSE_IMAGE_ARCHIVE_FILE), &images);

    assert_eq!(
        tag.get_args().map(|value| value.to_string_lossy()).collect::<Vec<_>>(),
        ["image", "tag", "web", images[0].tag.as_str()]
    );
    assert_eq!(
        save.get_args().map(|value| value.to_string_lossy()).collect::<Vec<_>>(),
        ["image", "save", "--output", artifact::COMPOSE_IMAGE_ARCHIVE_FILE, images[0].tag.as_str()]
    );
    assert!(artifact::COMPOSE_OVERRIDE_FILE.ends_with(".yaml"));
    Ok(())
}

#[test]
fn reserved_compose_paths_reject_existing_files_directories_and_symlinks() -> anyhow::Result<()> {
    let names =
        [artifact::COMPOSE_OVERRIDE_FILE, artifact::COMPOSE_IMAGE_INVENTORY_FILE, artifact::COMPOSE_IMAGE_ARCHIVE_FILE];

    for name in names {
        let file_context = tempfile::tempdir()?;
        fs::write(file_context.path().join(name), b"committed")?;
        assert!(reserve_artifact_paths(file_context.path()).is_err(), "file conflict was accepted for {name}");

        let directory_context = tempfile::tempdir()?;
        fs::create_dir(directory_context.path().join(name))?;
        assert!(
            reserve_artifact_paths(directory_context.path()).is_err(),
            "directory conflict was accepted for {name}"
        );

        let symlink_context = tempfile::tempdir()?;
        let target = tempfile::NamedTempFile::new()?;
        symlink(target.path(), symlink_context.path().join(name))?;
        assert!(reserve_artifact_paths(symlink_context.path()).is_err(), "symlink conflict was accepted for {name}");
    }
    Ok(())
}

#[test]
fn generated_files_are_created_without_clobbering_existing_paths() -> anyhow::Result<()> {
    let source = tempfile::tempdir()?;
    let path = source.path().join(artifact::COMPOSE_OVERRIDE_FILE);
    write_new_file(&path, b"first")?;
    assert_eq!(fs::read(&path)?, b"first");
    assert!(write_new_file(&path, b"second").is_err());
    assert_eq!(fs::read(&path)?, b"first");
    Ok(())
}

#[test]
fn archive_persistence_is_no_clobber_and_keeps_the_complete_temp_file() -> anyhow::Result<()> {
    let source = tempfile::tempdir()?;
    let temporary = source.path().join("temporary.tar");
    let destination = source.path().join(artifact::COMPOSE_IMAGE_ARCHIVE_FILE);
    fs::write(&temporary, b"complete image archive")?;

    persist_no_clobber(&temporary, &destination)?;
    assert_eq!(fs::read(&destination)?, b"complete image archive");
    assert!(persist_no_clobber(&temporary, &destination).is_err());
    Ok(())
}
