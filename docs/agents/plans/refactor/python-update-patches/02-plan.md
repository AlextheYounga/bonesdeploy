# Plan

## Current behavior

`crates/bonesdeploy/src/commands/update/mod.rs` updates the local binary and
then calls `commands/update/patches/mod.rs` before refreshing `.bones`.
That Rust module owns a two-entry registry for `0001-config-repo` and
`0002-root-config-repo`, parses the target version, selects patches introduced
in `0.7.3`, writes local markers below `bones_data_root()/patches/<project>`,
and delegates local patch behavior to `patches/local/`.

The first local patch initialises `.bones` as a Git repository and requires an
existing `origin` to match the root-owned config-repository URL. The second
sets or creates `origin` to that URL. `update/release.rs` establishes a root
SSH session, installs the requested `bonesremote` binary, prepares the project
root, and invokes the Rust remote patch dispatcher. The dispatcher calls
`bonesremote patch apply` once for every selected patch.

`crates/bonesremote/src/commands/patch.rs` validates the site and patch name,
migrates a legacy `/home/git/<site>.bones.git` repository to
`/root/.config/bonesremote/repos/<site>.bones.git`, initialises and secures the
canonical bare repository, sets `master` as its HEAD, installs the
pre-receive hook, and atomically writes a remote marker. Both patch identifiers
execute that same migration.

The Rust `bonesinfra` crate embeds the Python package, materializes it into a
local virtual environment, and runs `python -m bonesinfra`. Its current Typer
CLI delegates remote setup, runtime, service, SSL, and manifest workflows to
`pyinfra.runner.run`. The runner builds a one-host SSH inventory from
`DeployContext`; its remote plans use pyinfra `files` and `server` operations.
`setup/directories.py` already owns the canonical `.bones` repository and hook
provisioning logic but does not migrate legacy repositories or write update
markers.

## Intended behavior

The private `bonesinfra patches apply` command receives a config path, target
version, and scope. Its Python patch registry selects the two existing patches
in their existing order. For the local scope, Python applies the local Git
migrations and atomically writes the existing local marker only after each
patch succeeds. For the remote scope, the command runs a root pyinfra plan that
applies the remote migration and writes the existing remote marker only after
each patch succeeds.

`bonesdeploy update` invokes this command after the local binary update and
after the remote binary update. It retains release download and project-root
preparation, but no longer implements patch registry, selection, marker, Git,
or remote-patch behavior. `bonesremote` no longer exposes a patch command.

## Approach

Add a `patches` package under `src/bonesinfra` with a compact registry and one
module for local config-repository patches and one module for remote
config-repository operations. Keep version parsing, registry selection, and
atomic marker writes in the registry module so both scopes use one source of
truth. The local module uses `pathlib`, `subprocess`, and atomic filesystem
writes to match the current Git behavior.

Add `patches apply` to the private Typer CLI. The command reads
`DeployContext`, validates the requested scope, directly runs local patches,
or invokes the existing pyinfra runner for the remote patch plan. Extend the
runner with an explicit SSH-user override so this command connects as `root`
without changing the configured SSH user used by other BonesInfra commands.

The remote patch module plans idempotent pyinfra operations for the canonical
repository parent, legacy repository move, bare repository initialization,
ownership, `master` HEAD, and pre-receive hook. It uses the existing embedded
hook asset and writes each patch marker only after its migration operations
complete. The same remote migration remains registered under both legacy patch
identifiers to preserve marker compatibility.

Replace the Rust calls to `update::patches` with direct invocations of the
private BonesInfra command, remove the Rust patch directory and `bonesremote`
patch CLI/command wiring, and update the update-patch documentation.

## Responsibilities and boundaries

`bonesdeploy::commands::update` remains the public update coordinator. It
chooses when local and remote update scopes run and invokes BonesInfra with the
config path and target version. `update::release` remains responsible for
downloading `bonesremote` and preparing the remote project-root parent.

`bonesinfra.cli.app` owns private `patches apply` argument parsing and thin
command dispatch. `bonesinfra.patches` owns patch identity, version selection,
scope dispatch, marker paths, and marker durability. Its local module owns
workstation `.bones` Git changes. Its remote module owns pyinfra operations for
the server's config repository and its marker. `bonesinfra.pyinfra.runner`
owns SSH inventory construction and operation execution, including the
explicit update-only root-user override.

`bonesremote` retains deployment and site-import behavior but no longer owns
update patch parsing, migration, or marker creation.

## Affected areas

- `crates/bonesinfra/python/src/bonesinfra/cli/app.py`
- New `crates/bonesinfra/python/src/bonesinfra/patches/` modules for registry,
  local Git changes, and remote pyinfra operations
- `crates/bonesinfra/python/src/bonesinfra/pyinfra/runner.py`
- `crates/bonesinfra/python/tests/` for patch selection, local migration,
  remote plan, and private CLI coverage
- `crates/bonesdeploy/src/commands/update/mod.rs` and `release.rs`
- Removed `crates/bonesdeploy/src/commands/update/patches/`
- Removed `crates/bonesremote/src/commands/patch.rs` and its CLI/command module
  wiring
- `CONTEXT.md` and `crates/bonesinfra/python/CONTEXT.md`

## Decisions

- Python owns the complete patch registry and both scopes because updates are
  infrastructure migrations and the requested goal is readable, modular Python
  patch code.
- The local BonesInfra runtime executes remote patches over SSH. Python is not
  installed or invoked on deployment servers, preserving the existing client
  execution model.
- Remote operations use pyinfra instead of `bonesremote` because pyinfra is the
  existing idempotent remote-infrastructure mechanism in BonesInfra.
- The registry retains two entries that share one remote migration because
  existing marker names are durable compatibility state.
- The runner receives an explicit SSH-user override rather than modifying
  `bones.toml`, so update patches retain their root-only behavior without
  altering other provisioning commands.
- Rust patch code and the remote patch command are deleted rather than retained
  as wrappers, leaving one authoritative patch implementation.

## Risks

- Python version parsing or registry ordering can select a different patch set
  for prerelease target versions.
- A local Git command can change the current distinction between rejecting an
  unexpected existing `origin` and forcibly replacing it.
- A remote pyinfra plan can write a marker before a failed repository migration,
  preventing the required retry.
- The root SSH override can accidentally affect the configured user for
  unrelated BonesInfra commands.
- Removing the `bonesremote patch` wiring can leave stale CLI references or
  break release update compilation.

## Validation

- Add Python tests proving version-gated registry selection, prerelease parsing,
  ordered execution, and marker creation only after successful local patches.
- Add Python tests proving local `.bones` initialization, root config-repository
  `origin` behavior, and rejection of an unexpected existing `origin`.
- Add Python tests that inspect the remote pyinfra plan for legacy migration,
  canonical bare repository setup, executable pre-receive hook, root operation
  ownership, and marker behavior.
- Add private CLI and runner tests proving remote patch application requests the
  explicit root SSH user without changing standard command behavior.
- Run `ruff check .`, `ruff format .`, and `uv run pytest` from
  `crates/bonesinfra/python`; run affected Rust tests, `cargo fmt`, `cargo
  clippy`, and `shfmt -w .`; do not run end-to-end tests.
- Review the final diff to confirm the Rust registry and `bonesremote patch`
  command are removed and documentation names Python as the patch owner.
