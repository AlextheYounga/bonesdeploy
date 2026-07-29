use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Status {
    Pass,
    Fail,
    Unverified,
    NotApplicable,
}

impl Status {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Unverified => "UNVERIFIED",
            Self::NotApplicable => "NOT APPLICABLE",
        }
    }

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

pub(super) fn pass(rule: &'static str, observed: String) -> Finding {
    Finding {
        rule,
        principle: "Linux identities and writable parents define effective authority.",
        expected: "Untrusted identities cannot modify protected state.".to_string(),
        observed,
        risk: "The checked authority boundary is intact.".to_string(),
        remediation: "None.".to_string(),
        status: Status::Pass,
    }
}

pub(super) fn fail(rule: &'static str, observed: String) -> Finding {
    Finding {
        rule,
        principle: "Linux identities and writable parents define effective authority.",
        expected: "Untrusted identities cannot modify protected state.".to_string(),
        observed,
        risk: "A compromised application could gain authority outside its site.".to_string(),
        remediation: "Restore root ownership and remove write access from untrusted identities.".to_string(),
        status: Status::Fail,
    }
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
