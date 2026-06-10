use std::env;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::result::Result as StdResult;
use std::sync::mpsc::{self, Sender};
use std::thread;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Map, Value};
use spinners::{Spinner, Spinners, Stream};
use tempfile::NamedTempFile;

use crate::commands::remote_setup;
use crate::config;
use shared::paths::{DeploymentPaths, ssl_certificate_key_path, ssl_certificate_path};

const BRAND: &str = "☠ bonesdeploy";
const CLEAR_LINE: &str = "\r\u{1b}[2K";
const RESET: &str = "\u{1b}[0m";
const BOLD: &str = "\u{1b}[1m";
const DIM: &str = "\u{1b}[2m";
const GREEN: &str = "\u{1b}[32m";
const RED: &str = "\u{1b}[31m";

#[derive(Debug, PartialEq, Eq)]
enum OutputLine {
    Task(String),
    Error(String),
}

pub(crate) fn run(cfg: &config::BonesConfig, ssh_user: &str, extra_vars: Value, extra_args: &[String]) -> Result<()> {
    remote_setup::ensure_remote_python3_available(cfg, ssh_user)?;

    let interactive = io::stdout().is_terminal();
    let mut stdout = io::stdout();
    writeln!(stdout, "{BRAND}")?;
    stdout.flush()?;

    let vars_file = write_vars_file(&build_ansible_vars(cfg, extra_vars)?)?;
    let child = build_ansible_command(cfg, ssh_user, vars_file.path(), extra_args)?
        .spawn()
        .context("Failed to run ansible-playbook")?;

    let status = stream_ansible_output(&mut stdout, interactive, child)?;
    if !status.success() {
        bail!("ansible-playbook failed with status {status}");
    }

    Ok(())
}

fn build_ansible_command(
    cfg: &config::BonesConfig,
    ssh_user: &str,
    vars_file: &Path,
    extra_args: &[String],
) -> Result<Command> {
    validate_playbook_and_roles_directories()?;

    let ansible_playbook_binary = remote_setup::resolve_ansible_playbook_binary()?;
    let mut command = Command::new(&ansible_playbook_binary);

    add_base_ansible_args(&mut command, cfg, ssh_user);
    command.arg("-e").arg(format!("@{}", vars_file.display()));
    add_final_args(&mut command, extra_args);

    Ok(command)
}

fn validate_playbook_and_roles_directories() -> Result<()> {
    let playbook = Path::new(config::Constants::BONES_REMOTE_SETUP_PLAYBOOK);
    if !playbook.is_file() {
        bail!("Missing remote setup playbook: {}", playbook.display());
    }

    let roles_dir = Path::new(config::Constants::BONES_REMOTE_ROLES_DIR);
    if !roles_dir.is_dir() {
        bail!("Missing remote roles directory: {}", roles_dir.display());
    }

    Ok(())
}

fn add_base_ansible_args(command: &mut Command, cfg: &config::BonesConfig, ssh_user: &str) {
    let roles_dir = Path::new(config::Constants::BONES_REMOTE_ROLES_DIR);
    let host = config::resolve_host(cfg).unwrap_or_else(|_| String::new());
    let inventory = format!("{host},");
    let roles_path = env_ansible_roles_path(roles_dir);

    command.env("ANSIBLE_ROLES_PATH", roles_path);
    command.arg("-i").arg(&inventory).arg("-u").arg(ssh_user);
}

fn add_final_args(command: &mut Command, extra_args: &[String]) {
    let playbook = Path::new(config::Constants::BONES_REMOTE_SETUP_PLAYBOOK);
    command.args(extra_args);
    command.arg(playbook);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
}

fn stream_ansible_output<W: Write>(writer: &mut W, interactive: bool, mut child: Child) -> Result<ExitStatus> {
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("stdout was not piped"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("stderr was not piped"))?;
    let (tx, rx) = mpsc::channel();
    spawn_stream_reader(stdout, tx.clone());
    spawn_stream_reader(stderr, tx);
    let mut progress = Progress::new(interactive);

    for line in rx {
        if let Some(output) = classify_output_line(&line) {
            write_output_line(writer, &mut progress, output)?;
        }
    }

    progress.finish(writer)?;
    child.wait().context("Failed to wait for ansible-playbook")
}

struct Progress {
    interactive: bool,
    rendered: bool,
    spinner: Option<Spinner>,
    task: String,
    failed: bool,
}

impl Progress {
    fn new(interactive: bool) -> Self {
        Self { interactive, rendered: false, spinner: None, task: String::from("running remote setup"), failed: false }
    }

    fn set_task<W: Write>(&mut self, writer: &mut W, task: String) -> Result<()> {
        if self.failed {
            return Ok(());
        }

        self.task = task;
        if self.interactive {
            self.restart_spinner();
            self.rendered = true;
            return Ok(());
        }

        writeln!(writer, "{}", self.task)?;
        writer.flush()?;
        Ok(())
    }

    fn set_error<W: Write>(&mut self, writer: &mut W, error: String) -> Result<()> {
        if !self.interactive {
            writeln!(writer, "error: {error}")?;
            writer.flush()?;
            return Ok(());
        }

        self.stop_spinner();
        self.task = error;
        self.failed = true;
        write!(writer, "{}", format_error_line(&self.task))?;
        writer.flush()?;
        self.rendered = true;
        Ok(())
    }

    fn finish<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        if let Some(mut spinner) = self.spinner.take() {
            spinner.stop_with_newline();
            self.rendered = false;
            return Ok(());
        }

        if self.rendered {
            writeln!(writer)?;
            writer.flush()?;
            self.rendered = false;
        }

        Ok(())
    }

    fn restart_spinner(&mut self) {
        self.stop_spinner();
        self.spinner =
            Some(Spinner::with_stream(Spinners::Dots8Bit, format_progress_message(&self.task), Stream::Stdout));
    }

    fn stop_spinner(&mut self) {
        if let Some(mut spinner) = self.spinner.take() {
            spinner.stop();
        }
    }
}

fn format_progress_message(task: &str) -> String {
    format!("{DIM}Setting up remote:{RESET} {GREEN}{BOLD}{task}{RESET}\u{1b}[K")
}

fn format_error_line(error: &str) -> String {
    format!("{CLEAR_LINE}{RED}{BOLD}error{RESET} {RED}{error}{RESET}")
}

fn write_output_line<W: Write>(writer: &mut W, progress: &mut Progress, line: OutputLine) -> Result<()> {
    match line {
        OutputLine::Task(task) if progress.interactive => progress.set_task(writer, task)?,
        OutputLine::Task(task) => writeln!(writer, "{task}")?,
        OutputLine::Error(error) => progress.set_error(writer, error)?,
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

fn build_ansible_vars(cfg: &config::BonesConfig, extra_vars: Value) -> Result<Value> {
    let paths =
        DeploymentPaths::new(&cfg.data.project_name, &cfg.data.repo_path, &cfg.data.project_root, &cfg.data.web_root);
    let mut vars = Map::new();

    vars.insert(String::from("ansible_port"), Value::String(cfg.data.port.clone()));
    vars.insert(String::from("deploy_user"), Value::String(cfg.permissions.defaults.deploy_user.clone()));
    vars.insert(String::from("service_user"), Value::String(cfg.permissions.defaults.service_user.clone()));
    vars.insert(String::from("group"), Value::String(cfg.permissions.defaults.group.clone()));
    vars.insert(String::from("project_root_parent"), Value::String(paths.project_root_parent.clone()));
    vars.insert(String::from("project_root"), Value::String(cfg.data.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(cfg.data.web_root.clone()));
    vars.insert(String::from("project_name"), Value::String(cfg.data.project_name.clone()));
    vars.insert(String::from("repo_path"), Value::String(cfg.data.repo_path.clone()));
    vars.insert(String::from("paths"), serde_json::to_value(paths)?);

    if cfg.ssl.enabled && !cfg.ssl.domain.is_empty() {
        vars.insert(String::from("nginx_server_name"), Value::String(cfg.ssl.domain.clone()));
        vars.insert(String::from("nginx_ssl_enabled"), Value::Bool(true));
        vars.insert(String::from("nginx_ssl_certificate_path"), Value::String(ssl_certificate_path(&cfg.ssl.domain)));
        vars.insert(
            String::from("nginx_ssl_certificate_key_path"),
            Value::String(ssl_certificate_key_path(&cfg.ssl.domain)),
        );
    }

    merge_extra_vars(&mut vars, extra_vars)?;
    Ok(Value::Object(vars))
}

fn merge_extra_vars(vars: &mut Map<String, Value>, extra_vars: Value) -> Result<()> {
    match extra_vars {
        Value::Object(extra) => {
            for (key, value) in extra {
                vars.insert(key, value);
            }
            Ok(())
        }
        Value::Null => Ok(()),
        other => bail!("extra vars must be a JSON object, got {other}"),
    }
}

fn write_vars_file(vars: &Value) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new().context("Failed to create temporary Ansible vars file")?;
    serde_json::to_writer_pretty(file.as_file_mut(), vars).context("Failed to write Ansible vars file")?;
    file.as_file_mut().write_all(b"\n").context("Failed to finish Ansible vars file")?;
    Ok(file)
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
#[path = "remote_setup_output_tests.rs"]
mod tests;
