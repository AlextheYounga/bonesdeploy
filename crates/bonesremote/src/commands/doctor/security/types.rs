use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Pass,
    Fail,
    Unverified,
    NotApplicable,
}

impl Status {
    pub fn requires_failure(self) -> bool {
        matches!(self, Self::Fail | Self::Unverified)
    }
}

pub struct Finding {
    pub rule: &'static str,
    pub observed: String,
    pub remediation: String,
    pub status: Status,
}

#[derive(Clone, Debug)]
pub struct Account {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub shell: String,
    pub groups: BTreeSet<u32>,
}

#[derive(Clone, Debug)]
pub struct Site {
    pub name: String,
    pub project_root: PathBuf,
    pub runtime: Account,
    pub build: Account,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Clone, Debug)]
pub struct FileNode {
    pub path: PathBuf,
    pub kind: FileKind,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub has_acl: bool,
}

#[derive(Clone, Debug)]
pub struct PathTree {
    pub requested: PathBuf,
    pub nodes: BTreeMap<PathBuf, FileNode>,
}

impl PathTree {
    pub fn node(&self, path: &Path) -> Option<&FileNode> {
        self.nodes.get(path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrentState {
    Missing,
    Broken,
    NotSymlink,
    Active(PathBuf),
}

#[derive(Clone, Debug)]
pub struct ReleaseEvidence {
    pub site: String,
    pub releases_root: PathBuf,
    pub current: CurrentState,
    pub filesystem: PathTree,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    Allowed(String),
    Denied,
    Unverified(String),
}

#[derive(Clone, Debug)]
pub struct SudoEvidence {
    pub user: String,
    pub decision: PolicyDecision,
}

#[derive(Clone, Copy)]
pub struct Rule {
    pub name: &'static str,
    pub remediation: &'static str,
}

pub fn unverified(rule: &'static str, observed: String) -> Finding {
    Finding {
        rule,
        observed,
        remediation: "Fix read-only inspection access and rerun doctor as root.".to_string(),
        status: Status::Unverified,
    }
}

pub fn finding(status: Status, rule: Rule, observed: String) -> Finding {
    Finding { rule: rule.name, observed, remediation: rule.remediation.to_string(), status }
}
