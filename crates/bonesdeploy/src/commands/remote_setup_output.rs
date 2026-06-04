use std::env;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::result::Result as StdResult;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};

use crate::commands::remote_setup;
use crate::config;

const BRAND: &str = "☠ bonesdeploy";
const SPINNER_FRAMES: [char; 4] = ['|', '/', '-', '\\'];
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(120);

#[cfg(test)]
mod tests {
    use super::{clean_error_line, clean_task_line, format_status_line};

    #[test]
    fn clean_task_line_removes_ansible_task_wrapper() {
        let cleaned = clean_task_line("TASK [users : Create deploy user]");

        assert_eq!(cleaned.as_deref(), Some("users : Create deploy user"));
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
    fn format_status_line_clears_previous_content() {
        let rendered = format_status_line('|', "users : Create deploy user", false);

        assert_eq!(rendered, "\r\u{1b}[2K[|] users : Create deploy user");
    }
}

pub(crate) fn run(cfg: &config::BonesConfig, ssh_user: &str, extra_args: &[String]) -> Result<()> {
    remote_setup::ensure_remote_python3_available(cfg, ssh_user)?;

    let interactive = io::stdout().is_terminal();
    let stdout = io::stdout();
    let mut renderer = LiveStatusRenderer::new(stdout, interactive);
    renderer.print_brand()?;
    renderer.set_task(format!("running remote setup on {}", cfg.data.host))?;

    let mut child =
        build_ansible_command(cfg, ssh_user, extra_args)?.spawn().context("Failed to run ansible-playbook")?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("stdout was not piped"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("stderr was not piped"))?;

    renderer.set_task(format!("running ansible playbook on {}", cfg.data.host))?;
    let status = stream_ansible_output(child, stdout, stderr, &mut renderer)?;
    renderer.finish()?;

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

fn stream_ansible_output(
    mut child: Child,
    stdout: ChildStdout,
    stderr: ChildStderr,
    renderer: &mut LiveStatusRenderer<io::Stdout>,
) -> Result<ExitStatus> {
    let (tx, rx) = mpsc::channel();
    spawn_stream_reader(stdout, StreamKind::Stdout, tx.clone());
    spawn_stream_reader(stderr, StreamKind::Stderr, tx);

    let mut last_error: Option<String> = None;

    loop {
        match rx.recv_timeout(EVENT_POLL_INTERVAL) {
            Ok(StreamEvent::Line { kind, line }) => {
                if let Some(task) = clean_task_line(&line) {
                    renderer.set_task(task)?;
                } else if let Some(error) = clean_error_line(&line) {
                    last_error = Some(error.clone());
                    renderer.set_error(error)?;
                } else if matches!(kind, StreamKind::Stderr) {
                    renderer.set_error(line.trim().to_string())?;
                }
            }
            Err(RecvTimeoutError::Timeout) => renderer.tick()?,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    let status = child.wait().context("Failed to wait for ansible-playbook")?;
    if !status.success()
        && let Some(error) = last_error
    {
        renderer.set_error(error)?;
    }

    Ok(status)
}

fn env_ansible_roles_path(roles_dir: &Path) -> String {
    env::var("ANSIBLE_ROLES_PATH")
        .ok()
        .filter(|value| !value.is_empty())
        .map_or_else(|| roles_dir.display().to_string(), |existing| format!("{}:{existing}", roles_dir.display()))
}

pub(crate) fn clean_task_line(line: &str) -> Option<String> {
    let task = line.trim().strip_prefix("TASK [")?.strip_suffix(']')?.trim();
    (!task.is_empty()).then(|| task.to_string())
}

pub(crate) fn clean_error_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.contains("fatal:") || trimmed.contains("FAILED!") || trimmed.contains("UNREACHABLE!") {
        Some(trimmed.to_string())
    } else {
        None
    }
}

pub(crate) fn format_status_line(spinner: char, text: &str, failed: bool) -> String {
    let indicator = if failed { 'x' } else { spinner };
    format!("\r\u{1b}[2K[{indicator}] {text}")
}

struct LiveStatusRenderer<W: Write> {
    writer: W,
    interactive: bool,
    spinner_index: usize,
    current: String,
    failed: bool,
    rendered: bool,
}

impl<W: Write> LiveStatusRenderer<W> {
    fn new(writer: W, interactive: bool) -> Self {
        Self { writer, interactive, spinner_index: 0, current: String::new(), failed: false, rendered: false }
    }

    fn print_brand(&mut self) -> Result<()> {
        writeln!(self.writer, "{BRAND}")?;
        self.rendered = true;
        self.writer.flush()?;
        Ok(())
    }

    fn set_task(&mut self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        if self.current == text && !self.failed && self.rendered {
            return Ok(());
        }

        self.current = text;
        self.failed = false;
        self.render()
    }

    fn set_error(&mut self, text: impl Into<String>) -> Result<()> {
        let text = text.into();
        if self.current == text && self.failed && self.rendered {
            return Ok(());
        }

        self.current = text;
        self.failed = true;
        self.render()
    }

    fn tick(&mut self) -> Result<()> {
        if !self.interactive || self.current.is_empty() {
            return Ok(());
        }

        self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();
        self.render_live()
    }

    fn finish(&mut self) -> Result<()> {
        if self.interactive && self.rendered {
            writeln!(self.writer)?;
            self.writer.flush()?;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        if self.interactive { self.render_live() } else { self.render_plain() }
    }

    fn render_live(&mut self) -> Result<()> {
        self.rendered = true;
        let frame = SPINNER_FRAMES[self.spinner_index];
        write!(self.writer, "{}", format_status_line(frame, &self.current, self.failed))?;
        self.writer.flush()?;
        Ok(())
    }

    fn render_plain(&mut self) -> Result<()> {
        if !self.rendered {
            self.rendered = true;
            writeln!(self.writer, "{BRAND}")?;
        }

        if self.failed {
            writeln!(self.writer, "error: {}", self.current)?;
        } else {
            writeln!(self.writer, "task: {}", self.current)?;
        }

        self.writer.flush()?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}

enum StreamEvent {
    Line { kind: StreamKind, line: String },
}

fn spawn_stream_reader<T>(reader: T, kind: StreamKind, sender: Sender<StreamEvent>)
where
    T: io::Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(StdResult::ok) {
            if sender.send(StreamEvent::Line { kind, line }).is_err() {
                break;
            }
        }
    });
}
