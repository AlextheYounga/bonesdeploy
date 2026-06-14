# BonesDeploy Architecture Rehaul Plan

## Goal

Rework BonesDeploy around a stable deploy/runtime boundary:

```text
deploy user owns releases and activation
runtime user runs the app and owns mutable runtime state
runtime reads deployed code through a dedicated code-read group
normal deploys do not chown the app tree
root only provisions OS resources and performs narrow service operations
```

This is a breaking architectural change. We are assuming there are no existing BonesDeploy-managed deployments to preserve, so we will not add legacy fallbacks, compatibility modes, or `legacy_flip` support.

## Current Problems

The current implementation centers on ownership flipping:

- `docs/PROJECT.md` describes `bonesremote` changing ownership to the deploy user, then hardening back afterward.
- `crates/bonesremote/src/commands/stage_release.rs` uses root to reopen project/build/release paths for the deploy user.
- `crates/bonesremote/src/commands/wire_release.rs` reopens `shared/` and recursively changes shared-path ownership during deploy.
- `crates/bonesremote/src/commands/post_deploy.rs` hardens the active release and `shared/` recursively after deployment.
- `crates/bonesremote/src/permissions.rs` encodes recursive `chown` and recursive mode application as normal deployment behavior.

That design makes every deployment renegotiate filesystem ownership. The new system should provision ownership once, then deploy within that contract.

## Target Architecture

For a site named `foo`:

```text
/srv/sites/foo/
├── releases/                       foo-deploy:foo-code 2750
│   ├── 20260613_180000/
│   ├── 20260613_190000/
│   └── 20260613_200000/
├── build/                          foo-deploy:foo-code 2750
│   └── workspace/
├── current -> releases/20260613_200000
└── shared/                         foo-run:foo-run 0750
    ├── .env
    ├── storage/
    ├── uploads/
    ├── cache/
    └── database.sqlite
```

Users and groups:

```text
foo-deploy  deploys code
foo-run     runs the application
foo-code    shared read/execute group for deployed code
www-data    routes traffic only
root        provisions global OS resources
```

Membership rules:

```text
foo-deploy is in foo-code
foo-run is in foo-code
foo-deploy is not in foo-run
foo-run is not in foo-deploy
```

## Core Rules

```text
Deploy user writes releases.
Runtime user reads releases.
Runtime user writes only declared shared state.
Root provisions OS resources and performs narrow service operations.
```

Normal deployment crosses the boundary by symlinking shared runtime paths into a release. It does not cross by changing ownership.

## Desired Config Shape

Move toward an explicit site/platform contract:

```yaml
site:
  name: foo
  domain: foo.example.com

users:
  deploy: foo-deploy
  run: foo-run
  code_group: foo-code

paths:
  root: /srv/sites/foo
  releases: /srv/sites/foo/releases
  shared: /srv/sites/foo/shared
  current: /srv/sites/foo/current
  build: /srv/sites/foo/build/workspace
  public: public

release:
  keep: 5
  dir_mode: "2750"
  file_mode: "0640"
  executable_mode: "0750"

shared:
  files:
    - .env
  dirs:
    - storage
    - uploads
    - cache

runtime:
  service: foo.service
  user: foo-run
  group: foo-run
  supplementary_groups:
    - foo-code
  runtime_dir: /run/foo

isolation:
  systemd:
    protect_system: strict
    private_tmp: true
    protect_home: true
    no_new_privileges: true
    read_write_paths:
      - /srv/sites/foo/shared/storage
      - /srv/sites/foo/shared/uploads
      - /srv/sites/foo/shared/cache
      - /run/foo
```

This should replace the current implicit split across:

- `data.project_name`
- `data.project_root`
- `permissions.defaults.*`
- runtime-only `shared.paths`
- hard-coded deploy user/group defaults in `shared::paths`

## Implementation Phases

## Phase 1: Replace Naming and Path Derivation

Update `crates/shared/src/paths.rs` and shared config derivation so all product-owned names are deterministic and site-based.

Required changes:

- Change default project root parent from `/srv/deployments` to `/srv/sites`.
- Derive deploy user as `<site>-deploy`.
- Derive runtime user as `<site>-run`.
- Derive code group as `<site>-code`.
- Derive runtime directory as `/run/<site>`.
- Derive nginx and systemd unit names from the site name.
- Keep all path/name derivation centralized under `shared::paths`.

Acceptance criteria:

- No module outside centralized path logic invents new roots or resource names.
- Remote data passed to pyinfra reflects deploy user, runtime user, and code group explicitly.

## Phase 2: Introduce Explicit Identity and Ownership Config

Refactor shared config types under `crates/shared/src/config.rs` and both local/remote config loaders.

Required changes:

- Add explicit site/user/path structs.
- Remove design dependence on `service_user` and `service_group` as the primary model.
- Remove dependence on global `git` and `www-data` as ownership defaults for application code.
- Make shared writable paths explicit in config.
- Make release modes explicit in config.

Acceptance criteria:

- The config schema directly expresses deploy owner, runtime owner, code-read group, mutable shared paths, and runtime isolation paths.
- The same schema is consumed by both `bonesdeploy` and `bonesremote`.

## Phase 3: Move Ownership Setup into Provisioning

Shift ownership and directory creation out of the hot deploy path and into provisioning.

Required provisioning responsibilities:

- Create deploy user.
- Create runtime user.
- Create code-read group.
- Add deploy and runtime users to the code-read group.
- Create site root.
- Create `releases/`, `build/`, `shared/`, and `current` parent paths.
- Set `releases/` and `build/` to deploy-owned setgid directories.
- Set `shared/` to runtime-owned.
- Provision runtime directory ownership and mode.

Likely implementation targets:

- `infra/setup.py`
- `infra/runtime.py`
- new `bonesremote provision-site` command

Acceptance criteria:

- A fresh provisioned site already has correct long-lived ownership before first deploy.
- Normal deploy no longer depends on root to reopen the filesystem.

## Phase 4: Remove Ownership Flipping from Deploy Commands

Refactor remote release commands so normal deployment runs as the deploy user.

Commands to redesign:

- `bonesremote release stage`
- `bonesremote hooks post-receive`
- `bonesremote release wire`
- `bonesremote hooks deploy`
- `bonesremote release activate`
- `bonesremote release prune`
- `bonesremote release rollback`

Required changes:

- Remove recursive `chown` from stage flow.
- Remove shared-dir reopening from wire flow.
- Remove post-deploy hardening of active release and shared state.
- Keep atomic symlink switching for `current`.
- Keep pruning and rollback as symlink/release operations under deploy ownership.

Acceptance criteria:

- A normal deploy performs no recursive `chown` on project files.
- A normal deploy does not require privileged ownership repair.
- Activation and rollback still work by atomically moving `current`.

## Phase 5: Replace `permissions.rs` with Provisioning and Validation Logic

`crates/bonesremote/src/permissions.rs` currently encodes the wrong deployment center.

Refactor it into narrower responsibilities:

- release-local mode application
- provisioning-time ownership helpers
- ownership validation checks
- shared-path safety validation

Required behavior:

- Apply release file and directory modes without changing ownership.
- Optionally apply executable mode to declared executables.
- Validate that shared symlinks resolve under the site `shared/` root.
- Validate that runtime writable paths are explicitly declared.

Acceptance criteria:

- No normal-path function recursively flips release ownership.
- Validation failures are explicit and actionable.

## Phase 6: Rework Shared Path Wiring

`crates/bonesremote/src/commands/wire_release.rs` should wire by reference only.

Required changes:

- Stop moving release content into shared during deploy.
- Stop recursively changing shared path ownership during deploy.
- Validate that each declared shared target already exists under `shared/`.
- Remove any release-local path at the target location.
- Create symlink from release/build path to the shared target.
- Reject absolute paths and parent traversal.

Shared target creation should happen during provisioning or explicit admin/runtime setup, not as an implicit deploy mutation.

Acceptance criteria:

- The deploy user can wire shared paths without owning them.
- Shared paths remain runtime-owned throughout the deploy.

## Phase 7: Narrow the Privileged Surface Area

Current sudoers permissions should be replaced.

Remove privileged deploy-time commands from the default flow:

- `release stage`
- `release wire`
- `hooks post-deploy`

Replace them with narrow privileged operations only where needed:

- `bonesremote provision-site --site <name>`
- `bonesremote service restart --site <name>`
- `bonesremote service reload --site <name>`
- `bonesremote nginx reload`
- `bonesremote systemd install --site <name>`
- `bonesremote runtime install --site <name>`

Rules:

- No privileged command should accept arbitrary shell input.
- No privileged command should operate on arbitrary paths.
- Site-scoped commands must validate deterministic site-owned targets.

Acceptance criteria:

- Sudoers grants only narrow OS-level operations.
- Normal deploy flow runs without broad root privileges.

## Phase 8: Rework Runtime Services and Isolation

Update systemd units and runtime templates so the runtime user reads release code and writes only declared mutable paths.

Base service model:

```ini
[Service]
User=foo-run
Group=foo-run
SupplementaryGroups=foo-code
WorkingDirectory=/srv/sites/foo/current
NoNewPrivileges=yes
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict
ReadWritePaths=/srv/sites/foo/shared/storage
ReadWritePaths=/srv/sites/foo/shared/uploads
ReadWritePaths=/srv/sites/foo/shared/cache
ReadWritePaths=/run/foo
RuntimeDirectory=foo
RuntimeDirectoryMode=0750
```

Required changes:

- Update nginx systemd wrapper templates.
- Update framework runtime services like Laravel PHP-FPM.
- Move writable runtime targets from `current/...` to `shared/...` where appropriate.
- Prefer deploy-time cache generation so runtime write access is minimized.

Acceptance criteria:

- Runtime services run as the runtime user.
- Runtime services can read releases through the code group.
- Runtime services cannot mutate the release tree.

## Phase 9: Rework nginx and Public Asset Policy

nginx should not own application code.

Required rules:

- nginx only proxies to app sockets or serves `current/public`.
- Public files may be world-readable if needed.
- Private code, secrets, and internal storage must remain inaccessible.

Recommended initial policy:

```text
public/      0755
public/*     0644
app code     0750 or 2750 via foo-code group
shared/.env  0640 foo-run:foo-run
```

Acceptance criteria:

- TLS challenges and static public assets work.
- nginx does not require ownership of app code or secrets.

## Phase 10: Add Site Validation and Namespace Checks

BonesDeploy operates in shared global namespaces and needs explicit collision detection.

Add validation commands such as:

- `bonesremote validate-site --config ...`
- `bonesremote doctor --ownership --config ...`
- `bonesremote doctor --namespaces --config ...`

Checks should include:

- deploy user exists
- runtime user exists
- code group exists
- deploy user is in code group
- runtime user is in code group
- site root ownership is correct
- releases/build ownership is correct
- shared ownership is correct
- current is a valid symlink into releases
- shared links point under shared
- nginx path is deterministic and unclaimed
- systemd unit path is deterministic and unclaimed
- domain is not already assigned to another site
- runtime directory path is deterministic and unclaimed

Acceptance criteria:

- BonesDeploy fails before mutating global namespaces when collisions are detected.
- Doctor output is specific enough to repair a broken site contract.

## Phase 11: Update Hook Flow

The hook flow should stop relying on privileged ownership transitions.

Target flow:

```text
pre-push
  -> bonesdeploy doctor --local

pre-receive
  -> bonesremote doctor / validate-site
  -> bonesremote release stage --config ...

post-receive
  -> bonesremote hooks post-receive --config ... --revision <newrev>
  -> bonesremote release wire --config ...
  -> bonesremote hooks deploy --config ...
  -> bonesremote release activate --config ...
  -> sudo bonesremote service restart --site <name>
  -> bonesremote release prune --config ...
```

Acceptance criteria:

- The deploy user can perform the full normal release pipeline.
- Only service restart/reload remains privileged.

## Phase 12: Update Templates, Docs, and Tests

Files that will need coordinated updates:

- `docs/PROJECT.md`
- `README.md`
- `docs/commands/bonesdeploy/*.md`
- `docs/commands/bonesremote/*.md`
- `infra/setup.py`
- `infra/runtime.py`
- `infra/assets/**/*.j2`
- `crates/bonesdeploy/embeds/kit/*.yaml`
- `crates/bonesdeploy/embeds/runtimes/*/runtime.yaml`
- framework-specific runtime operations and service templates
- unit/integration tests covering init assets, doctor behavior, remote data, and release flows

Specific documentation changes:

- Remove language describing temporary ownership handoff.
- Remove docs that present `www-data` as the application ownership group.
- Document deploy-owned releases and runtime-owned shared state as the primary model.
- Document root as a provisioning-only boundary except for narrow service commands.

Acceptance criteria:

- Docs match the codebase.
- Tests assert the new deployment contract, not the legacy ownership-flip model.

## Non-Goals

These are explicitly not part of the first rehaul pass:

- legacy compatibility modes
- automatic migration of old BonesDeploy installs
- ACL-based solutions
- containerization as the primary isolation mechanism
- AppArmor-first redesign before the filesystem model is stable

## Final Design Principle

BonesDeploy should compile site config into a validated OS contract:

```text
site config
  -> users
  -> groups
  -> directories
  -> release layout
  -> shared mutable paths
  -> systemd units
  -> nginx config
  -> runtime directories
  -> optional AppArmor profiles
  -> validation checks
```

The result should be a system where ownership is already correct before deployment starts, so deployments only build, wire, activate, restart, and prune.
