# Idea

## Request

Correct the update "patches" approach so it does not depend on Unix shell
scripts. Write local patches in Rust and use the least-cost Rust solution for
remote patches.

## Problem

`bonesdeploy update` embeds Bash scripts and starts `bash` to apply local
patches. The local update workflow therefore cannot run on systems without a
Unix shell. Remote patches also embed and execute shell scripts even though the
installed `bonesremote` binary can own that server-side work.

## Definitions

**Patch:** A version-gated, ordered migration applied during `bonesdeploy`
update. A patch has a stable identifier and is complete only after its
per-project marker has been written. It is not a Git diff or a user deployment
script.

**Local patch:** A patch executed by the local `bonesdeploy` binary against the
project's `.bones` configuration repository. Local patches use the local data
directory for completion markers.

**Remote patch:** A patch executed by the installed `bonesremote` binary on
the deployment server. Remote patches use `/var/lib/bonesdeploy/patches/<site>`
for completion markers.

## Desired outcome

On a machine with Rust's supported platform runtime and Git, `bonesdeploy`
update applies its local configuration-repository migrations without requiring
Bash. Remote configuration-repository migrations retain their existing
observable result, retry behavior, ordering, version gating, and completion
markers while running through `bonesremote` Rust code.

## Scope

- Replace the embedded local Bash patches with Rust implementations in
  `bonesdeploy`.
- Replace the embedded remote Bash patches with a Rust-owned `bonesremote`
  patch command invoked by the existing SSH update flow.
- Preserve the two existing configuration-repository patch identifiers and
  their migration behavior.
- Add regression tests for patch selection and the Rust-owned migration
  behavior.

## Constraints

- Do not require Bash to apply local patches.
- Preserve ordered version selection and per-project, per-scope idempotent
  completion markers.
- Keep remote patch execution within the existing root SSH update flow.
- Keep the generated Git `pre-receive` hook as a shell script because Git runs
  hooks as executable programs.
- Do not run end-to-end tests.

## Exclusions

- Supporting remote operating systems other than the existing Linux server
  target.
- Changing patch versions, identifiers, or the configuration-repository
  migration's externally visible behavior.
- Replacing unrelated shell usage in update, deployment, or Git hooks.
