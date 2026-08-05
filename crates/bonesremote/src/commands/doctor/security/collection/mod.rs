//! Read-only collection of host evidence used by the security evaluators.
//!
//! Each collector owns a single source of evidence: the account databases,
//! the site inventory, the filesystem tree, or the sudo policy. Collectors
//! only gather; they never decide policy.

mod accounts;
mod filesystem;
mod sites;
mod sudo;

pub(super) use accounts::{collect_accounts, collect_identity_groups};
pub(super) use filesystem::{collect_path_tree, collect_release};
pub(super) use sites::collect_sites;
pub(super) use sudo::collect_sudo_policy;
