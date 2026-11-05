# Runaway Build Cgroup Kill

<!-- Merged from fix/runaway-build-cgroup-kill -->

## Idea

### Request

Replace the weak process-level attempts to stop runaway native builds with a
guaranteed build-user cgroup kill after a build-script timeout. Pin generated
Next and Nuxt projects to Node `25.9.0` by default because repeated Node 24
builds have exhausted memory during manual deployment testing.

### Problem

BonesRemote currently applies `RuntimeMaxSec` to a transient `systemd-run`
command that invokes `podman exec`. When the command times out, systemd stops
the Podman client unit, but the build payload remains alive in Podman's separate
container scope. BonesRemote then runs `podman rm --force` through the same
resource-starved build-user session. During the confirmed Next incident that
cleanup also timed out, leaving `next build` alive for more than two hours.

The dedicated build-user slice had an effective memory maximum of 773.8 MiB on
a 967 MiB server. The build stayed just below that limit while exhausting
global host memory. SSH and HTTPS stopped responding until the kernel's global
OOM killer killed `next-build`. The existing timeout therefore reports failure
without guaranteeing that the resource consumer stopped.

Generated framework configuration also assigns the same Node 24 default to
every framework. Next and Nuxt consequently reproduce a Node version that has
repeatedly exhibited unacceptable memory behavior in these deployments.

### Definitions

**Build-user cgroup:** The cgroup-v2 `user-<uid>.slice` belonging to one site's
dedicated `<site>-build` identity. It contains that identity's systemd user
manager, rootless Podman container payloads, and supporting processes. It does
not contain the site's runtime services, which use the separate `<site>` UID.

**Cgroup kill:** A root-authorized write to the build-user cgroup's
`cgroup.kill` control file followed by verification that the cgroup is absent
or reports `populated 0`. This is the kernel-enforced containment operation;
signalling an individual PID or asking Podman to remove a container is not a
cgroup kill.

**Build timeout:** A build script reaching its configured
`build.timeout_seconds` deadline and receiving systemd's `Result=timeout`.
Ordinary nonzero script exits are build failures, not timeouts.

**Framework Node default:** The exact Node version written into a newly
generated framework's runtime configuration and committed `.env.build`.
Explicit project configuration continues to override this generated default.

### Desired outcome

When a native build script times out, BonesRemote kills the complete dedicated
build-user cgroup and verifies that it no longer contains processes before
returning the timeout failure. No Node worker, Podman helper, or build-user
manager from that build can continue consuming host resources. Cancellation of
an interrupted build uses the same fail-closed boundary.

New Next and Nuxt projects consistently pin Node `25.9.0` for both build and
runtime behavior. Other framework defaults and existing explicit pins remain
unchanged.

### Scope

- Give timed build-script units stable identities long enough for BonesRemote
  to distinguish systemd timeout results from ordinary command failures.
- Kill and verify the complete dedicated build-user cgroup after a timeout.
- Prevent timeout unwinding from falling back to Podman cleanup in the killed
  session.
- Use the same cgroup termination operation for release cancellation while the
  deployment is still in a cancellable build phase.
- Remove process/container cleanup behavior made obsolete by the cgroup kill.
- Generate Next and Nuxt runtime and `.env.build` defaults with Node `25.9.0`.
- Add focused regression tests and update security/context documentation.

### Constraints

- Cgroup termination is executed by root and must not depend on Podman, the
  build user's systemd manager, or cooperation from the runaway process.
- The killed boundary is the dedicated build-user slice; runtime application
  services must remain running.
- A timeout is not considered contained until the cgroup is absent or reports
  `populated 0`.
- Provisioning must reject native build hosts that do not expose the required
  cgroup-v2 `cgroup.kill` control for the dedicated build-user slice.
- Containment verification is bounded; failure to prove an empty cgroup within
  the deadline is a hard error and must not permit release-state cleanup.
- Ordinary build failures retain normal container cleanup.
- A later deployment may restart the build-user manager and remove stale
  Podman metadata before creating a new container.
- Node versions remain exact `X.Y.Z` pins. Explicit user-selected versions are
  not rewritten.
- Do not run the repository end-to-end test suite.

### Exclusions

- Moving builds off the production server or implementing local artifacts.
- Changing the existing build-user memory, CPU, swap, task, disk, or I/O
  resource limits.
- Changing runtime-service cgroups or terminating runtime application users.
- Automatically rewriting existing projects from Node 24 to Node 25.
- Changing Node defaults for Django, Laravel, Rails, SvelteKit, Vue, or custom
  projects.
- Proving that Node 25 uses less memory for every Next or Nuxt workload.


## Plan

### Current behavior

`crates/bonesremote/src/release/lifecycle/build/run_scripts.rs` prepares one
dedicated build user and `BuildContainer`, then executes each numbered build
script through `BuildContainer::run_script`.

`crates/bonesremote/src/release/lifecycle/build/container.rs` invokes
`podman exec` through `build_script_command`. On ordinary completion it removes
the container with `podman rm --force --time 0`. Its `Drop` implementation
repeats that removal as best-effort cleanup whenever normal removal has not
completed.

`crates/bonesremote/src/release/lifecycle/build/build_user.rs` constructs
`systemd-run --machine=<build-user>@ --user --pipe --wait`. Timed commands add
`RuntimeMaxSec`, while control commands receive a separate 20-second
`RuntimeMaxSec`. The transient script unit has no caller-selected name and is
collected automatically, so BonesRemote receives a failed exit status but does
not inspect the unit's `Result` to distinguish a timeout.

The timed `podman exec` client and its payload do not share the same systemd
unit. Rootless Podman places the payload in a `libpod-*.scope` below the
dedicated build-user slice. Stopping the transient client unit therefore does
not stop the payload. Container removal remains a userspace request through the
same build-user manager and can itself time out under memory pressure.

`crates/bonesremote/src/commands/release/kill.rs` cancels a build by signalling
the recorded root BonesRemote PID with TERM and then KILL. After taking the
site lock, it starts or verifies the build-user manager and requests Podman
container removal. Neither operation guarantees termination of all build-user
processes.

Framework scaffolding obtains runtime Node defaults from
`Framework::runtime_defaults` in `crates/bonesdeploy/src/frameworks.rs`, which
currently uses the core-wide `24.19.0` default for every framework. Next and
Nuxt `.env.build` templates declare an empty `NODE_VERSION`, so their generated
build input does not pin the requested framework-specific version.

### Intended behavior

Each timed build script runs under a named transient user unit whose result
remains inspectable after `systemd-run --wait` returns. BonesRemote reads the
unit result. An ordinary exit preserves the current success/failure behavior
and normal Podman cleanup. `Result=timeout` causes the root process to write to
the complete build-user cgroup's `cgroup.kill`, stop the associated user
manager, and verify that the cgroup is absent or unpopulated before returning a
specific timeout error.

Once the build-user cgroup has been killed, `BuildContainer` does not issue
further commands through that dead session. Stale Podman metadata is harmless
and is removed by the existing pre-build reconciliation after a later deploy
restarts and verifies the build-user manager.

Release cancellation in a cancellable build phase kills and verifies the same
build-user cgroup before stopping the root coordinator and cleaning staged
filesystem state. It does not restart the build-user manager merely to ask
Podman to stop processes that the kernel has already terminated.

Native build-user provisioning verifies that the active build-user slice
exposes `cgroup.kill`. A host without that cgroup-v2 containment primitive
cannot pass site setup and cannot claim guaranteed timeout containment.

New Next and Nuxt projects receive Node `25.9.0` in both runtime defaults and
their committed `.env.build`. Other frameworks continue receiving the core
Node `24.19.0` default and their existing build template behavior.

### Approach

Add build-user cgroup identity and termination behavior to `build_user.rs`,
where build UID resolution and systemd user-manager control already live. The
operation derives `/sys/fs/cgroup/user.slice/user-<uid>.slice`, writes `1` to
`cgroup.kill`, stops `user@<uid>.service`, and verifies `cgroup.events` until
the slice is absent or reports `populated 0`. Failure to prove termination is
returned as a containment error rather than hidden behind the original build
failure.

Give timed script transient units a site-specific BonesDeploy unit name. Timed
commands omit `--collect`, allowing failed units to remain inspectable after
`systemd-run --wait` returns. Before each script, reset an inactive stale unit;
after completion, query `Result` and reset the unit on every non-containment
path. Represent script completion as a result that distinguishes normal process
exit from timeout. On timeout, invoke build-user cgroup termination and mark the
`BuildContainer` session unavailable so `Drop` does not run `podman rm` through
a manager that was just killed. Per-site deployment locking and sequential
script execution prevent concurrent use of the site-specific unit name.

Keep ordinary successful and non-timeout failure cleanup unchanged. Remove the
claim that `RuntimeMaxSec` kills the build process tree; it supplies the
deadline and timeout result, while `cgroup.kill` supplies containment. Remove
release cancellation's build-user readiness and Podman-removal steps and use
the common cgroup termination operation before coordinator signalling.

Implement cgroup control as a small object with an explicit cgroup root, UID,
and verification deadline. Production constructs it from `/sys/fs/cgroup`, the
resolved build UID, and a fixed five-second deadline. Tests construct it below a
temporary root with a zero-duration deadline, allowing success, missing-control,
and still-populated behavior to be exercised without privileged host mutation.
After writing `cgroup.kill`, request `systemctl stop --no-block user@<uid>.service`
and poll `cgroup.events` every 100 milliseconds. Absence or `populated 0` is
success; timeout is a containment failure.

Add a framework-specific Node default in the framework layer. Next and Nuxt
select `25.9.0`; all other frameworks delegate to the existing core default.
Pass the selected `Runtime` into the Next and Nuxt build-environment renderers
and substitute `runtime.node_version` into their templates. This keeps one
selected value for both generated runtime and build configuration and preserves
an explicit non-default runtime version supplied before scaffolding.

Extend BonesInfra's root provisioning verification for each native build user
to require the active `user-<uid>.slice/cgroup.kill` control file. This check
runs after the build-user manager starts, when its slice is guaranteed to
exist.

### Responsibilities and boundaries

`bonesremote::release::lifecycle::build::build_user` owns build-user identity,
named transient-unit inspection, and kernel cgroup termination. It does not
know release state or framework behavior.

`BuildContainer` owns the relationship between one script execution and its
container session. It initiates containment on timeout and suppresses obsolete
Podman cleanup after that session has been killed.

`run_scripts` remains the sequential native-build coordinator and reports the
specific script failure. It does not implement process signalling or cgroup
paths.

`commands::release::kill` remains the root cancellation coordinator. It checks
whether cancellation is allowed, terminates the untrusted build-user boundary,
stops the root deployment coordinator, acquires the site lock, and cleans
release filesystem state.

If cgroup termination or verification fails, cancellation stops immediately:
the coordinator is not signalled and release filesystem/state cleanup does not
run. This preserves ownership of the active operation rather than declaring a
runaway build cancelled without proof.

`commands::deploy::coordinator` distinguishes containment failure from an
ordinary build error. It preserves the active deployment record, build context,
and staged release when containment cannot be proven, while successfully
contained timeouts follow normal failed-build cleanup.

`bonesdeploy::frameworks` owns framework-specific generated defaults. The core
configuration default remains the fallback for frameworks without a specific
Node policy.

### Affected areas

- `crates/bonesremote/src/release/lifecycle/build/build_user.rs`
- `crates/bonesremote/src/release/lifecycle/build/container.rs`
- `crates/bonesremote/src/release/lifecycle/build/run_scripts.rs`
- `crates/bonesremote/src/release/lifecycle/build/mod.rs`
- `crates/bonesremote/src/commands/release/kill.rs`
- `crates/bonesremote/src/commands/deploy/coordinator.rs`
- `crates/bonesremote/tests/release/lifecycle_build_user.rs`
- `crates/bonesremote/tests/release/lifecycle_build_container.rs`
- `crates/bonesremote/tests/commands/release_kill.rs`
- `crates/bonesdeploy/src/frameworks.rs`
- `crates/bonesdeploy/src/frameworks/next.rs`
- `crates/bonesdeploy/src/frameworks/nuxt.rs`
- `crates/bonesdeploy/assets/frameworks/next/next.env.build.example`
- `crates/bonesdeploy/assets/frameworks/nuxt/nuxt.env.build.example`
- `crates/bonesdeploy/tests/config_frameworks.rs`
- `crates/bonesdeploy/tests/assets.rs`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/site/users.py`
- `crates/bonesinfra/python/tests/test_build_user.py`
- `crates/bonesremote/tests/release/lifecycle_build_scripts.rs`
- `tests/ASSERTIONS.md`
- `docs/security/invariants.md`
- `CONTEXT.md`

### Decisions

- Kill the complete dedicated build-user slice rather than only the container
  scope. The incident left payload and host-side Podman helpers in separate
  descendant units; the user slice is the smallest existing boundary that
  contains all of them without affecting runtime services.
- Use cgroup v2's `cgroup.kill` control file rather than PID enumeration,
  signals, or Podman cleanup. The kernel operation is fork-safe and does not
  depend on userspace cooperation from the exhausted session.
- Verify `cgroup.events` after the kill. Returning before `populated 0` would
  recreate the current false claim that failure means containment.
- Retain systemd `RuntimeMaxSec` only as the deadline and authoritative
  `Result=timeout` source. It is not treated as the kill mechanism.
- Remove `--collect` only from named timed script units. Untimed and control
  commands keep their current automatically collected transient behavior.
- Skip Podman cleanup after a cgroup kill. Restarting a dead build session during
  failure handling expands the failure surface; the next deployment already
  reconciles an existing named container before building.
- Cancellation kills the build-user cgroup before signalling the root
  coordinator. This removes the untrusted resource consumer first while
  preserving the coordinator's ownership of locks and state until cancellation
  takes over.
- Pin Node `25.9.0`, the latest published Node 25 release, only for newly
  generated Next and Nuxt projects. Node 25 is not an LTS line, but this explicit
  policy follows the requested operational workaround without silently changing
  unrelated frameworks or existing projects.
- Preserve deployment readiness before a normal build. The existing deploy
  lifecycle restarts and verifies the build-user manager, then pre-build
  container reconciliation removes stale metadata left by a prior cgroup kill.

### Risks

- Killing the whole build-user slice discards every build process for that site,
  not only the currently named container. The build identity is dedicated and
  deployments are serialized per site, so this is intended containment.
- A stale Podman container record remains after timeout until a later build
  starts the user manager and runs existing pre-build removal. Readiness and
  removal must continue producing actionable errors if Podman cannot reconcile
  it.
- Named transient units can collide or accumulate if their identity and cleanup
  are not deterministic. Names must be unique within the serialized build and
  normal completion must reset/remove them.
- A failed cgroup verification deliberately leaves release cancellation
  incomplete. Operators receive a containment error and must use console-level
  recovery rather than having BonesRemote mutate state while processes may
  still be alive.
- Killing the user manager can cause the waiting `systemd-run` transport to
  fail noisily. Timeout reporting must preserve the original script name and
  state that containment succeeded.
- Direct cgroup paths are Linux cgroup-v2-specific. Supported production hosts
  already use systemd and cgroup v2; doctor/provisioning assumptions must remain
  coherent with that requirement.
- Node `25.9.0` is end-of-life and non-LTS. The exact pin is reproducible but
  receives no ongoing security maintenance, so it must remain a deliberate
  Next/Nuxt-only default rather than a global default.
- A mismatch between runtime and `.env.build` Node versions can produce native
  module incompatibility. Tests must assert both generated values together.

### Validation

- Focused BonesRemote tests prove timed script commands have inspectable named
  units, normal failures remain distinct from `Result=timeout`, and timeout
  handling targets the expected `user-<uid>.slice` cgroup.
- Temporary-root cgroup tests prove a writable kill control plus `populated 0`
  succeeds, a missing `cgroup.kill` fails closed, and a still-populated cgroup
  fails at the bounded verification deadline.
- Focused BonesRemote tests prove a contained timeout suppresses subsequent
  Podman removal and release cancellation uses build-user cgroup termination
  instead of build-user readiness plus container removal.
- Focused coordinator tests prove containment failure preserves release state
  while ordinary build failure retains existing abort cleanup.
- Focused BonesDeploy tests prove Next and Nuxt runtime defaults and generated
  `.env.build` both contain `25.9.0`, while another framework continues using
  the core `24.19.0` runtime default.
- Existing explicit version parsing and override tests continue to pass.
- Focused BonesInfra tests prove build-user provisioning checks for the
  site-specific `cgroup.kill` control after starting the user manager.
- `uv run pytest`, `ruff check .`, and `ruff format .` pass in
  `crates/bonesinfra/python`.
- `cargo test --workspace --exclude e2e` passes; the E2E suite is not run.
- `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`,
  `cargo fmt --all -- --check`, `shfmt -w .`, and `git diff --check` complete
  without errors or warnings.
- Final diff review confirms no PID enumeration or Podman removal remains as the
  claimed timeout/cancellation containment mechanism and documentation no
  longer claims `RuntimeMaxSec` alone kills the build process tree.


## Tasks

### Implementation

- [x] Add named transient build-script units and inspectable systemd results so
      BonesRemote distinguishes `Result=timeout` from ordinary script exits.
- [x] Add root-owned build-user cgroup termination that writes `cgroup.kill`,
      requests nonblocking user-manager stop, and proves within a bounded
      deadline that the slice is absent or unpopulated.
- [x] Make `BuildContainer` kill the build-user cgroup on timeout and suppress
      Podman cleanup through the terminated session while preserving ordinary
      success and failure cleanup.
- [x] Replace release cancellation's build-user readiness and container-removal
      sequence with build-user cgroup termination before coordinator shutdown.
- [x] Preserve active deployment state and build artifacts when cgroup
      termination cannot be proven, without changing ordinary abort cleanup.
- [x] Add Next/Nuxt-specific Node `25.9.0` runtime defaults and render the same
      selected runtime pin into their generated `.env.build` files without
      changing other framework defaults or explicit overrides.
- [x] Make native build-user provisioning fail when the active dedicated slice
      does not expose cgroup-v2 `cgroup.kill`.
- [x] Update security invariants, context, and generated assertion inventory to
      describe cgroup containment and framework-specific Node defaults.

### Validation

- [x] Run focused BonesRemote tests proving named timeout detection, cgroup kill
      targeting, bounded success/failure verification, timeout cleanup
      suppression, fail-closed cancellation behavior, and coordinator state
      preservation after containment failure.
- [x] Run focused BonesDeploy tests proving Next/Nuxt build/runtime pins are
      `25.9.0`, an explicit selected version remains aligned, and
      non-Next/Nuxt runtime defaults remain `24.19.0`.
- [x] Run focused BonesInfra tests proving provisioning requires the build-user
      slice's cgroup-v2 kill control.
- [x] Run `cargo test --workspace --exclude e2e` and resolve every non-E2E test
      failure without running the E2E suite.
- [x] Run `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`,
      `cargo fmt --all -- --check`, `shfmt -w .`, and `git diff --check` with no
      warnings or errors.
- [x] Run `uv run pytest`, `ruff check .`, and `ruff format .` from
      `crates/bonesinfra/python` with no failures or formatting drift.

### Completion

- [x] Review the final diff for stale weak-kill claims, obsolete helpers,
      accidental framework default changes, and unrelated edits.
- [x] Record validation results, deviations, discoveries, and deliberately
      unfinished work in the completion notes.

### Completion notes

Implementation completed on 2026-09-29.

- Timed script units use a deterministic execution ordinal rather than the
  script filename so every accepted numbered filename remains compatible with
  systemd unit-name syntax.
- Containment errors are typed so the deployment coordinator preserves active
  state and build artifacts for operator recovery. This required the
  clarification recorded in `04-containment-error-clarity.md`.
- The committed BonesInfra wheel was regenerated after adding the provisioning
  check.
- Focused Rust tests passed for named-unit construction, timeout-result
  classification, cgroup targeting, missing controls, successful empty-cgroup
  verification, populated-cgroup failure, and typed containment errors.
- Focused framework tests passed for Next/Nuxt `25.9.0` defaults, explicit
  selected-version rendering, and the unchanged `24.19.0` fallback.
- All 510 BonesInfra Python tests passed, along with Ruff checks and formatting.
- `cargo test --workspace --exclude e2e`, strict workspace Clippy, Rust
  formatting, `shfmt -w .`, and `git diff --check` passed.
- The E2E suite and privileged mutation of a live systemd/cgroup hierarchy were
  deliberately not run, as required by the plan.


## Containment Error Clarification

### Trigger

Implementation review found that the deployment coordinator currently aborts
and cleans release state for every build error. The approved constraint requires
containment-verification failure to preserve active release state instead.

### Decision

Build-user termination failures use a distinct containment error. The deployment
coordinator returns that error without deleting the build context, failed
release, or active deployment record. Ordinary build failures, including a
successfully contained script timeout, keep the existing abort cleanup.

### Supersedes

This adds the error-propagation detail needed to enforce the existing
fail-closed constraint. It does not replace an earlier decision.

### Required Authoritative Updates

- `02-plan.md` assigns containment-error handling to the deployment coordinator
  and includes its source file in the affected areas.
- `03-tasks.md` includes the coordinator behavior and its regression validation.
