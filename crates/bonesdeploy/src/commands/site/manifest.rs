use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use console::style;
use serde::Deserialize;

use crate::{config, infra};

#[derive(Deserialize)]
struct ManifestReport {
    strategy: Strategy,
    entries: Vec<Artifact>,
    managed_services: Vec<ManagedService>,
    compose: Option<ComposeReport>,
}

#[derive(Deserialize)]
struct Strategy {
    framework: String,
    mode: String,
    backend: String,
    ssl: bool,
}

#[derive(Deserialize)]
struct Artifact {
    path: String,
    kind: String,
    state: String,
}

pub(super) struct PathArtifact {
    pub(super) path: String,
    pub(super) kind: String,
    pub(super) state: Option<String>,
}

impl From<Artifact> for PathArtifact {
    fn from(artifact: Artifact) -> Self {
        Self { path: artifact.path, kind: artifact.kind, state: Some(artifact.state) }
    }
}

#[derive(Deserialize)]
struct ManagedService {
    unit: String,
    running: bool,
    enabled: bool,
}

#[derive(Deserialize)]
struct ComposeReport {
    project_name: String,
    files: Vec<String>,
    security_mode: String,
    security_warning: String,
    persistent_data: String,
    services: Vec<ComposeService>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ComposeService {
    service: String,
    condition: String,
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    artifact: Option<PathArtifact>,
}

pub fn run(format: &str) -> Result<()> {
    super::readiness::ensure_project_ready()?;

    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    let json = bonesinfra::run_with_request_output(&["manifest", "show", "--request-stdin"], &request)?;
    if format == "json" {
        print!("{json}");
        return Ok(());
    }

    let report = serde_json::from_str(&json).context("BonesInfra returned an invalid manifest report")?;
    print!("{}", render_text(report)?);
    Ok(())
}

fn render_text(report: ManifestReport) -> Result<String> {
    let mut lines = vec![
        summary("Framework", &format!("{} ({})", report.strategy.framework, report.strategy.mode)),
        summary("Runtime", &report.strategy.backend),
        summary(
            super::SSL_LABEL,
            &if report.strategy.ssl {
                style("enabled").green().to_string()
            } else {
                style("disabled").yellow().to_string()
            },
        ),
        String::new(),
        style("Manifest").cyan().bold().to_string(),
    ];
    lines.extend(render_path_tree(report.entries.into_iter().map(PathArtifact::from).collect())?);

    lines.push(String::new());
    lines.push(style("Managed services").cyan().bold().to_string());
    for service in report.managed_services {
        let marker =
            if service.running && service.enabled { style("✓").green().bold() } else { style("✗").red().bold() };
        lines.push(format!("{marker} {}", service.unit));
    }

    if let Some(compose) = report.compose {
        append_compose(compose, &mut lines);
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn summary(label: &str, value: &str) -> String {
    format!("{} {value}", style(format!("{label:<9}")).dim())
}

pub(super) fn render_path_tree(artifacts: Vec<PathArtifact>) -> Result<Vec<String>> {
    let mut lines = vec![style(char::from(b'/')).bold().to_string()];
    let mut root = TreeNode::default();
    for artifact in artifacts {
        insert_artifact(&mut root, artifact)?;
    }
    append_children(&root.children, "", &mut lines);
    Ok(lines)
}

fn insert_artifact(root: &mut TreeNode, artifact: PathArtifact) -> Result<()> {
    if !artifact.path.starts_with('/') {
        bail!("Manifest artifact path is not absolute: {}", artifact.path);
    }
    let mut node = root;
    for part in artifact.path.split('/').filter(|part| !part.is_empty()) {
        node = node.children.entry(part.to_owned()).or_default();
    }
    node.artifact = Some(artifact);
    Ok(())
}

fn append_children(children: &BTreeMap<String, TreeNode>, prefix: &str, lines: &mut Vec<String>) {
    for (index, (name, node)) in children.iter().enumerate() {
        let is_last = index + 1 == children.len();
        let connector = if is_last { "└── " } else { "├── " };
        let label = if node.artifact.as_ref().is_none_or(|artifact| artifact.kind == "directory") {
            format!("{name}/")
        } else {
            name.clone()
        };
        let marker = node.artifact.as_ref().and_then(|artifact| match artifact.state.as_deref() {
            Some("present") => Some(style("✓").green().bold().to_string()),
            Some("missing") => Some(style("!").yellow().bold().to_string()),
            Some(_) => Some(style("✗").red().bold().to_string()),
            None => None,
        });
        let item = marker.map_or(label.clone(), |marker| format!("{marker} {label}"));
        lines.push(format!("{}{}{item}", style(prefix).dim(), style(connector).dim()));
        append_children(&node.children, &format!("{prefix}{}", if is_last { "    " } else { "│   " }), lines);
    }
}

fn append_compose(compose: ComposeReport, lines: &mut Vec<String>) {
    lines.extend([
        String::new(),
        style("Compose").cyan().bold().to_string(),
        summary("Project", &compose.project_name),
        summary("Files", &if compose.files.is_empty() { "not available".into() } else { compose.files.join(", ") }),
        summary("Security", &compose.security_mode),
        format!("{} {}", style("Warning").yellow().bold(), compose.security_warning),
        format!("{} {}", style("Persistent data").dim(), compose.persistent_data),
    ]);
    for service in compose.services {
        lines.push(format!("{} {}  {}", style("•").cyan(), service.service, style(service.condition).dim()));
    }
    if let Some(error) = compose.error {
        lines.push(format!("{} {error}", style("✗").red().bold()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::output::strip_ansi;

    #[test]
    fn text_manifest_renders_a_status_only_path_tree() -> Result<()> {
        let report = serde_json::from_str(
            r#"{
            "strategy":{"framework":"none","mode":"custom","backend":"native","ssl":false},
            "entries":[
                {"path":"/srv/sites/example/current","kind":"link","state":"missing"},
                {"path":"/srv/sites/example","kind":"directory","state":"present"},
                {"path":"/etc/nginx/example.conf","kind":"file","state":"wrong-kind"}
            ],
            "managed_services":[{"unit":"example.service","running":true,"enabled":true}],
            "compose":null
        }"#,
        )?;

        let output = strip_ansi(&render_text(report)?);
        assert!(output.contains("├── etc/\n│   └── nginx/\n│       └── ✗ example.conf"));
        assert!(output.contains("└── srv/\n    └── sites/\n        └── ✓ example/\n            └── ! current"));
        assert!(output.contains("Managed services\n✓ example.service"));
        assert!(!output.contains("directory"));
        assert!(!output.contains("setup"));
        Ok(())
    }
}
