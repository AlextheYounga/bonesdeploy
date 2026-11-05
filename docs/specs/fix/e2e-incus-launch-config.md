# E2E Incus Launch Config

<!-- Merged from fix/e2e-incus-launch-config -->

## Idea

### Request

Correct the E2E failure encountered while running all framework scenarios, and
record the error and its repair path in durable planning documentation.

### Problem

Every ignored E2E scenario fails before framework setup because
`Container::launch` passes a trailing `--config` argument to `incus launch`
without the required configuration value. Incus rejects the command with
`Error: flag needs an argument: --config`, so neither the cached base image nor
the shared test container can be created.

### Definitions

**Container launch configuration:** The complete sequence of `--config` option
and `key=value` argument pairs supplied by `Container::launch` to the Incus
CLI. It includes the memory, CPU, and nesting settings required by E2E
containers; it excludes a standalone option with no value.

### Desired outcome

`Container::launch` invokes `incus launch` with only complete configuration
pairs. With a reachable, initialized Incus daemon, the E2E harness can create
its base image and shared container, allowing every framework scenario to run.

### Scope

- Correct the Incus argument list owned by `e2e/src/container.rs`.
- Validate the corrected command through an E2E scenario and the complete
  serialized E2E framework matrix.

### Constraints

- Create the repair branch through Git Flow with a linked worktree.
- Preserve the existing memory, CPU, and nesting configuration values.
- Keep the E2E suite serialized because scenarios share the Incus daemon.
- Do not modify application framework fixtures or deployment behavior.

### Exclusions

- Starting, initializing, or granting access to the local Incus daemon is host
  setup, not an application-code fix.
- Changing Incus resource limits or nesting behavior is out of scope.
- Repairing failures that occur after container launch is out of scope for this
  bugfix and requires separate investigation.


## Plan

### Current behavior

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

### Intended behavior

`Container::launch` will pass exactly the three complete launch configuration
pairs. Incus will accept the launch invocation and both the base-image and
shared-harness paths can proceed.

### Approach

Remove the trailing standalone `--config` from the static argument array in
`Container::launch`. This keeps the existing Incus CLI wrapper and all resource
settings intact while eliminating the invalid option at its single source.

### Responsibilities and boundaries

`e2e/src/container.rs` remains the owner of container launch arguments and
lifecycle cleanup. `e2e/src/incus.rs` remains a thin CLI execution boundary;
scenario modules remain unchanged because they consume the shared harness after
container creation succeeds.

### Affected areas

- `e2e/src/container.rs`: remove the invalid incomplete launch option.
- `docs/plans/fix/e2e-incus-launch-config/`: retain the settled bugfix record
  and validation evidence.

### Decisions

- Remove the incomplete option rather than add a fourth configuration value:
  the three preceding pairs cover the documented resource and nesting needs,
  and the standalone option has no associated setting.
- Use E2E execution as the regression boundary. The invalid command is an
  external CLI invocation constructed inside the lifecycle owner, and success
  requires a real reachable Incus daemon.

### Risks

- A typo while editing the static array could remove or alter a required
  resource setting, preventing nested Docker or constraining E2E containers.
- The corrected launch can expose later framework or host prerequisite failures
  that were previously masked by this common failure.

### Validation

- Run one ignored setup scenario against the active Incus daemon; it must create
  a container rather than report an incomplete `--config` argument.
- Run `cargo test-e2e`; all ten framework scenarios must pass serially.
- Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` as required repository
  checks, then inspect the final diff for an argument-only code change plus its
  durable planning record.


## Tasks

### Implementation

- [ ] Remove the trailing standalone `--config` from `Container::launch` while
  preserving its memory, CPU, and nesting configuration pairs.

### Validation

- [ ] Run an ignored E2E setup scenario with a reachable Incus daemon; verify
  container launch progresses beyond Incus argument parsing.
- [ ] Run `cargo test-e2e`; verify Django, Compose, Laravel, Next server/static,
  Nuxt server/static, Rails, SvelteKit, and Vue pass serially.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without warnings or
  formatting changes that require further correction.

### Completion

- [ ] Review the final diff to confirm the static launch argument repair and
  durable plan are the only changes.

### Completion notes

The full E2E matrix was first run on 2026-09-29. It initially stopped because
the local Incus daemon was inactive; after the host daemon was started, all ten
framework scenarios reached `Container::launch` and failed on the same invalid
trailing `--config` option. No implementation change has been made pending
human review and approval of this plan.
