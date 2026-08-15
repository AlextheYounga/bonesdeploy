use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::{Result, bail};
<<<<<<< HEAD
use easy_tree::Tree;
use serde::Deserialize;
=======
use bonesdeploy_core::config::PROJECT_SETUP_ERROR;
>>>>>>> feature/decentralization

use crate::infra::git;
use bonesdeploy_core::paths;

pub fn run(format: &str) -> Result<()> {
    git::ensure_git_repository()?;

    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() || !Path::new(paths::LOCAL_INFRA_DIR).is_dir() {
        bail!(PROJECT_SETUP_ERROR);
    }

<<<<<<< HEAD
    if format == "json" {
        return bonesinfra::run(&["manifest", "show", "--config", paths::LOCAL_BONES_TOML, "--format", format]);
    }

    let output =
        bonesinfra::run_capture(&["manifest", "show", "--config", paths::LOCAL_BONES_TOML, "--format", "json"])?;
    let report: ManifestReport = serde_json::from_str(&output)?;
    print!("{}", render_text(&report));
    Ok(())
}

#[derive(Deserialize)]
struct ManifestReport {
    strategy: Strategy,
    entries: Vec<Artifact>,
    managed_services: Vec<ManagedService>,
}

#[derive(Deserialize)]
struct Strategy {
    backend: String,
    framework: String,
    mode: String,
    services: Vec<String>,
    ssl: bool,
}

#[derive(Deserialize)]
struct Artifact {
    path: String,
    kind: String,
    owner: String,
    state: String,
    actual_kind: Option<String>,
}

#[derive(Deserialize)]
struct ManagedService {
    unit: String,
    owner: String,
    running: bool,
    enabled: bool,
}

struct TreeEntry {
    name: String,
    artifact: Option<usize>,
}

fn render_text(report: &ManifestReport) -> String {
    let strategy = &report.strategy;
    let services = if strategy.services.is_empty() { String::from("none") } else { strategy.services.join(", ") };
    let mut output = format!(
        "Framework: {} ({})\nRuntime backend: {}\nServices: {}\nSSL: {}\n\nManifest:\n",
        strategy.framework,
        strategy.mode,
        strategy.backend,
        services,
        if strategy.ssl { "enabled" } else { "disabled" },
    );
    output.push_str(&render_tree(&report.entries));
    output.push_str("\nManaged services:\n");
    for service in &report.managed_services {
        let state = if service.running { "running" } else { "stopped" };
        let enabled = if service.enabled { "enabled" } else { "disabled" };
        output.push_str(&format!("- [{state}, {enabled}] {} {}\n", service.unit, service.owner));
    }
    output
}

fn render_tree(entries: &[Artifact]) -> String {
    let mut tree = Tree::new();
    let root = tree.add_node(TreeEntry { name: String::from("/"), artifact: None });
    let mut nodes = BTreeMap::from([(PathBuf::from("/"), root)]);
    let mut indices: Vec<_> = (0..entries.len()).collect();
    indices.sort_by_key(|&index| PathBuf::from(&entries[index].path));

    for index in indices {
        let mut parent_path = PathBuf::from("/");
        let mut parent = root;
        for component in Path::new(&entries[index].path).components() {
            if matches!(component, Component::RootDir) {
                continue;
            }
            parent_path.push(component.as_os_str());
            parent = *nodes.entry(parent_path.clone()).or_insert_with(|| {
                tree.add_child(
                    parent,
                    TreeEntry { name: component.as_os_str().to_string_lossy().into_owned(), artifact: None },
                )
            });
        }
        if let Some(node) = tree.get_mut(parent) {
            node.artifact = Some(index);
        }
    }

    TreeRenderer { tree: &tree, artifacts: entries, output: String::from("/\n") }.render_children(root, "")
}

struct TreeRenderer<'a> {
    tree: &'a Tree<TreeEntry>,
    artifacts: &'a [Artifact],
    output: String,
}

impl TreeRenderer<'_> {
    fn render_children(mut self, parent: usize, prefix: &str) -> String {
        self.render_child_nodes(parent, prefix);
        self.output
    }

    fn render_child_nodes(&mut self, parent: usize, prefix: &str) {
        let children = self.tree.children(parent);
        for (child_index, child) in children.iter().enumerate() {
            let last = child_index + 1 == children.len();
            if let Some(node) = self.tree.get(*child) {
                self.output.push_str(prefix);
                self.output.push_str(if last { "└── " } else { "├── " });
                self.output.push_str(node.name.as_str());
                if let Some(artifact) = node.artifact {
                    self.output.push_str(&format_artifact(&self.artifacts[artifact]));
                }
                self.output.push('\n');
                let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
                self.render_child_nodes(*child, &child_prefix);
            }
        }
    }
}

fn format_artifact(artifact: &Artifact) -> String {
    let actual = artifact.actual_kind.as_ref().map_or_else(String::new, |kind| format!(" (actual: {kind})"));
    format!(" [{}] [{}] {}{}", artifact.state, artifact.kind, artifact.owner, actual)
}

#[cfg(test)]
mod tests {
    use super::{Artifact, render_tree};

    #[test]
    fn renders_artifacts_in_a_path_tree() {
        let entries = vec![
            Artifact {
                path: String::from("/etc/nginx/sites-available/example.conf"),
                kind: String::from("file"),
                owner: String::from("runtime"),
                state: String::from("present"),
                actual_kind: None,
            },
            Artifact {
                path: String::from("/run/example/app.sock"),
                kind: String::from("socket"),
                owner: String::from("framework"),
                state: String::from("present"),
                actual_kind: None,
            },
        ];

        let output = render_tree(&entries);

        assert!(output.contains("example.conf [present] [file] runtime"));
        assert!(output.contains("app.sock [present] [socket] framework"));
    }
=======
    bonesinfra::run(&["manifest", "show", "--env-file", paths::DOT_ENV, "--format", format])
>>>>>>> feature/decentralization
}
