use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Status {
    Pass,
    Fail,
    Unverified,
    NotApplicable,
}

impl Status {
    pub(super) fn requires_failure(self) -> bool {
        matches!(self, Self::Fail | Self::Unverified)
    }
}

pub(super) struct Finding {
    pub(super) rule: &'static str,
    pub(super) principle: &'static str,
    pub(super) expected: String,
    pub(super) observed: String,
    pub(super) risk: String,
    pub(super) remediation: String,
    pub(super) status: Status,
}

#[derive(Clone, Debug)]
pub(super) struct Account {
    pub(super) name: String,
    pub(super) uid: u32,
    pub(super) gid: u32,
    pub(super) shell: String,
    pub(super) groups: BTreeSet<u32>,
}

#[derive(Clone, Debug)]
pub(super) struct Site {
    pub(super) name: String,
    pub(super) project_root: PathBuf,
    pub(super) runtime: Account,
    pub(super) build: Account,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FileKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Clone, Debug)]
pub(super) struct FileNode {
    pub(super) path: PathBuf,
    pub(super) kind: FileKind,
    pub(super) uid: u32,
    pub(super) gid: u32,
    pub(super) mode: u32,
    pub(super) has_acl: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PathTree {
    pub(super) requested: PathBuf,
    pub(super) nodes: Vec<FileNode>,
}

impl PathTree {
    pub(super) fn node(&self, path: &Path) -> Option<&FileNode> {
        self.nodes.iter().find(|node| node.path == path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum CurrentState {
    Missing,
    Broken,
    NotSymlink,
    Active(PathBuf),
}

#[derive(Clone, Debug)]
pub(super) struct ReleaseEvidence {
    pub(super) site: String,
    pub(super) releases_root: PathBuf,
    pub(super) current: CurrentState,
    pub(super) filesystem: PathTree,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum PolicyDecision {
    Allowed,
    Denied,
    Unverified(String),
}

#[derive(Clone, Debug)]
pub(super) struct SudoEvidence {
    pub(super) user: String,
    pub(super) decision: PolicyDecision,
}

#[derive(Clone, Copy)]
pub(super) struct Rule {
    pub(super) name: &'static str,
    pub(super) principle: &'static str,
    pub(super) expected: &'static str,
    pub(super) risk: &'static str,
    pub(super) remediation: &'static str,
}

pub(super) fn unverified(rule: &'static str, observed: String) -> Finding {
    Finding {
        rule,
        principle: "Inaccessible evidence cannot prove a security boundary.",
        expected: "The doctor can inspect the complete boundary without changing it.".to_string(),
        observed,
        risk: "The boundary may be weakened but could not be verified.".to_string(),
        remediation: "Fix read-only inspection access and rerun doctor as root.".to_string(),
        status: Status::Unverified,
    }
}

pub(super) fn finding(status: Status, rule: Rule, observed: String) -> Finding {
    Finding {
        rule: rule.name,
        principle: rule.principle,
        expected: rule.expected.to_string(),
        observed,
        risk: rule.risk.to_string(),
        remediation: rule.remediation.to_string(),
        status,
    }
}
