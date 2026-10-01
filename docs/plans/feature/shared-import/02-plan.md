# Plan

## Current behavior

`bonesdeploy site export` loads the local project configuration, connects with
the configured privileged SSH identity, and runs `zip -q -r -y - shared` from
the canonical `/srv/sites/<site>` root. It streams stdout into a private local
temporary file and publishes the ZIP without overwriting an existing file. The
archive contains one top-level `shared/` directory, includes hidden files such
as `shared/.env`, and stores symbolic links rather than following them. It has
no manifest, signature, or embedded site identity, so an independently created
ZIP with the same layout is structurally equivalent.

The local CLI has no import or restore command. `SshTransport` already supports
reader-based binary uploads with the transfer deadline and bounded remote
diagnostics. BonesRemote owns typed privileged site mutations and
`SiteMutation` combines site validation, mutability checks, and the deployment
lock. Its release artifact extractor already demonstrates defensive path,
duplicate-entry, link, type, count, and expanded-size validation, but it handles
framed tar-gzip artifacts rather than ZIP shared archives.

The site's runtime user owns `shared/`; `shared/.env` is a protected exception
written atomically by `bonesdeploy secrets push`. Releases refer to that fixed
absolute environment path. Site services belong to a project-derived systemd
target and use `PartOf=<site>.target`, so stopping the target quiesces native and
Compose-backed application services. The service command can start/restart and
verify registered services, but there is no shared-import cutover operation.

Laravel provisioning creates `storage`, `storage/framework/sessions`,
`storage/framework/views`, `cache`, and `uploads` beneath `shared/`. BonesDeploy
does not assign semantic meaning to their contents. Importing an older Laravel
archive can therefore restore stale sessions, caches, compiled views, logs, and
uploads; database- or Redis-backed state outside `shared/` is unaffected.

## Intended behavior

The public command is:

```text
bonesdeploy site import <archive> [--yes]
```

The local command requires a readable regular file, loads the configured site,
prints that non-environment shared data will be replaced and that services will
briefly stop, and requires the existing exact-project-name confirmation unless
`--yes` is supplied. It then streams the file over the privileged SSH connection
to `bonesremote shared import --site <site>` and reports successful import,
environment preservation, and service restart.

BonesRemote acquires the site mutation boundary, ensures no deployment is in
flight or interrupted, and resolves only canonical site-owned paths. Before
reading a new archive it recovers any interrupted prior cutover recorded for
the site. It receives stdin into a private transaction directory on the same
filesystem as `shared/`, retaining a filesystem safety reserve rather than
allowing transfer or extraction to fill the volume.

The remote command opens the completed ZIP, validates every entry, and extracts
accepted entries into a staged replacement. All entries must be rooted beneath
`shared/`; absolute paths, parent traversal, duplicate or conflicting paths,
special files, unsafe links, overlong paths, excessive entry counts, and data
that exceeds the transaction's filesystem-space budget are rejected. Safe
relative symbolic links are created only after ordinary entries. Archive
`shared/.env` is validated as an entry but omitted from extraction. A missing
archive `.env` is accepted.

Imported directories and files receive the site runtime user and group,
special mode bits and world permissions are removed, and symbolic links are
owned without following them. Archive ownership is never trusted. The live
`shared/.env` must be a regular non-symlink file with its established protected
ownership and mode.

After staging succeeds, BonesRemote stops the site target, copies the current
environment file into the staged replacement with its protected metadata, and
records the cutover transaction durably. It atomically exchanges the live and
staged directories on their common filesystem. It starts the site target and
uses the existing registered-service verification behavior. Successful
verification removes the prior shared directory and transaction record. Failed
verification exchanges the directories back, restarts and verifies the prior
site, and returns an error that distinguishes import failure from rollback or
recovery failure.

A process interruption after services stop cannot delete either shared tree.
The durable transaction record identifies whether exchange occurred. The next
shared import first restores the prior tree when required, restarts and verifies
the site, cleans the interrupted transaction, and only then accepts new input.

`secrets push` sends its validated plaintext through a typed BonesRemote shared
environment command that takes the same `SiteMutation` lock before performing
the existing atomic environment replacement. This preserves its public behavior
while preventing an environment update from racing shared cutover.

## Approach

1. Extend the local Clap site command and thin dispatcher with positional
   `<archive>` and `--yes`, then implement a focused site-import orchestrator
   that performs local file validation, confirmation, privileged connection,
   streaming upload, close handling, and success output.
2. Add fixed shared-operation command builders beside the existing remote
   infrastructure command helpers. Stream archive input with
   `SshTransport::stream_cmd_with_reader`; do not buffer shared archives in the
   local process or construct arbitrary remote paths.
3. Add a BonesRemote `Shared` command group with typed `Import` and environment
   installation operations. Both validate root privilege and acquire
   `SiteMutation`; import additionally requires idle deployment state.
4. Add a focused shared-import module that owns transaction receipt, ZIP
   validation and extraction, ownership normalization, environment carryover,
   systemd cutover, durable recovery, rollback, and cleanup. Reuse the artifact
   extractor's validation rules where they represent the same archive safety
   concepts, without generalizing tar and ZIP extraction into one format-heavy
   abstraction.
5. Add the existing `zip` crate version used by BonesInfra to BonesRemote and
   use private temporary files/directories on the project filesystem. Use the
   Linux atomic directory-exchange primitive for cutover and encapsulate its
   small unsafe boundary with explicit path and same-filesystem checks.
6. Route `secrets push` plaintext through the typed shared environment operation
   instead of its current command-local remote shell mutation. Preserve dotenv
   validation, protected ownership/mode, and user-visible behavior.
7. Document accepted archive construction, destructive replacement, `.env`
   preservation, downtime and rollback, space requirements, and opaque volatile
   state consequences. Include a Laravel note recommending manual application
   checks such as `php artisan optimize:clear` after an import when appropriate.

## Responsibilities and boundaries

| Boundary | Responsibility |
| --- | --- |
| `bonesdeploy` CLI arguments and dispatch | Parse `site import <archive> [--yes]` and route without business logic. |
| Local site-import command | Validate the local file, obtain confirmation, stream it, and present results. |
| `SshTransport` | Provide bounded binary stdin transfer and remote diagnostic handling. |
| BonesDeploy infrastructure command helpers | Construct fixed typed BonesRemote shared commands from a validated configured site. |
| BonesRemote shared command group | Enforce root, acquire the site mutation boundary, and dispatch shared-data operations. |
| BonesRemote shared-import module | Own archive trust-boundary validation, staging, transaction state, cutover, recovery, rollback, and cleanup. |
| Existing service/systemd inspection code | Stop, start, and verify the project-derived site target and its registered services. |
| Shared environment operation | Serialize and atomically install validated plaintext environment content without exposing arbitrary paths. |
| Application operator | Decide whether imported framework state needs application-specific cleanup or database coordination. |

## Affected areas

- `crates/bonesdeploy/src/cli/args.rs` and `cli/dispatch.rs`
- `crates/bonesdeploy/src/commands/site/` and its module exports
- `crates/bonesdeploy/src/commands/secrets/mod.rs`
- `crates/bonesdeploy/src/infra/` remote command helpers and focused tests
- `crates/bonesdeploy/tests/commands.rs`
- `crates/bonesremote/Cargo.toml`
- `crates/bonesremote/src/cli/args.rs` and `cli/dispatch.rs`
- `crates/bonesremote/src/commands/` shared import, environment, and service lifecycle code
- `crates/bonesremote/src/release/` mutation/state helpers needed for durable import serialization
- Focused BonesRemote archive-security, transaction, rollback, and recovery tests
- `README.md`, `CONTEXT.md`, and relevant architecture/security references

## Decisions

- Replace rather than merge so the result corresponds to the archive and stale
  remote files cannot silently survive.
- Preserve the live `.env` and ignore the archive copy because encrypted local
  secrets and explicit `secrets push` remain the environment source of truth.
- Stage on the remote site filesystem and exchange whole directories because
  in-place extraction cannot provide all-or-nothing replacement or safe
  rollback.
- Keep services running during receipt and extraction, then stop them for
  cutover because this minimizes downtime while preventing writes to an old
  tree and stale open handles after replacement.
- Use a typed BonesRemote operation instead of a privileged arbitrary extraction
  shell command because ZIP input is untrusted and mutates runtime-owned data.
- Accept structurally equivalent archives rather than only bytes emitted by
  `site export` because the export format intentionally has no manifest and a
  manually rsynced site can reproduce the same `shared/` layout.
- Restore volatile application files exactly and document their effects because
  framework-specific cache/session policies would violate generic replacement
  semantics and could silently delete meaningful application data.
- Normalize imported ownership and restrictive modes because ZIP identity and
  mode metadata are untrusted and the runtime ownership boundary is a
  provisioning/security contract.
- Serialize environment publication with import because copying `.env` without
  a common lock could discard a concurrent successful `secrets push`.
- Retain durable cutover state until service verification succeeds because
  ordinary error rollback alone does not cover process termination or host
  interruption.

## Risks

- A large archive requires simultaneous space for the compressed upload, staged
  data, and current shared tree. Receipt and extraction must stop before the
  filesystem safety reserve is consumed and leave live data untouched.
- Malicious ZIP metadata can attempt traversal, symlink escape, duplicate-path
  replacement, special-file creation, decompression exhaustion, or permission
  widening. Validation and extraction must never use convenience extraction
  that trusts archive paths.
- Failure after services stop can leave the site unavailable even though both
  data trees remain. Durable transaction state and start-of-command recovery
  must restore service or return explicit manual recovery information.
- A rollback restart can also fail. Errors must preserve both failure contexts
  and must not delete either tree when recovery is incomplete.
- Imported Laravel sessions, caches, views, and uploads may be stale or
  inconsistent with the destination database and preserved `APP_KEY`. The
  command cannot prove application-level consistency; warnings and
  documentation must make this operator responsibility explicit.
- Moving environment publication behind BonesRemote could change diagnostics or
  ownership if the existing validation and atomic-write behavior are not
  preserved exactly.
- Atomic directory exchange is Linux-specific. The implementation must report a
  clear unsupported-filesystem/kernel error before deleting or renaming live
  data.

## Validation

- CLI parsing tests prove the positional archive and `--yes` forms are accepted
  and malformed command shapes are rejected.
- Local command tests prove missing, non-regular, and unreadable archives fail
  before SSH; declined confirmation performs no transfer; and transfer/close
  failures do not report success.
- ZIP tests prove export-compatible and manually constructed archives import;
  `.env` is omitted; safe relative links survive; and traversal, absolute paths,
  unsafe links, duplicates, path conflicts, special types, overlong paths,
  excessive entries, malformed ZIPs, and space-budget violations fail before
  cutover.
- Filesystem tests prove imported entries receive the runtime identity and
  restrictive modes while the live `.env` content and protected metadata remain
  unchanged.
- Transaction tests prove successful replacement removes files absent from the
  archive, preserves services during staging, stops them during cutover, and
  deletes the old tree only after verified restart.
- Failure tests prove extraction leaves live data unchanged, restart failure
  restores the prior tree, rollback failure retains both trees and actionable
  state, and a later invocation recovers an interrupted pre- or post-exchange
  transaction before reading a new archive.
- Concurrency tests prove deploy, import, and environment publication serialize
  on the site mutation lock and a successful `secrets push` cannot be lost by
  cutover.
- Run focused crate tests, the full non-E2E Rust and Python suites,
  `cargo clippy`, `cargo fmt`, `shfmt -w .`, Python Ruff checks, and
  `git diff --check`. Do not run E2E tests.
- Review rendered `--help`, README examples, architecture/security statements,
  and the final diff for obsolete direct environment mutation, unsafe archive
  handling, accidental framework policy, or unrelated changes.
