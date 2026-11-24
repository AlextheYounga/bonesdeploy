use std::collections::BTreeMap;

use anyhow::{Result, bail};
use bonesinfra::manifest::{ArtifactKind, ArtifactState, ComposeReport, ManifestReport, ResolvedArtifact};
use console::style;

pub(crate) struct PathArtifact {
    pub(crate) path: String,
    pub(crate) kind: String,
    pub(crate) state: Option<String>,
}

impl From<ResolvedArtifact> for PathArtifact {
    fn from(artifact: ResolvedArtifact) -> Self {
        Self {
            path: artifact.path,
            kind: artifact_kind(artifact.kind).to_owned(),
            state: Some(artifact_state(artifact.state).to_owned()),
        }
    }
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    artifact: Option<PathArtifact>,
}

fn artifact_kind(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::File => "file",
        ArtifactKind::Directory => "directory",
        ArtifactKind::Link => "link",
        ArtifactKind::Socket => "socket",
    }
}

fn artifact_state(state: ArtifactState) -> &'static str {
    match state {
        ArtifactState::Present => "present",
        ArtifactState::Missing => "missing",
        ArtifactState::WrongKind => "wrong-kind",
    }
}

/// Renders an inspected manifest as a human-readable status tree.
pub fn render_report(report: ManifestReport) -> Result<String> {
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

pub(crate) fn render_path_tree(artifacts: Vec<PathArtifact>) -> Result<Vec<String>> {
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
