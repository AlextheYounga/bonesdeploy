# Tasks

## Implementation

- [ ] Extend the typed BonesInfra manifest into one resource inventory covering
  filesystem paths, systemd services and memberships, AppArmor profiles, Unix
  users/groups, databases/accounts, key-value data, certificates, backups,
  deployment state, tunnels, and framework runtime containers/images for every
  supported deployment strategy.
- [ ] Update common, framework, runtime-service, SSL, backup, and custom manifest
  composition so every resource created by site setup, services, runtime, SSL,
  backup, and deployment is declared once through canonical context/path
  derivation.
- [ ] Add pre-mutation manifest validation that rejects unsupported resource
  kinds, non-canonical or escaping paths, symlink parent escapes, shared paths or
  identities, malformed project-derived names, and unsafe custom declarations.
- [ ] Implement descriptor-anchored, no-follow, same-filesystem path removal that
  rejects path-kind changes, symlink replacement, and mount crossings during
  deletion rather than relying only on preflight validation.
- [ ] Resolve validated resources into a stable, secret-free deletion plan and
  make retries consume the first persisted plan instead of recomputing ownership
  from current local configuration.
- [ ] Expand `site manifest` text and JSON inspection to report every resource
  kind and state without exposing secrets, file contents, or deletion commands.
- [ ] Add canonical BonesRemote decommission state, persisted opaque deletion
  plans, unverified/verified deletion tombstones, and a framed coordinator
  protocol that retains the existing deployment lock until verified success or
  disconnect.
- [ ] Add short-lived coordinator capabilities bound to site, operation, process
  identity, and lock ownership; pass them through typed requests and revalidate
  them before each BonesInfra mutation or verification phase without logging or
  persisting the token.
- [ ] Make every normal BonesRemote site mutation reject a decommissioning site,
  while keeping deletion retries and read-only reporting safe.
- [ ] Implement trusted, idempotent BonesInfra removal handlers for each manifest
  resource kind without uninstalling or reconfiguring shared server resources.
- [ ] Implement ordered BonesInfra site teardown that removes ingress and
  schedules, stops services, removes runtime/service data and identities,
  removes certificates/backups/configuration/users/files, reloads shared
  managers, verifies provisioning-owned resources before completion, and
  verifies the complete persisted plan after BonesRemote removes its state.
- [ ] Add the public `bonesdeploy site delete` command, exact project-name prompt,
  `--yes` option, manifest summary, and orchestration of preflight,
  decommissioning, teardown, verification, and final state cleanup.
- [ ] Make `site setup` preserve deletion protection throughout provisioning and
  hold the same site lock, then reactivate a previously deleted site only after
  every setup phase succeeds.
- [ ] Make other BonesInfra site mutation entry points reject decommissioning and
  deleted sites unless invoked through the authorized delete or setup
  coordinator flow.
- [ ] Ensure failure before state cleanup preserves decommissioning state,
  failure after state cleanup preserves an unverified tombstone, both retain the
  deletion plan and resource context, and the same command resumes safely from
  either phase.

## Validation

- [ ] Add manifest completeness tests for common resources and every supported
  framework/backend, service, SSL/tunnel, and backup strategy.
- [ ] Add manifest safety tests for traversal, filesystem roots, symlink escapes,
  shared paths and identities, malformed names, unknown kinds, and custom
  declarations, asserting failure occurs before mutation and replacement races
  cannot escape trusted roots during deletion.
- [ ] Add BonesInfra deletion tests for phase ordering, exact resource targeting,
  missing-resource idempotency, partial failure, rerun completion, post-delete
  inspection, persisted-plan reuse after local configuration changes, secret
  redaction, etckeeper completion commits, and preservation of shared and
  neighboring-site resources.
- [ ] Add BonesRemote tests for coordinator lock lifetime, concurrent deletion
  and setup rejection, persisted-plan reuse, disconnect behavior, mutation
  rejection, unverified-to-verified completion, capability authorization and
  expiry, setup reactivation, protection preservation before state cleanup, and
  retry after disconnect between tombstone creation and final verification.
- [ ] Add Rust CLI integration tests for command parsing, exact-name confirmation,
  cancellation, `--yes`, orchestration ordering, failure propagation, and
  preservation of all local project and Git state.
- [ ] Run focused Rust and Python tests, then `cargo test --workspace --exclude
  e2e` and `uv run pytest`; do not execute E2E tests locally.
- [ ] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `uv run ruff check .`, and
  `uv run ruff format . --check`; address all warnings and errors.

## Completion

- [ ] Update `README.md`, `CONTEXT.md`, BonesInfra context, CLI help, and
  architecture documentation for destructive confirmation, retained local and
  shared state, expanded manifest ownership, rerun behavior, and deployment
  blocking during decommissioning.
- [ ] Review the final diff against every site setup and deployment resource,
  verifying no duplicate inventory, undeclared resource, secret exposure,
  unsafe path or identity, shared-resource deletion, or premature decommission
  completion remains.

## Completion notes

No implementation has started. The plan is awaiting human review and approval.
