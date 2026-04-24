# Landlock Runtime Plan

## Goal

Add default per-site runtime isolation without using containers.

The isolation target is the running application process, not the deployment
pipeline. Each project should run as its own service user, and the runtime
process should be confined with Landlock so that a compromise in one site does
not allow that process to read or write other project trees.

This plan intentionally separates build concerns from runtime concerns.

## Security Model

- `deploy_user` remains the user that performs deployments
- `service_user` defaults to the project name
- build happens in a permissive workspace
- runtime runs from a finalized runtime tree
- Landlock is applied only when launching the runtime process
- Landlock policy is anchored to the resolved runtime path, not the `live_root`
  symlink text

This keeps build-time freedom and runtime confinement independent.

## New Layout

For a project named `lawsnipe`:

```text
/srv/deployments/lawsnipe/
├── build/
│   └── workspace/
├── runtime/
│   ├── 20260423_120001/
│   └── 20260424_091500/
├── current -> /srv/deployments/lawsnipe/runtime/20260424_091500
└── shared/
    ├── .env
    └── storage/

/var/www/lawsnipe -> /srv/deployments/lawsnipe/current
```

Definitions:

- `build_root`: dirty build workspace reused across deploys
- `runtime_root`: versioned runtime-ready output trees
- `current`: symlink to the active runtime tree
- `live_root`: stable service/web entrypoint
- `shared`: persistent files and directories that survive runtime switches

## Why This Split Exists

The release-based layout improved atomic deploys, but it still mixed build and
runtime concerns in one tree:

```text
live_root -> deploy_root/current -> active release
```

That creates tension:

- build steps often need toolchains, caches, and dev dependencies
- runtime should be as small and locked down as possible
- Landlock is most useful when the runtime tree is narrow and predictable

The new layout fixes that by making the build output an explicit handoff.

## Runtime Flow

### Deployment Side

1. `bonesremote release stage` creates or prepares:
   - `build_root`
   - `runtime_root/<timestamp>`
   - `shared/`
2. checkout and deployment scripts operate in `build_root`
3. deployment scripts are responsible for producing the final runtime-ready
   contents
4. BonesDeploy copies the resulting build output into the versioned
   `runtime_root/<timestamp>`
5. activation atomically switches `current` to that runtime tree
6. `live_root` remains a symlink to `current`

### Runtime Side

1. `systemd` starts the project service as the per-project `service_user`
2. the unit launches `bonesremote landlock ...`
3. `bonesremote landlock` resolves `live_root` to the active runtime tree
4. `bonesremote landlock` loads runtime policy from `bones.toml`
5. it applies `PR_SET_NO_NEW_PRIVS`
6. it applies Landlock rules
7. it `exec`s the real application command

Landlock is therefore a runtime launcher concern, not a deployment-step
concern.

## bones.toml Changes

`bones.toml` remains the source of truth.

### Existing Fields

- `live_root` stays the stable public path
- `deploy_root` stays the root that contains build, runtime, shared, and
  `current`
- `service_user` should default to `project_name`

### New Data Layout

The deployment root layout becomes:

- `build_root = <deploy_root>/build/workspace`
- `runtime_root = <deploy_root>/runtime`

These can be derived internally and do not need to be user-facing prompts in the
first pass.

### New Runtime Section

Add a runtime section to describe how the site should be started and what it is
allowed to write to.

Suggested shape:

```toml
[runtime]
command = ["/usr/bin/node", "server.js"]
working_dir = "."
writable_paths = ["storage", "bootstrap/cache"]

# Future options
# restart = "always"
# landlock = true
```

Notes:

- `command` is the final runtime command executed by `bonesremote landlock`
- `working_dir` is relative to the resolved runtime tree unless explicitly
  absolute
- `writable_paths` are relative to the resolved runtime tree unless mapped to
  `shared/`
- network access remains unrestricted by default for now

## Command Surface

### bonesdeploy

`bonesdeploy server setup` becomes responsible for runtime isolation setup.

It should:

- provision the per-project service user
- install any required Landlock runtime support
- render or install the project `systemd` service unit
- ensure any required runtime support files exist

### bonesremote

Add a `bonesremote landlock` command namespace for the runtime launcher.

Suggested shape:

```text
bonesremote landlock exec --config <bones.toml path>
```

Responsibilities:

1. load `bones.toml`
2. resolve `live_root` to the active runtime tree
3. build the Landlock ruleset from the resolved runtime tree and runtime config
4. apply `no_new_privs`
5. apply Landlock restrictions
6. `exec` the configured runtime command

This keeps runtime confinement logic in `bonesremote` without turning deploy
hooks into the isolation boundary.

## Ansible Role Changes

Runtime isolation should be provisioned by `bonesdeploy server setup`.

### users role

- default `service_user` to the project name
- create the per-project service user as a system user with no login and no
  home

### common role

- verify that the host kernel supports Landlock
- install `bonesremote` with Landlock support

### new runtime role

Add a dedicated role for service runtime setup.

Responsibilities:

1. create the project service unit
2. point `ExecStart` at `bonesremote landlock exec --config ...`
3. set `User=<service_user>`
4. define working directory and restart policy
5. enable or start the service if desired

Example `ExecStart` shape:

```ini
ExecStart=/usr/local/bin/bonesremote landlock exec --config /var/www/lawsnipe/.bones/bones.toml
```

The final config path may be different, but the service should launch through
the Landlock entrypoint, not the application command directly.

## Landlock Policy Strategy

The policy should be generated from the resolved runtime tree, not the
`live_root` symlink text.

If `live_root` points to:

```text
/var/www/lawsnipe -> /srv/deployments/lawsnipe/current
current -> /srv/deployments/lawsnipe/runtime/20260424_091500
```

then the policy should be built against:

```text
/srv/deployments/lawsnipe/runtime/20260424_091500
```

### Allowed Reads

The first implementation should allow read and execute access to:

- the resolved runtime tree
- any interpreter/runtime paths needed to start the app
- any mandatory system paths required by the runtime

These system paths should be discovered conservatively and documented in code.

### Allowed Writes

Write access should be limited to the paths described by runtime config.

The default source of truth is:

- `runtime.writable_paths`
- existing shared path conventions
- permission hardening expectations already expressed in `permissions.paths`

Landlock write policy should be explicit even if permission hardening already
exists. DAC and Landlock should complement each other, not be treated as the
same thing.

### Network

Do not restrict network access in the first version.

Future work may add opt-in network rules, but the initial rollout should focus
on filesystem isolation.

## Doctor Checks

Extend `bonesremote doctor` with Landlock awareness.

Checks should include:

1. host kernel supports Landlock
2. ABI level is sufficient for the chosen policy
3. runtime command is configured
4. resolved runtime tree exists when the service is expected to run
5. service user exists
6. systemd unit exists if runtime management is enabled

Because support is intentionally limited to Debian and Ubuntu, doctor can be
explicit about that assumption in error messages.

## Deployment Behavior

Deployment and runtime remain separate concerns.

- build runs in `build_root`
- deployment scripts produce the runtime-ready output
- BonesDeploy copies the final output into a versioned `runtime_root`
- activation flips `current`
- Landlock only matters when the service starts

This means build scripts keep their freedom, while runtime gets the tighter
boundary.

## Abstraction Strategy

The user should not need to think about every internal path by default.

First-pass abstraction rules:

- derive `build_root` and `runtime_root` internally from `deploy_root`
- default `service_user` to the project name
- keep runtime config minimal and commented
- let deployment scripts decide what goes into the final runtime tree
- hide Landlock path generation behind `bonesremote landlock`

Advanced users can still override or extend behavior later.

## Implementation Order

1. update the deployment model to split build and runtime trees
2. change `service_user` defaults to `project_name`
3. add runtime config to `bones.toml`
4. add `bonesremote landlock exec`
5. extend `bonesdeploy server setup` to provision the service unit
6. extend doctor to verify Landlock support and runtime readiness
7. update templates and docs to explain the new build/runtime split

## Explicit Non-Goals For First Pass

- network restriction
- non-Debian/non-Ubuntu host support
- separate per-project groups
- sandboxing deployment scripts
- exposing every internal path decision to the user

## Summary

This design keeps the developer ergonomics of build scripts while making runtime
confinement a first-class default.

The key decisions are:

- build in a single dirty `build_root`
- publish into versioned `runtime_root/<timestamp>`
- keep `live_root` as the stable entrypoint
- launch the app through `bonesremote landlock`
- provision the runtime service from `bonesdeploy server setup`
- treat Landlock as runtime-only isolation
