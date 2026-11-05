# Clarification

## Trigger

The integration audit found existing Git, SSH, and secrets wrappers that callers
bypass, plus a dead Rust-to-Python input contract.

## Decision

The integration child change will strengthen existing boundaries rather than add
parallel adapters.

Concrete bypasses are:

- `crates/bonesdeploy/src/commands/doctor.rs` directly invokes `git` instead of
  using `infra/git.rs`;
- `crates/bonesdeploy/src/commands/update/release.rs` directly invokes `ssh`
  instead of using `infra/ssh.rs`;
- Python setup directly initializes bare Git repositories through pyinfra shell
  commands because no Python Git concept exists for that responsibility;
- the secrets command assembles remote temporary-file shell commands outside its
  GPG/secrets boundary;
- Rust builds `bonesinfra_input` JSON in `commands/remote/data.rs`, but Python
  does not read that stdin contract and derives its context from `.env` instead.

The child plan must choose one owner for each behavior, migrate callers, remove
dead contracts, and add static checks against direct process bypasses. It must
preserve the Rust `bonesinfra::run*` subprocess boundary and Python's pyinfra
runner boundary.

## Supersedes

This adds concrete integration findings to the parent summary and supersedes the
generic statement that Git and SSH wrappers merely need strengthening.

## Effect on the record

- `02-plan.md` records the exact direct-process callers and dead JSON contract.
- `03-tasks.md` requires the integration child plan to address those callers and
  contracts explicitly.
