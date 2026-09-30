# Plan

## Current behavior

`bonesdeploy site setup --yes` provisions the site, applies its runtime, then
runs remote site doctor. Placeholder provisioning in
`bonesinfra/.../site/placeholder.py` creates the canonical placeholder release
and targets it through `current`. The Compose systemd unit intentionally has a
`ConditionPathExists={{ paths.current }}/compose.yaml`, so it is deferred until
the first deployment.

`bonesremote/src/commands/doctor/site.rs` calls `check_docker_runtime` for
Docker-backed sites. It reports a first Compose deployment as pending only when
`current` does not exist. Because the placeholder directory exists,
`validate_active_configuration` runs and `ComposeFiles::discover` rejects the
missing Compose file. `commands/doctor/services.rs` already exposes
`current_is_placeholder` for its condition-deferred service handling.

## Intended behavior

When the current release is the placeholder, Compose doctor will report that
the first Compose deployment is pending and will not validate an active Compose
configuration or inspect its containers. Non-placeholder current releases will
retain all existing Compose file, registration, stack, ingress, and health
checks.

## Approach

Use `services::current_is_placeholder` in `check_docker_runtime` together with
the existing missing-current branch. Return the existing pending outcome before
calling `validate_active_configuration`. Add a focused integration test that
creates a canonical placeholder current link and verifies the Compose doctor
path emits pending rather than an invalid active configuration issue.

## Responsibilities and boundaries

`bonesremote/src/commands/doctor/site.rs` owns Compose-specific doctor
classification. `commands/doctor/services.rs` remains the canonical owner of
the placeholder-release predicate. `runtime/docker/command.rs` remains strict:
it validates only actual active Compose releases and does not interpret
placeholder lifecycle state.

## Affected areas

- `crates/bonesremote/src/commands/doctor/site.rs`: classify a placeholder
  current release as a pending first Compose deployment.
- `crates/bonesremote/tests/commands/doctor_site.rs`: cover the placeholder
  Compose-doctor regression through the public doctor boundary.
- `docs/plans/fix/compose-doctor-placeholder/`: record the defect, decision,
  tasks, and validation results.

## Decisions

- Reuse `current_is_placeholder` because it already represents the canonical
  release-state distinction used by doctor service checks.
- Return before Compose discovery rather than making discovery accept an empty
  release. A deployed Compose release without a Compose file remains invalid.
- Use a focused doctor test plus the ignored Compose E2E scenario; the former
  guards the release-state classification and the latter proves site setup can
  progress in a real nested-Docker environment.

## Risks

- Applying the placeholder exception to all current releases would conceal a
  broken deployed Compose release; the canonical predicate prevents that.
- Returning after only the Compose checks preserves general site checks, but
  could leave a separate Docker installation issue reported during initial
  setup; that remains intentional because Docker is required before deployment.
- The E2E scenario can expose a later Compose build, deployment, or rollback
  defect after site setup progresses.

## Validation

- Run the focused BonesRemote doctor regression test; a placeholder current
  release must produce a pending first-deployment result and no active Compose
  configuration issue.
- Run `cargo test -p bonesremote` and preserve strict Compose-discovery tests.
- Run `cargo test -p e2e --test setup -- docker_compose --ignored
  --test-threads=1 --nocapture`; initial site setup must pass and the full
  Compose lifecycle scenario must complete.
- Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`, then inspect the final
  diff.
