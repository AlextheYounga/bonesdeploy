# Plan

## Current behavior

`bonesdeploy site setup` is parsed in
`crates/bonesdeploy/src/cli/args.rs`, dispatched by `cli/dispatch.rs`, and
orchestrated in `commands/site/setup.rs`. It verifies the server baseline, sends
a typed provisioning request to `bonesinfra site apply`, provisions configured
services and the framework runtime, then runs site diagnostics. Site setup has a
default-no confirmation and supports `--yes`.

BonesInfra owns provisioning through PyInfra. `DeployContext` and
`DeploymentPaths` derive site identities and remote paths. Common site setup
creates runtime and build users, groups, the bare repository, site directories,
BonesRemote state, a placeholder release, and optional Borg backups. Service and
framework modules add database or cache resources, systemd units, nginx
configuration, AppArmor profiles, runtime paths, containers or images, SSL, and
Quick Tunnel resources.

`crates/bonesinfra/python/src/bonesinfra/manifest.py` currently combines common,
framework, configured-service, and SSL declarations. It models filesystem
`Artifact` values and `ManagedService` values, inspects them read-only, and
renders text or JSON through `bonesinfra manifest show`. Project-owned
`infra/custom/manifest.py` declarations are composed after the selected managed
framework manifest. `bonesdeploy site manifest` delegates to this command and
does not interpret the manifest in Rust.

The manifest does not currently represent Unix identities, database identities,
full Certbot certificate state, backup behavior, or runtime objects that are not
paths. It also accepts explicit artifact paths from project manifests for
inspection; those paths cannot safely be passed directly to a root deletion
operation.

BonesRemote serializes remote mutations with `SiteMutation` and a per-site
`DeploymentLock`. Deploy, rollback, failed-release cleanup, prune, service
restart, and backup operations use that boundary, but no durable state prevents
a new deployment from beginning while BonesInfra removes a site.

## Intended behavior

`bonesdeploy site delete` will load and validate the local configuration, build
the same typed request used by provisioning, and ask BonesInfra to resolve and
validate the complete deletion manifest without mutating the host. The command
will display a concise summary of the site and resources to be destroyed. Unless
`--yes` is present, the operator must type the exact configured project name;
empty, mismatched, or cancelled input aborts without remote mutation.

After confirmation, BonesDeploy will ask BonesRemote to begin decommissioning.
BonesRemote will acquire the existing site deployment lock, fail if a deployment
or other mutation holds it, atomically persist the decommissioning marker and
validated deletion plan, and keep the privileged coordinator session and lock
open for the full teardown. Every normal `SiteMutation` acquisition will reject
a decommissioning site. A retry reuses the persisted plan; a concurrent deletion
fails on the same lock instead of running a second teardown.

BonesInfra will then remove resources declared by the validated manifest in a
fixed safe order:

1. Remove public ingress and scheduled jobs so no new traffic or backup work
   starts.
2. Stop and disable the site target and every site-specific systemd service.
3. Remove site runtime containers, images, service state, databases, database
   accounts, and cache data without changing shared engines or packages.
4. Remove site certificates through Certbot and remove site AppArmor profiles,
   systemd units, target memberships, nginx files, runtime files, logs, and
   backup repositories.
5. Disable build-user linger and remove site runtime/build users and groups after
   their processes and owned data no longer need those identities.
6. Remove the site project tree, configuration tree, bare Git repository, and
   remaining declared paths.
7. Reload affected shared managers such as systemd, nginx, and AppArmor only
   after their site-specific configuration has been removed and validated.

Missing resources will be treated as already deleted. Resource-specific failures
will stop completion, preserve the decommissioning marker, and identify the
failed resource so rerunning the command continues from the remaining inventory.

After teardown, BonesInfra will inspect the persisted plan and fail if any
provisioning-owned resource remains. BonesDeploy will only then signal success
to the still-open BonesRemote coordinator. While retaining the lock, BonesRemote
removes its own mutable deployment state and atomically records a deletion
tombstone containing the deletion plan. A final inspection verifies every
declaration, including BonesRemote-owned state, is absent while the coordinator
still holds the lock. BonesDeploy then acknowledges final verification so the
coordinator marks the tombstone verified, exits, and releases the lock. The
tombstone continues rejecting delayed or manually invoked mutations after the
bare repository and stored site configuration are gone.

If teardown, verification, the local process, or the SSH connection fails, the
coordinator releases its lock on process exit. Before BonesRemote state cleanup,
the decommissioning marker and deletion plan remain. After state cleanup, an
unverified tombstone and the plan remain. A retry acquires the lock, resumes from
the persisted inventory, repeats both verification phases, and marks the
tombstone verified only after final inspection succeeds.

When `site setup` is later run for the same name, the tombstone remains active
while BonesInfra reprovisions the site. Setup uses a corresponding privileged
coordinator session to hold the site lock for all provisioning phases. After
every setup phase succeeds, that coordinator removes the tombstone before the
existing final doctor flow. Failed setup leaves the tombstone in place and
deployments blocked. BonesInfra's other mutating site entry points reject
decommissioning and deleted sites unless they run inside the authorized setup or
delete coordinator flow. Setup rejects an unverified tombstone and directs the
operator to finish deletion first.

`bonesdeploy site manifest` will render the expanded resource kinds as well as
paths and managed services. It remains read-only and does not expose passwords,
certificate private keys, file contents, or executable teardown commands.

## Approach

Extend `bonesinfra.manifest` from two parallel lists into one typed resource
inventory with an explicit resource kind, stable identity, ownership scope, and
inspection state. Keep collection composed from common declarations, the
selected framework manifest, configured runtime services, SSL, and backup
configuration. Extend framework and service declarations rather than creating a
separate deletion registry.

Resolve the inventory into a stable, secret-free deletion plan before mutation.
Pass that opaque plan to BonesRemote's decommission coordinator for atomic
persistence beside decommission state. On first deletion, the coordinator
returns the stored plan it accepted; on retries, it returns the existing plan
instead of replacing it. BonesInfra validates the returned plan again and uses
it for deletion and verification. Rust transports the plan but does not
interpret its resource schema.

Add trusted inspection and deletion handlers in BonesInfra for each supported
resource kind. Filesystem handlers will accept only canonical absolute paths
that equal a configured site-owned path or are descendants of approved
site-owned roots. Identity handlers will require exact project-derived user,
group, unit, database, certificate-domain, container, image, and service names.
Manifest validation will resolve and validate the complete inventory before
decommissioning starts. Deletion handlers will receive typed declarations and
will not execute command strings from managed or custom manifests.

Filesystem handlers will anchor operations to trusted, root-owned parent
directory descriptors and use no-follow relative inspection and removal. They
will unlink symlinks rather than traversing them, refuse mount-point crossings,
and fail when a path changes kind or escapes its approved boundary between
preflight and deletion. Recursive deletion will use a symlink-attack-resistant,
same-filesystem implementation rather than interpolated `rm -rf` commands.

Keep shared service credentials out of manifest output. Database and account
names are deterministic from the validated project name and existing service
configuration; deletion will use the existing local service request boundary
when administrative operations require service context. Password values will
never be rendered.

Add a narrow, framed coordinator protocol to BonesRemote for decommission and
setup operations. The decommission coordinator acquires the existing lock,
persists or loads the deletion plan, returns the canonical stored plan, and
holds the lock through provisioning-resource verification, BonesRemote-state
cleanup, and final all-resource verification. Completion replaces mutable
deployment state with an unverified tombstone that retains the plan; final
acknowledgement marks it verified. Disconnect preserves the current protected
state and plan. The setup coordinator holds the same lock while a verified
deleted site is reprovisioned and removes the tombstone and old plan only on
explicit success. Normal site mutations check decommission/tombstone state after
acquiring the lock. Read-only status and diagnostic commands may report the
state but must not repair or reactivate the site.

On opening a coordinator session, BonesRemote returns an unguessable, short-lived
capability bound to the site, operation, coordinator process identity, and held
lock. BonesDeploy passes it only through the typed BonesInfra request. BonesInfra
uses a narrow BonesRemote authorization check before each destructive setup or
delete phase and before verification; the check compares the capability without
acquiring the already-held lock and rejects an expired, mismatched, or dead
coordinator. The capability is never stored in the deletion plan, rendered by
the manifest, logged, or accepted as a command-line argument.

Add `site delete` to BonesInfra as the remote teardown command and keep PyInfra
as the SSH/operation boundary. Separate manifest resolution, safety validation,
inspection, and ordered removal so tests can prove policy without connecting to
a real host. Use the existing etckeeper wrapper to record surviving `/etc`
changes after successful teardown; etckeeper history itself remains shared.

Add `site delete` to the Rust CLI as the orchestration boundary. The Rust handler
owns local readiness, the exact-name confirmation, invoking manifest preflight,
maintaining the privileged coordinator session, invoking BonesInfra deletion,
and signaling verified completion. Rust does not duplicate resource collection
or deletion policy.

## Responsibilities and boundaries

`bonesdeploy/src/cli/args.rs`, `cli/dispatch.rs`, and
`commands/site/delete.rs` own the public command, local config loading, resource
summary, destructive confirmation, the long-lived coordinator session, and
sequencing across BonesRemote and BonesInfra.

`bonesdeploy/src/ui/prompts.rs` owns the exact project-name confirmation using
the existing terminal prompt library. The prompt defaults to no by requiring a
matching non-empty value; `--yes` skips only this interaction.

`bonesinfra.manifest` owns canonical site-resource declarations, resource
identity resolution, destructive safety validation, inspection, and text/JSON
representation. Framework, runtime-service, SSL, and backup modules continue to
declare the resources they own through this boundary.

The BonesInfra site deletion command owns ordered, idempotent remote teardown and
post-deletion inspection. Resource-specific removal stays in the module that
understands the corresponding platform resource, while the delete command owns
ordering and failure propagation.

BonesRemote owns deployment locking, the persisted opaque deletion plan, durable
decommissioning state, deletion tombstones, and the coordinator protocol. All
mutating BonesRemote commands continue to enter through `SiteMutation`; that
guard rejects decommissioning and deleted sites. BonesRemote does not interpret
manifest resources, call BonesInfra, or delete provisioning-owned resources.

`DeployContext`, `DeploymentPaths`, and `bonesdeploy-core::paths` remain the
canonical configuration and path sources. The deletion flow must not add
independent path formulas.

## Affected areas

- `crates/bonesdeploy/src/cli/args.rs` and `cli/dispatch.rs` for the public
  `site delete` command.
- `crates/bonesdeploy/src/commands/site/delete.rs` and `commands/site/mod.rs` for
  orchestration.
- `crates/bonesdeploy/src/commands/site/setup.rs` for tombstone-aware setup and
  post-provisioning reactivation.
- `crates/bonesdeploy/src/ui/prompts.rs` for exact-name confirmation.
- `crates/bonesdeploy/src/infra/ssh.rs` or its existing callers for privileged
  long-lived BonesRemote coordinator sessions.
- `crates/bonesremote/src/cli/`, `commands/`, and
  `release/site_mutation.rs` for the coordinator protocol and mutation
  rejection.
- `crates/bonesremote/src/release/state/` and `bonesdeploy-core::paths` for the
  atomically persisted decommission marker, deletion plan, tombstone, and
  canonical paths.
- `crates/bonesinfra/python/src/bonesinfra/manifest.py` and `project.py` for the
  expanded typed resource inventory and composed custom declarations.
- `crates/bonesinfra/python/src/bonesinfra/cli/app.py` and
  `cli/commands/site/` for manifest preflight and site teardown.
- BonesInfra framework, runtime-service, nginx, AppArmor, systemd, SSL, backup,
  user, and runtime modules for resource declarations and focused removal
  operations.
- Rust and Python integration tests under the owning crates.
- User and architecture documentation describing the destructive command,
  retained local state, and manifest ownership.

## Decisions

- Site deletion is remote-only. Local source, configuration, secrets, Git, GPG,
  and caches are retained because they are not remote site resources and remain
  useful evidence of what was deleted.
- Deletion removes backups and certificates. They are site-owned remote data and
  the requested operation is complete deletion rather than deprovisioning with
  retention.
- The manifest is the single site-resource inventory for inspection and
  deletion. A second teardown registry would drift from setup and make complete
  removal unverifiable.
- Manifest entries describe identity and kind, never arbitrary deletion code.
  Trusted handlers keep destructive behavior reviewable and prevent custom
  manifests from executing root commands during teardown.
- Filesystem preflight is supplemented by descriptor-anchored, no-follow,
  same-filesystem deletion. This closes the path replacement gap between
  validation and recursive removal.
- The complete manifest is safety-validated before the decommission marker or
  any site resource changes. A bad custom declaration therefore fails closed
  without leaving a partially deleted site.
- The first validated deletion plan is persisted and reused by every retry.
  Recomputing it from changed local configuration could omit resources that were
  created by the original deployment strategy.
- The command uses exact project-name confirmation instead of a yes/no prompt
  because deletion includes databases, backups, and shared application data.
  `--yes` remains available for deliberate automation and follows existing CLI
  conventions.
- Decommissioning and the final deletion tombstone are persisted in BonesRemote
  and checked under the existing deployment lock. The privileged coordinator
  session retains that lock for the complete teardown. A short-lived,
  process-local, or BonesInfra-only lock would permit another delete, setup, or
  Git deployment to race the multi-step operation, while removing all markers at
  completion would allow a delayed privileged mutation to recreate state.
- An active mutation causes deletion to fail rather than killing it. Operators
  must let it finish or use the existing release cancellation workflow, avoiding
  implicit termination during a data-destructive command.
- Partial failure leaves the site decommissioning and the command is rerunnable.
  Automatically re-enabling deployment against partially removed resources is
  unsafe.
- A later setup removes the deletion tombstone only after all provisioning
  phases succeed, and only when final deletion verification previously marked
  that tombstone verified. This permits deliberate reuse of the site name
  without exposing a partially deleted or reprovisioned site to deployment.
- Shared packages and daemons remain installed even when the deleted site was
  their last consumer. Package garbage collection is server maintenance, not
  site deletion.

## Risks

- An incomplete manifest would leave a site-owned resource behind. Inventory
  completeness tests must compare representative setup strategies with their
  declared resources. Pre-completion inspection must reject remaining
  provisioning-owned resources, and final inspection must reject any remaining
  declaration after BonesRemote removes its state.
- A changed local configuration could omit resources during a retry. Persisting
  and revalidating the first deletion plan must make retries independent of
  later framework, service, SSL, domain, backup, and custom-manifest changes.
- An unsafe custom path could cause root-level deletion outside the site. The
  complete manifest must pass canonical path-boundary and identity validation
  before mutation, including rejection of roots, traversal, symlink-based parent
  escapes, shared paths, and unknown resource kinds. Handlers must also resist
  symlink replacement and mount crossings during deletion itself.
- Removing users before stopping all of their processes or deleting databases
  before stopping application traffic can produce failures or corruption. The
  orchestrator must enforce the defined phase ordering.
- A network or command failure can leave only part of the site removed. Durable
  decommissioning and idempotent handlers must make rerunning safe and prevent
  deployment while recovery is incomplete.
- Certificate and database commands affect shared engines. Handlers must target
  exact manifest identities and tests must prove that neighboring site and
  global resources are never selected.
- Removing decommission protection can reopen a deployment race. Completion must
  retain a deletion tombstone, and setup must clear it only after provisioning
  has successfully recreated the site.

## Validation

- Python manifest tests cover common, every supported framework/backend,
  configured services, SSL, Quick Tunnel, backup, runtime identity, container or
  image, database, user/group, and deployment-state declarations.
- Python safety tests reject root paths, parent traversal, symlink escapes,
  shared server paths and identities, malformed names, unsupported kinds, and
  unsafe custom declarations before any operation is emitted.
- Python deletion tests use operation inspection or a fake executor to prove
  phase ordering, exact resource targeting, missing-resource idempotency,
  first-failure propagation, post-delete inspection, secret redaction, and
  preservation of shared and neighboring-site resources.
- BonesRemote integration tests prove the coordinator retains the deployment
  lock until success or disconnect, concurrent deletion and setup fail on that
  lock, retries return the original persisted plan, all normal mutations reject
  a marked site, state cleanup creates an unverified tombstone, final
  acknowledgement marks it verified, setup accepts only verified tombstones,
  coordinator capabilities reject unauthorized or dead sessions, and failures
  leave protection intact.
- Rust CLI integration tests prove parsing, exact-name confirmation, cancellation,
  `--yes`, preflight-before-mutation ordering, failure propagation, and that
  local files and Git configuration are untouched.
- Focused tests prove a partially failed deletion can be rerun to completion and
  a final manifest inspection reports no site-owned resources, even after local
  configuration changes between attempts.
- Focused BonesInfra tests prove successful teardown records surviving `/etc`
  changes through etckeeper and failed teardown does not record completion.
- Run `uv run pytest`, `uv run ruff check .`, and `uv run ruff format . --check`
  for BonesInfra.
- Run `cargo test --workspace --exclude e2e`, `cargo clippy`, `cargo fmt`, and
  `shfmt -w .`. Compile the E2E targets without executing the E2E suite locally.
- Review the final diff for undeclared setup resources, duplicated path formulas,
  secret exposure, shell injection, unsafe deletion boundaries, shared-resource
  removal, and documentation drift.
