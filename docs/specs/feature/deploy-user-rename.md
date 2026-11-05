# Deploy User Rename

<!-- Merged from feature/rename-deploy-user -->

## Idea

### Request

Rename the historical `git` deployment user to `deploy` now that local
artifact builds and SSH artifact transport have replaced Git-push deployment.
Treat the rename as a breaking host contract and do not migrate accounts from
past releases.

### Problem

The production account named `git` no longer hosts repositories, accepts Git
pushes, exports source, or builds applications. It is the narrow SSH transport
identity for routine deployment, but its old name describes removed behavior
and obscures its current security role in code, diagnostics, provisioning, and
documentation.

### Definitions

**Deploy identity:** The global production Unix account named `deploy`
that accepts authorized-key SSH connections for routine artifact deployment.
It may invoke only the exact root-owned BonesRemote config-sync and deploy
commands granted by the BonesDeploy sudoers policy. It is not an application
runtime identity, build identity, or administrative SSH identity.

**Administrative SSH identity:** The configured privileged account used by
server and site administration. It remains separate from the deploy identity
and is not renamed by this change.

**Runtime identity:** The dedicated per-site Unix account that runs an
application and owns its mutable shared state. Runtime identities remain
isolated from the global deploy identity.

### Desired outcome

Fresh BonesDeploy server provisioning creates `deploy` as the deploy
identity, installs its authorized keys, grants its existing narrow sudo policy,
and does not create a `git` or `bonesdeploy` account. Routine deployment, remote
version checks, and SSH connectivity checks connect as `deploy`. BonesRemote
diagnostics and security checks recognize `deploy` as the deploy identity and continue
to reject membership in site runtime groups.

Code, tests, generated assets, architecture descriptions, security guidance,
and user documentation consistently call this account `deploy` or the
deploy identity. References to Git remain only where they describe local source
selection or actual Git operations.

### Scope

- Change the canonical Rust and Python deploy-account values from `git` to
  `deploy`.
- Provision `/home/deploy`, copy the configured administrative authorized keys
  into it, and render the existing sudoers allowlist for `deploy`.
- Make routine BonesDeploy SSH transport use `deploy`.
- Update BonesRemote identity collection, site isolation checks, doctor output,
  tests, and fixtures to use the renamed account.
- Reserve the project/site name `deploy` so runtime identity provisioning cannot
  collide with the global deploy account.
- Update embedded guidance and repository documentation that describe the
  production deploy identity or its home directory.
- Remove stale deploy-account terminology and test data exposed by the rename.

### Constraints

- This is a breaking production-host contract. Existing `git` and
  `bonesdeploy` accounts,
  authorized keys, home contents, UIDs, GIDs, and sudoers entries are not
  migrated or retained for compatibility.
- Existing hosts must be reprovisioned to satisfy the new contract.
- The deploy identity's privilege boundary remains unchanged: it receives no
  runtime-group or Docker-group membership and may sudo only exact BonesRemote
  config-sync and deploy command forms.
- The account continues to use authorized-key SSH and a shell capable of
  executing the existing remote commands.
- Per-site runtime identities and the configured administrative SSH identity
  remain unchanged.
- E2E scenarios may be updated but are not run by an agent unless explicitly
  requested.

### Exclusions

- In-place account rename, UID/GID preservation, home-directory movement,
  authorized-key migration, legacy sudoers cleanup, or update patches for hosts
  provisioned by past releases.
- Automatic fallback from `deploy` to `git` or `bonesdeploy` during SSH connection.
- Changes to the two-command sudoers authority or the root-owned BonesRemote
  deployment lifecycle.
- Per-site deploy identities, multi-operator tenant isolation, forced-command
  SSH, or broader SSH-account hardening.
- Renaming local Git concepts, commands, repositories, branches, or revision
  provenance that still represent actual Git behavior.


## Plan

### Current behavior

`bonesdeploy-core/src/paths.rs` defines the canonical Rust deploy-account value
as `git`, and `default_deploy_user()` exposes it to BonesDeploy. Routine deploy,
remote-version, and SSH-connectivity paths use `infra::ssh::connect`, which
always connects as that account. Deployment then streams the sanitized control
plane and local artifact through the exact sudoed BonesRemote commands.

BonesInfra separately defines `DEPLOY_USER = "git"` in its Python context.
Server setup creates that account with a home and Bash shell, copies the
configured administrative account's `authorized_keys` into `/home/git/.ssh`,
and renders `/etc/sudoers.d/bonesdeploy` with the `git` principal. The account
does not own project releases and is not placed in runtime or Docker groups.

BonesRemote uses the Rust constant when collecting the deploy account and when
checking that it is absent from every runtime group. The server and site doctor
therefore require `git`, while sudoers tests, security tests, embedded skill
content, and architecture/security documentation encode the same name.

The repository's remote update-patch mechanism is scoped to compatibility work,
but this feature deliberately provides no old-account migration. Privileged
administration and some site operations continue to use the configured
administrative SSH identity rather than the deploy identity.

### Intended behavior

The canonical deploy identity is `deploy` in both Rust and Python. Fresh
server setup creates `/home/deploy`, installs its authorized keys, and
grants that principal the same two anchored BonesRemote sudo command forms.
Routine deployment and unprivileged connectivity/version probes connect as
`deploy`.

BonesRemote server and site diagnostics require the `deploy` account and
verify that it remains isolated from every runtime identity. User-facing and
architectural descriptions identify it as the deploy identity or `deploy`,
without implying that it hosts application Git repositories.

No code detects, renames, copies from, deletes, or falls back to a legacy `git`
or `bonesdeploy` account. A host provisioned under an old contract is
unsupported until it is reprovisioned under the new contract.

### Approach

Replace the two existing cross-language deploy-account constants with `deploy`
and let their current consumers carry the new identity through
SSH, provisioning templates, sudoers rendering, and diagnostics. Keep the
current separation between Rust and embedded Python constants because each
runtime must package its own value and the project already tests the rendered
boundary.

Update focused tests and fixtures to assert the new account, home, SSH command,
sudoers principal, and identity-isolation behavior. Search human-authored source
and documentation for deploy-account uses of `git`, changing only references to
the removed production identity while preserving genuine local Git terminology.

Reserve `deploy` in the existing project/site validation lists in both Rust and
Python because those names become runtime users and groups. Do not extend the
patch registry or add compatibility branches. Fresh server setup is the only
account-provisioning path covered by this feature.

### Responsibilities and boundaries

- `bonesdeploy-core` owns the canonical Rust deploy-account name shared by the
  CLI and BonesRemote.
- `bonesdeploy::infra::ssh` continues to own selection of the routine SSH
  principal; callers do not hardcode the account.
- BonesInfra server-user provisioning owns account and authorized-key creation,
  while its sudoers operation owns the exact root elevation policy.
- BonesRemote doctor and security modules own deploy-account existence and
  runtime-group isolation checks.
- Existing tests at each boundary own proof that constants propagate into SSH,
  provisioning, sudoers, and security evaluation.
- Repository and embedded documentation own the operator-facing terminology and
  breaking host expectation.

### Affected areas

- Rust identity constants and consumers under `crates/bonesdeploy-core`,
  `crates/bonesdeploy`, and `crates/bonesremote`.
- BonesInfra context, server-user provisioning, authorized-key script,
  sudoers template, and their Python tests.
- BonesRemote doctor, inspection, and security tests containing deploy-account
  fixtures or expected messages.
- E2E harness comments or assertions that name the provisioned deploy account.
- `README.md`, `CONTEXT.md`, BonesInfra context, architecture and security docs,
  and embedded BonesDeploy skill material that describe the account as `git`.
- Generated or embedded BonesInfra artifacts affected by Python source or asset
  changes, following the repository's existing generation checks.

### Decisions

- Name the account `deploy`, matching the user's requested concise Unix identity.
- Preserve the dedicated deploy identity rather than using root SSH, because it
  remains the narrow transport principal for the sudo-controlled deployment
  boundary.
- Make a clean breaking change with no `git` or `bonesdeploy` fallback or
  migration. This keeps
  account state single-sourced and avoids unrequested handling of arbitrary old
  home contents and identity collisions.
- Preserve the current sudoers command allowlist exactly; local building did not
  remove the need for privileged release activation and service management.
- Preserve real Git terminology for local committed-source selection. Only the
  production Unix identity is renamed.
- Reserve the `deploy` project/site name because runtime identities are derived
  directly from those names.

### Risks

- A missed hardcoded `git` principal can make deployment, doctor, or sudoers
  disagree about which account is authoritative.
- Broad text replacement can incorrectly rename genuine local Git behavior or
  historical planning records.
- Hosts provisioned by older releases will reject routine SSH as `deploy`;
  this is intentional but must be stated clearly wherever the host contract is
  documented.
- Python source or template changes can leave the embedded BonesInfra wheel or
  generated artifacts stale if repository generation checks are skipped.
- Weakening group-isolation assertions while changing fixtures could silently
  grant the deploy identity access to runtime state.

### Validation

- Focused Rust tests prove `default_deploy_user()` and routine SSH command
  construction select `deploy`, project validation rejects `deploy`, and
  BonesRemote doctor/security evaluation
  imports that account and rejects its runtime-group membership.
- Focused Python tests prove server setup provisions `deploy`, copies keys into
  `/home/deploy/.ssh`, project validation rejects `deploy`, and renders the unchanged anchored sudoers
  command forms for the new principal.
- Repository searches show no current production deploy-account references to
  `git` or `/home/git`; remaining matches describe actual Git behavior or
  historical planning records.
- Run all non-E2E Rust and Python tests, `cargo clippy`, `cargo fmt`, `shfmt -w
  .`, Python Ruff checks and formatting, generated-artifact validation, and
  `git diff --check`. Do not execute E2E scenarios without explicit instruction.
- Review the final diff for accidental migration logic, `git` fallback,
  privilege changes, runtime-group access, unrelated Git renames, and stale
  documentation.


## Tasks

### Implementation

- [x] Change the canonical Rust and Python deploy-account values to
  `deploy`, then update routine SSH, BonesRemote doctor, and identity
  fixtures so every current consumer resolves the renamed account.
- [x] Update BonesInfra fresh-server provisioning and authorized-key assertions
  to create `/home/deploy` and install the configured key without adding
  legacy-account detection, rename, copy, deletion, or fallback behavior.
- [x] Render and test `/etc/sudoers.d/bonesdeploy` for the `deploy`
  principal while preserving the exact anchored config-sync and deploy command
  allowlist.
- [x] Update security and site-doctor tests to prove `deploy` is required,
  remains outside runtime groups, and receives no Docker or broader sudo
  authority.
- [x] Update affected E2E harness expectations without running E2E scenarios,
   and refresh generated or embedded BonesInfra artifacts required by the
  repository after Python source and asset changes.
- [x] Replace current deploy-account uses of `git`, `bonesdeploy`, and `/home/git`
  in user,
  architecture, security, context, and embedded skill documentation while
  preserving references to actual local Git behavior and historical plans.

### Validation

- [x] Run focused Rust tests for deploy-user defaults, SSH transport, and
  BonesRemote account/isolation diagnostics; confirm expected output names
  `deploy` and group-isolation failures remain enforced.
- [x] Run focused BonesInfra tests for user provisioning, authorized-key paths,
  sudoers rendering, and server provisioning order; confirm no migration or
  compatibility operation is emitted.
- [x] Search current source, assets, tests, and operational documentation for
  `git`, `deploy`, `bonesdeploy`, and `/home/git`; classify every remaining match as genuine Git behavior
  or immutable historical context.
- [x] Run `cargo test --workspace --exclude e2e --no-fail-fast`, the complete
  BonesInfra Python test suite, and repository generated-artifact checks without
  executing E2E scenarios.
- [x] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`, `ruff format
  .`, and `git diff --check`, addressing every warning, error, and formatting
  change.

### Completion

- [x] Review the final diff for legacy-account migration, SSH fallback,
  privilege drift, runtime-group access, unrelated Git terminology changes,
  stale generated content, and documentation that fails to state the breaking
  reprovisioning requirement.
- [x] Record validation evidence, meaningful deviations, and deliberately
  unfinished work in the completion notes.

### Completion notes

The earlier breaking rename is committed, and this follow-up changes the
production account from `bonesdeploy` to `deploy` without migration or fallback
behavior. The project/site name `deploy` is reserved to prevent runtime-user
collision. The stale repository-era `REPO_PATH` test value remains aligned with
the current empty production repository contract.

Focused Rust and Python tests, the complete 508-test BonesInfra suite, embedded
artifact checks, and the full non-E2E Rust workspace suite passed. The SSH
transport suite initially encountered a host-key collision when the full Rust
suite ran concurrently with another fixture process; its sequential rerun
passed all seven tests. Clippy, cargo fmt, shfmt, Ruff, and diff checks passed.
E2E scenarios remain deliberately excluded by repository policy.


## Deploy Account Name Clarity

### Trigger

The requested production deploy account name changed from `bonesdeploy` to
`deploy` after the initial implementation was completed.

### Decision

Use `deploy` as the global production deploy Unix account. Preserve the
product name `BonesDeploy` and the `/etc/sudoers.d/bonesdeploy` policy filename.
Reserve the project/site name `deploy` in both Rust and Python validation so a
runtime user and group cannot collide with the global deploy account.

The change remains a breaking fresh-host contract. Existing hosts using either
`git` or `bonesdeploy` must be reprovisioned; no account migration or SSH
fallback is added.

### Supersedes

Supersedes the account-name decision in `01-idea.md` and `02-plan.md` that
selected `bonesdeploy` as the Unix deploy account. The product and sudoers
policy names remain unchanged.

### Required Authoritative Updates

- `01-idea.md`: describe `deploy` as the deploy identity and reserve the
  project/site name.
- `02-plan.md`: update the intended behavior, decisions, risks, and validation
  to cover the account collision.
- `03-tasks.md`: add the account rename and project-name validation work.
