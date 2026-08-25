use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use bonesremote::release::lifecycle::prepare::{join_prepare_input, list_scripts, stream_prepare_input};
use bonesremote::release::output;

fn temp_dir(prefix: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn run_composed_bash(functions: &Path, script: &Path, log_path: &Path) -> Result<ExitStatus> {
    let functions_file = fs::File::open(functions)?;
    let script_file = fs::File::open(script)?;
    let mut child =
        Command::new("bash").arg("-s").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stdin = child.stdin.take().context("Failed to capture test Bash stdin")?;
    let input_handle = thread::spawn(move || stream_prepare_input(stdin, functions_file, script_file));
    let status = output::stream_child_output(&mut child, log_path, "composed prepare test")?;
    join_prepare_input(input_handle, functions, script)?;
    Ok(status)
}

#[test]
fn list_scripts_sorts_prepare_scripts() -> Result<()> {
    let root = temp_dir("bonesremote-prepare-list")?;
    fs::write(root.join("02_second.sh"), "")?;
    fs::write(root.join("01_first.sh"), "")?;
    fs::write(root.join("README.md"), "# Prepare Scripts")?;
    fs::create_dir_all(root.join("nested"))?;

    let scripts = list_scripts(&root)?;

    assert_eq!(scripts, vec![root.join("01_first.sh"), root.join("02_second.sh")]);

    fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn shared_functions_precede_prepare_script() -> Result<()> {
    let root = temp_dir("bonesremote-prepare-prelude")?;
    let functions = root.join("functions.sh");
    let script = root.join("01_prepare.sh");
    let log = root.join("prepare.log");
    fs::write(&functions, "log() { printf 'helper: %s\\n' \"$*\"; }")?;
    fs::write(&script, "log ready")?;

    let status = run_composed_bash(&functions, &script, &log)?;

    assert!(status.success());
    assert_eq!(fs::read_to_string(log)?, "helper: ready\n");
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn failing_prepare_preserves_status_and_output() -> Result<()> {
    let root = temp_dir("bonesremote-prepare-failure")?;
    let functions = root.join("functions.sh");
    let script = root.join("01_prepare.sh");
    let log = root.join("prepare.log");
    fs::write(&functions, "")?;
    fs::write(&script, "echo prepare-failed >&2\nexit 7")?;

    let status = run_composed_bash(&functions, &script, &log)?;

    assert_eq!(status.code(), Some(7));
    assert!(fs::read_to_string(log)?.contains("prepare-failed"));
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn early_successful_exit_ignores_broken_pipe() -> Result<()> {
    let root = temp_dir("bonesremote-prepare-early-exit")?;
    let functions = root.join("functions.sh");
    let script = root.join("01_prepare.sh");
    let log = root.join("prepare.log");
    fs::write(&functions, "exit 0\n")?;
    fs::write(&script, "# padding\n".repeat(1_000_000))?;

    let status = run_composed_bash(&functions, &script, &log)?;

    assert!(status.success());
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn framework_prepare_templates_do_not_source_control_plane_files() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../bonesdeploy/assets/frameworks");
    for framework in ["laravel", "rails", "django"] {
        let template = fs::read_to_string(
            root.join(framework).join("deployment/prepare").join(format!("01_prepare_{framework}.sh")),
        )?;
        assert!(!template.contains("DEPLOYMENT_DIR"), "{framework} prepare template still references DEPLOYMENT_DIR");
        assert!(!template.contains("functions.sh"), "{framework} prepare template still sources functions.sh");
    }
    Ok(())
}

#[test]
fn django_prepare_template_uses_the_configured_python_minor() -> Result<()> {
    let template = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../bonesdeploy/assets/frameworks/django/deployment/prepare/01_prepare_django.sh"),
    )?;

    assert!(template.contains("BONES_RUNTIME_PYTHON_VERSION"));
    assert!(template.contains("\"$PYTHON_BIN\" -m venv"));
    Ok(())
}

#[test]
fn rails_prepare_template_uses_the_managed_bundler_binary() -> Result<()> {
    let template = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../bonesdeploy/assets/frameworks/rails/deployment/prepare/01_prepare_rails.sh"),
    )?;

    assert!(template.contains("local bundle_binary=\"${ruby_binary%/*}/bundle\""));
    assert!(!template.contains("-S bundle"));
    Ok(())
}
