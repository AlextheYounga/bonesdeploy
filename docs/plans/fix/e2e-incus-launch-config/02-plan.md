# Plan

## Current behavior

`e2e/src/container.rs` owns Incus container lifecycle behavior.
`Container::launch` supplies three valid configuration pairs:
`limits.memory=2GiB`, `limits.cpu=1`, and `security.nesting=true`. The argument
array ends with a fourth, standalone `--config` option. The `incus` wrapper in
`e2e/src/incus.rs` runs the array unchanged and returns Incus's non-zero exit
status as an error.

`e2e/src/image.rs` calls `Container::launch` to build the cached base image,
and `e2e/tests/setup/harness.rs` calls it again for the shared test container.
Consequently all ten ignored framework scenarios fail at the same boundary
before provisioning or deployment.

## Intended behavior

`Container::launch` will pass exactly the three complete launch configuration
pairs. Incus will accept the launch invocation and both the base-image and
shared-harness paths can proceed.

## Approach

Remove the trailing standalone `--config` from the static argument array in
`Container::launch`. This keeps the existing Incus CLI wrapper and all resource
settings intact while eliminating the invalid option at its single source.

## Responsibilities and boundaries

`e2e/src/container.rs` remains the owner of container launch arguments and
lifecycle cleanup. `e2e/src/incus.rs` remains a thin CLI execution boundary;
scenario modules remain unchanged because they consume the shared harness after
container creation succeeds.

## Affected areas

- `e2e/src/container.rs`: remove the invalid incomplete launch option.
- `docs/plans/fix/e2e-incus-launch-config/`: retain the settled bugfix record
  and validation evidence.

## Decisions

- Remove the incomplete option rather than add a fourth configuration value:
  the three preceding pairs cover the documented resource and nesting needs,
  and the standalone option has no associated setting.
- Use E2E execution as the regression boundary. The invalid command is an
  external CLI invocation constructed inside the lifecycle owner, and success
  requires a real reachable Incus daemon.

## Risks

- A typo while editing the static array could remove or alter a required
  resource setting, preventing nested Docker or constraining E2E containers.
- The corrected launch can expose later framework or host prerequisite failures
  that were previously masked by this common failure.

## Validation

- Run one ignored setup scenario against the active Incus daemon; it must create
  a container rather than report an incomplete `--config` argument.
- Run `cargo test-e2e`; all ten framework scenarios must pass serially.
- Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` as required repository
  checks, then inspect the final diff for an argument-only code change plus its
  durable planning record.
