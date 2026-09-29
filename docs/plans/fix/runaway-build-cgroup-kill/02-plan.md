# Plan

## Current behavior

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

## Intended behavior

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

## Approach

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

## Responsibilities and boundaries

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

`bonesdeploy::frameworks` owns framework-specific generated defaults. The core
configuration default remains the fallback for frameworks without a specific
Node policy.

## Affected areas

- `crates/bonesremote/src/release/lifecycle/build/build_user.rs`
- `crates/bonesremote/src/release/lifecycle/build/container.rs`
- `crates/bonesremote/src/release/lifecycle/build/run_scripts.rs`
- `crates/bonesremote/src/release/lifecycle/build/mod.rs`
- `crates/bonesremote/src/commands/release/kill.rs`
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

## Decisions

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

## Risks

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

## Validation

- Focused BonesRemote tests prove timed script commands have inspectable named
  units, normal failures remain distinct from `Result=timeout`, and timeout
  handling targets the expected `user-<uid>.slice` cgroup.
- Temporary-root cgroup tests prove a writable kill control plus `populated 0`
  succeeds, a missing `cgroup.kill` fails closed, and a still-populated cgroup
  fails at the bounded verification deadline.
- Focused BonesRemote tests prove a contained timeout suppresses subsequent
  Podman removal and release cancellation uses build-user cgroup termination
  instead of build-user readiness plus container removal.
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
