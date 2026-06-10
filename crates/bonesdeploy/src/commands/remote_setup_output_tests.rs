use anyhow::Result;
use serde_json::Value;

use super::{OutputLine, classify_output_line, clean_error_line, clean_task_line, format_progress_message};
use crate::config;

/// Removes the Ansible task wrapper prefix and role group from a task line.
#[test]
fn clean_task_line_removes_ansible_task_wrapper() {
    let cleaned = clean_task_line("TASK [users : Create deploy user]");

    assert_eq!(cleaned.as_deref(), Some("Create deploy user"));
}

/// Accepts an Ansible-decorated task header with trailing asterisks.
#[test]
fn clean_task_line_accepts_ansible_decorated_task_headers() {
    let cleaned = clean_task_line("TASK [common : Install packages] ************************************************");

    assert_eq!(cleaned.as_deref(), Some("Install packages"));
}

/// Keeps the plain task name when there is no role group prefix.
#[test]
fn clean_task_line_keeps_plain_task_name_without_group_prefix() {
    let cleaned = clean_task_line("TASK [Create deploy user]");

    assert_eq!(cleaned.as_deref(), Some("Create deploy user"));
}

/// Returns None for non-task lines like ok and PLAY headers.
#[test]
fn clean_task_line_ignores_non_task_lines() {
    assert_eq!(clean_task_line("ok: [host]"), None);
    assert_eq!(clean_task_line("PLAY [all]"), None);
}

/// Detects Ansible fatal failure lines and extracts the error message.
#[test]
fn clean_error_line_detects_ansible_failures() {
    let cleaned = clean_error_line("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}");

    assert_eq!(cleaned.as_deref(), Some("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}"));
    assert_eq!(clean_error_line("ok: [host]"), None);
}

/// Classifies task and failure lines over other output types.
#[test]
fn classify_output_line_prefers_tasks_and_failures() {
    let task = classify_output_line("TASK [users : Create deploy user]");
    let error = classify_output_line("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}");

    assert_eq!(task, Some(OutputLine::Task(String::from("Create deploy user"))));
    assert_eq!(error, Some(OutputLine::Error(String::from("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}"))));
}

/// Ignores warning and noise lines from Ansible output.
#[test]
fn classify_output_line_ignores_warnings_and_noise() {
    assert_eq!(classify_output_line("[WARNING]: discovered interpreter"), None);
    assert_eq!(classify_output_line("ansible-playbook [core 2.20.5]"), None);
}

/// Styles the current task name with ANSI formatting for the progress display.
#[test]
fn format_progress_message_styles_current_task() {
    assert_eq!(
        format_progress_message("Ensure deploy user exists"),
        "\u{1b}[2mSetting up remote:\u{1b}[0m \u{1b}[32m\u{1b}[1mEnsure deploy user exists\u{1b}[0m\u{1b}[K"
    );
}

/// Includes the merged `setup_apt_packages` list in generated Ansible vars for remote setup.
#[test]
fn build_ansible_vars_includes_merged_setup_apt_packages() -> Result<()> {
    let cfg = config::BonesConfig {
        data: config::Data {
            remote_name: String::from("production"),
            project_name: String::from("acme"),
            host: String::from("example.com"),
            port: String::from("22"),
            repo_path: String::from("/home/git/acme.git"),
            project_root: String::from("/srv/deployments/acme"),
            web_root: String::from("public"),
            branch: String::from("main"),
            deploy_on_push: true,
        },
        permissions: config::Permissions {
            defaults: config::PermissionDefaults {
                deploy_user: String::from("git"),
                service_user: String::from("acme"),
                group: String::from("www-data"),
                dir_mode: String::from("750"),
                file_mode: String::from("640"),
            },
            paths: Vec::new(),
        },
        releases: config::Releases {
            keep: 5,
            shared_files: vec![String::from(".env")],
            shared_dirs: vec![String::from("storage")],
        },
        ssl: config::Ssl::default(),
    };

    let vars = super::build_ansible_vars(
        &cfg,
        serde_json::json!({
            "setup_apt_packages": ["curl", "git", "nginx"]
        }),
    )?;

    assert_eq!(
        vars.get("setup_apt_packages"),
        Some(&Value::Array(vec![
            Value::String(String::from("curl")),
            Value::String(String::from("git")),
            Value::String(String::from("nginx"))
        ]))
    );
    Ok(())
}
