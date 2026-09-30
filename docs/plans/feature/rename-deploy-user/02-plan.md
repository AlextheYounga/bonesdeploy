# Plan

## Current behavior

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

## Intended behavior

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

## Approach

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

## Responsibilities and boundaries

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

## Affected areas

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

## Decisions

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

## Risks

- A missed hardcoded `git` principal can make deployment, doctor, or sudoers
  disagree about which account is authoritative.
- Broad text replacement can incorrectly rename genuine local Git behavior or
  historical planning records.
- Hosts provisioned by older releases will reject routine SSH as `bonesdeploy`;
  this is intentional but must be stated clearly wherever the host contract is
  documented.
- Python source or template changes can leave the embedded BonesInfra wheel or
  generated artifacts stale if repository generation checks are skipped.
- Weakening group-isolation assertions while changing fixtures could silently
  grant the deploy identity access to runtime state.

## Validation

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
