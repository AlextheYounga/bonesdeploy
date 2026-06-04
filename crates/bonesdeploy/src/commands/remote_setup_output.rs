use std::env;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::result::Result as StdResult;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};

use crate::commands::remote_setup;
use crate::config;

const BRAND: &str = "☠ bonesdeploy";
const SPINNER_FRAMES: [char; 4] = ['|', '/', '-', '\\'];
const SPINNER_INTERVAL: Duration = Duration::from_millis(120);

#[derive(Debug, PartialEq, Eq)]
enum OutputLine {
    Task(String),
    Error(String),
}

pub(crate) fn run(cfg: &config::BonesConfig, ssh_user: &str, extra_args: &[String]) -> Result<()> {
    remote_setup::ensure_remote_python3_available(cfg, ssh_user)?;

    let interactive = io::stdout().is_terminal();
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{BRAND}")?;
    stdout.flush()?;

    let child = build_ansible_command(cfg, ssh_user, extra_args)?.spawn().context("Failed to run ansible-playbook")?;

    let status = stream_ansible_output(&mut stdout, interactive, child)?;
    if !status.success() {
        bail!("ansible-playbook failed with status {status}");
    }

    Ok(())
}

fn build_ansible_command(cfg: &config::BonesConfig, ssh_user: &str, extra_args: &[String]) -> Result<Command> {
    let playbook = Path::new(config::Constants::BONES_REMOTE_SETUP_PLAYBOOK);
    if !playbook.is_file() {
        bail!("Missing remote setup playbook: {}", playbook.display());
    }

    let roles_dir = Path::new(config::Constants::BONES_REMOTE_ROLES_DIR);
    if !roles_dir.is_dir() {
        bail!("Missing remote roles directory: {}", roles_dir.display());
    }

    let project_root_parent = remote_setup::resolve_project_root_parent(&cfg.data.project_root);
    let inventory = format!("{},", cfg.data.host);
    let roles_path = env_ansible_roles_path(roles_dir);

    let ansible_playbook_binary = remote_setup::resolve_ansible_playbook_binary()?;
    let mut command = Command::new(&ansible_playbook_binary);
    command.env("ANSIBLE_ROLES_PATH", roles_path);
    command
        .arg("-i")
        .arg(&inventory)
        .arg("-u")
        .arg(ssh_user)
        .arg("-e")
        .arg(format!("ansible_port={}", cfg.data.port))
        .arg("-e")
        .arg(format!("deploy_user={}", cfg.permissions.defaults.deploy_user))
        .arg("-e")
        .arg(format!("service_user={}", cfg.permissions.defaults.service_user))
        .arg("-e")
        .arg(format!("group={}", cfg.permissions.defaults.group))
        .arg("-e")
        .arg(format!("project_root_parent={project_root_parent}"))
        .arg("-e")
        .arg(format!("project_root={}", cfg.data.project_root))
        .arg("-e")
        .arg(format!("web_root={}", cfg.data.web_root))
        .arg("-e")
        .arg(format!("project_name={}", cfg.data.project_name))
        .arg("-e")
        .arg(format!("repo_path={}", cfg.data.repo_path));

    if cfg.ssl.enabled && !cfg.ssl.domain.is_empty() {
        command
            .arg("-e")
            .arg(format!("nginx_server_name={}", cfg.ssl.domain))
            .arg("-e")
            .arg("nginx_ssl_enabled=true")
            .arg("-e")
            .arg(format!("nginx_ssl_certificate_path=/etc/letsencrypt/live/{}/fullchain.pem", cfg.ssl.domain))
            .arg("-e")
            .arg(format!("nginx_ssl_certificate_key_path=/etc/letsencrypt/live/{}/privkey.pem", cfg.ssl.domain));
    }

    command.args(extra_args);
    command.arg(playbook);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    Ok(command)
}

fn stream_ansible_output<W: Write>(writer: &mut W, interactive: bool, mut child: Child) -> Result<ExitStatus> {
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("stdout was not piped"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("stderr was not piped"))?;
    let (tx, rx) = mpsc::channel();
    spawn_stream_reader(stdout, tx.clone());
    spawn_stream_reader(stderr, tx);
    let mut progress = Progress::new(interactive);

    loop {
        match rx.recv_timeout(SPINNER_INTERVAL) {
            Ok(line) => {
                if let Some(output) = classify_output_line(&line) {
                    progress.clear(writer)?;
                    write_output_line(writer, output)?;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => progress.tick(writer)?,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    progress.clear(writer)?;
    child.wait().context("Failed to wait for ansible-playbook")
}

struct Progress {
    interactive: bool,
    spinner_index: usize,
    rendered: bool,
}

impl Progress {
    fn new(interactive: bool) -> Self {
        Self { interactive, spinner_index: 0, rendered: false }
    }

    fn tick<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        if !self.interactive {
            return Ok(());
        }

        let frame = SPINNER_FRAMES[self.spinner_index];
        self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();
        write!(writer, "{}", format_progress_line(frame))?;
        writer.flush()?;
        self.rendered = true;
        Ok(())
    }

    fn clear<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        if self.rendered {
            write!(writer, "\r\u{1b}[2K")?;
            writer.flush()?;
            self.rendered = false;
        }

        Ok(())
    }
}

fn format_progress_line(spinner: char) -> String {
    format!("\r\u{1b}[2K[{spinner}] running remote setup")
}

fn write_output_line<W: Write>(writer: &mut W, line: OutputLine) -> Result<()> {
    match line {
        OutputLine::Task(task) => writeln!(writer, "{task}")?,
        OutputLine::Error(error) => writeln!(writer, "error: {error}")?,
    }

    writer.flush()?;
    Ok(())
}

fn env_ansible_roles_path(roles_dir: &Path) -> String {
    env::var("ANSIBLE_ROLES_PATH")
        .ok()
        .filter(|value| !value.is_empty())
        .map_or_else(|| roles_dir.display().to_string(), |existing| format!("{}:{existing}", roles_dir.display()))
}

fn classify_output_line(line: &str) -> Option<OutputLine> {
    if let Some(task) = clean_task_line(line) {
        return Some(OutputLine::Task(task));
    }

    clean_error_line(line).map(OutputLine::Error)
}

pub(crate) fn clean_task_line(line: &str) -> Option<String> {
    let line = line.trim().strip_prefix("TASK [")?;
    let (task, _) = line.split_once(']')?;
    let task = task.rsplit_once(" : ").map_or(task, |(_, description)| description).trim();

    (!task.is_empty()).then(|| task.to_string())
}

pub(crate) fn clean_error_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.contains("fatal:") || trimmed.contains("FAILED!") || trimmed.contains("UNREACHABLE!") {
        return Some(trimmed.to_string());
    }

    None
}

fn spawn_stream_reader<T>(reader: T, sender: Sender<String>)
where
    T: io::Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(StdResult::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{OutputLine, classify_output_line, clean_error_line, clean_task_line, format_progress_line};

    #[test]
    fn clean_task_line_removes_ansible_task_wrapper() {
        let cleaned = clean_task_line("TASK [users : Create deploy user]");

        assert_eq!(cleaned.as_deref(), Some("Create deploy user"));
    }

    #[test]
    fn clean_task_line_accepts_ansible_decorated_task_headers() {
        let cleaned =
            clean_task_line("TASK [common : Install packages] ************************************************");

        assert_eq!(cleaned.as_deref(), Some("Install packages"));
    }

    #[test]
    fn clean_task_line_keeps_plain_task_name_without_group_prefix() {
        let cleaned = clean_task_line("TASK [Create deploy user]");

        assert_eq!(cleaned.as_deref(), Some("Create deploy user"));
    }

    #[test]
    fn clean_task_line_ignores_non_task_lines() {
        assert_eq!(clean_task_line("ok: [host]"), None);
        assert_eq!(clean_task_line("PLAY [all]"), None);
    }

    #[test]
    fn clean_error_line_detects_ansible_failures() {
        let cleaned = clean_error_line("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}");

        assert_eq!(cleaned.as_deref(), Some("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}"));
        assert_eq!(clean_error_line("ok: [host]"), None);
    }

    #[test]
    fn classify_output_line_prefers_tasks_and_failures() {
        let task = classify_output_line("TASK [users : Create deploy user]");
        let error = classify_output_line("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}");

        assert_eq!(task, Some(OutputLine::Task(String::from("Create deploy user"))));
        assert_eq!(
            error,
            Some(OutputLine::Error(String::from("fatal: [203.0.113.10]: FAILED! => {\"msg\":\"boom\"}")))
        );
    }

    #[test]
    fn classify_output_line_ignores_warnings_and_noise() {
        assert_eq!(classify_output_line("[WARNING]: discovered interpreter"), None);
        assert_eq!(classify_output_line("ansible-playbook [core 2.20.5]"), None);
    }

    #[test]
    fn format_progress_line_renders_single_live_status() {
        assert_eq!(format_progress_line('|'), "\r\u{1b}[2K[|] running remote setup");
    }
}
