# Idea

## Request

Move the update patches logic into Python under
`crates/bonesinfra/python/src/bonesinfra` so every update is easier to read.
Use idempotent pyinfra commands for remote updates and Python code for local
updates. Organize patch code into clear modules.

## Problem

`bonesdeploy update` currently owns its patch registry, version selection,
local Git migrations, local completion markers, and remote dispatch in Rust.
Remote patches then depend on a separate `bonesremote patch` Rust command.
This splits one update concern across both Rust binaries and makes it difficult
to see the effects of an update in the Python provisioning code that already
owns remote infrastructure changes.

## Definitions

**Patch:** A version-gated, ordered migration applied by `bonesdeploy update`.
It has a stable identifier and is complete only after its per-project,
per-scope completion marker is written. A patch is not a Git diff or a user
deployment script.

**Local patch:** A patch that changes the workstation's `.bones` configuration
repository. Its completion marker is stored below the local BonesDeploy data
directory.

**Remote patch:** A patch that changes a deployment server through the local
embedded BonesInfra runtime and its pyinfra SSH connection. Its completion
marker is stored at `/var/lib/bonesdeploy/patches/<site>` on that server.

**Patch registry:** The Python-owned ordered mapping from patch identifiers to
their introduction versions and implementations. It selects only patches whose
introduction version is at or before the requested update version.

## Desired outcome

`bonesdeploy update` applies `0001-config-repo` and
`0002-root-config-repo` through the embedded BonesInfra Python runtime. The
local `.bones` repository and its `origin` retain their existing migration
behavior. The remote root-owned `.bones` repository retains its legacy
migration, canonical setup, and pre-receive hook behavior through idempotent
pyinfra operations. Failed patches leave no completion marker and retry on a
later update.

## Scope

- Move the two existing patch identifiers, their version gates, ordering,
  completion markers, and implementations into BonesInfra Python modules.
- Add a private BonesInfra patch command that Rust invokes for local and remote
  update scopes.
- Apply local repository changes with Python standard-library process and
  filesystem code.
- Apply remote repository changes through a pyinfra deploy plan over a root SSH
  connection.
- Remove the obsolete Rust update patch modules and the `bonesremote patch`
  command.
- Add focused Python tests and update patch ownership documentation.

## Constraints

- Preserve both patch identifiers, their introduction version `0.7.3`, order,
  marker locations, and externally observable repository migration behavior.
- Preserve atomic marker creation after successful local or remote patch
  completion.
- Remote patches run from the local embedded BonesInfra runtime, not by
  installing Python on the deployment server.
- Remote patch SSH uses `root`, matching the existing update flow.
- Use pyinfra operations for remote effects and Python code for local effects.
- Do not run end-to-end tests.

## Exclusions

- Changing release discovery, crates.io installation, static `bonesremote`
  download, or `.bones` scaffold synchronization.
- Changing remote setup, runtime, service, or SSL provisioning workflows.
- Adding patch identifiers or changing the configuration-repository migration's
  externally visible behavior.
- Removing unrelated Rust or shell code outside the obsolete patch paths.
