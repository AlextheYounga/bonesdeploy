# Security Model Migration Plan

This plan migrates BonesDeploy to the v1 security model without compatibility ballast. There are no existing installs to preserve.

Target boundary:

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary, disposable build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
```

Hard decisions already made:

- Keep bare repositories under `/home/git/<project>.git`.
- Do not push `.bones/` into bare repos.
- `bonesdeploy push` publishes the deployment dataset to `bonesremote` control-plane state.
- Use `/root/.config/bonesremote` for v1 remote control-plane state.
- No control-plane generations/history model.
- Build scripts mutate `/workspace/source`; no required `/workspace/output`.
- Prepare scripts run on the host as `foo` after shared paths are wired and before activation.
- Privileged paths, users, groups, and services come from bonesremote-controlled state, not repo-owned config.

## Phase 0: Freeze the Contract

Before changing code, write down the v1 command and filesystem contract so the rewrite has a stable target.

Remote layout:

```text
/home/git/<project>.git
/root/.config/bonesremote/sites/<project>/
  registry.toml
  runtime.toml
  deployment/build/*.sh
  deployment/prepare/*.sh
/srv/sites/<project>/
  shared/
  releases/
  current -> releases/<release_id>
```

Deployment phases:

```text
hook trigger
validate control-plane state
resolve revision and branch policy
export source into temp dir
run build scripts in podman at /workspace/source
promote mutated source into sealed release
wire shared paths
run prepare scripts as foo
activate current as root
restart registry-approved services
prune and cleanup
```

Script contract:

- Build scripts run in Podman with `cwd=/workspace/source`.
- Build scripts receive no `.env`, `shared/`, `current`, `releases/`, bare repo, `/root`, or control-plane mounts.
- Build scripts must not depend on `.git` existing.
- Build scripts write the deployable app shape into `/workspace/source`.
- Prepare scripts run as the site user with `cwd=<release_path>`.
- Prepare scripts can see wired shared paths and secrets.
- Prepare scripts are where migrations, optimize/cache commands, and runtime-state work belong.

Exit criteria:

- `docs/PROJECT.md` and command docs no longer describe bare-repo `.bones/` sync as the target behavior.
- The v1 script/environment contract is documented once and used by templates.

## Phase 1: Introduce Remote Control-Plane State

This is the first code boundary because it removes privileged authority from Git-owned files before the deploy pipeline is rewritten.

Implement bonesremote site state under:

```text
/root/.config/bonesremote/sites/<project>/
```

State contents:

- `registry.toml`: canonical derived state accepted by `bonesremote`.
- `runtime.toml`: user-editable runtime preferences after validation.
- `deployment/build/*.sh`: user-provided build scripts.
- `deployment/prepare/*.sh`: user-provided prepare scripts.

Keep it simple:

- Import into a temporary sibling path.
- Validate names, paths, script modes, runtime fields, and service identifiers.
- Replace the current site directory atomically enough to avoid partial reads.
- Do not add generations, rollback history, drift warnings, or audit trails.

`registry.toml` should derive or constrain privileged values:

- `site = <project>`.
- `repo_path = /home/git/<project>.git`.
- `site_root = /srv/sites/<project>`.
- `shared_root = /srv/sites/<project>/shared`.
- `releases_root = /srv/sites/<project>/releases`.
- `current_path = /srv/sites/<project>/current`.
- `runtime_user = <project>`.
- `runtime_group = <project>`.
- service names come from a constrained server-side convention or allowlist.
- `branch` and `deploy_on_push` live here because hook filtering belongs to bonesremote.

Exit criteria:

- `bonesremote` can import, validate, read, and export one site's current dataset.
- Invalid site names cannot escape `/root/.config/bonesremote/sites` or derive arbitrary host paths.
- Privileged commands can load the registry without reading from the bare repo.

## Phase 2: Rewire `bonesdeploy push` and `pull`

Change the existing command names, not the operator workflow.

`bonesdeploy push` becomes:

```text
local .bones dataset -> SSH/stream/upload -> bonesremote import-site
```

`bonesdeploy pull` becomes:

```text
bonesremote export-site -> local .bones recovery/update
```

Do not write deployment control-plane data into `/home/git/<project>.git/bones/` anymore.

Local `.bones/bones.toml` remains operator input. It is not privileged remote authority.

Exit criteria:

- A local project can publish `.bones/runtime.toml` and deployment scripts to bonesremote state.
- Pull recovers the currently published dataset.
- Bare repos contain source and hooks only, not the control plane.

## Phase 3: Make Hooks Thin

Replace hook behavior with a narrow trigger.

Remote `post-receive` should do only enough to pass the pushed ref/revision to bonesremote:

```text
bonesremote hook post-receive --site <project>
```

`bonesremote hook post-receive` owns:

- reading stdin ref updates.
- loading control-plane state.
- checking `branch` and `deploy_on_push`.
- calling the internal deploy path for the accepted revision.

The hook must not:

- parse `.bones` from the repo.
- check out source.
- run build scripts.
- write releases or shared state.
- restart services.

Exit criteria:

- Pushing an unrelated branch does nothing.
- Pushing the configured branch deploys via bonesremote.
- `doctor` can verify the bare repo hook exists and points to the thin trigger.

## Phase 4: Replace Source Checkout with Source Export

Delete the permanent build workspace path from the deployment model.

`bonesremote` should export source with the bare repo as input and a temporary directory as output:

```text
git --git-dir=/home/git/<project>.git archive <revision> | tar -x -C <temp>/source
```

The build container sees `/workspace/source`, not the bare repo and not `.git` history.

Exit criteria:

- No `/srv/sites/<project>/build` workspace is required.
- Build input is deleted on success and failure.
- Revisions are resolved against the registry-approved repo path.

## Phase 5: Add Disposable Podman Build Execution

Replace host build scripts with container build scripts.

Template layout:

```text
.bones/deployment/build/01_install_deps.sh
.bones/deployment/build/02_build_assets.sh
.bones/deployment/prepare/01_migrate.sh
.bones/deployment/prepare/02_optimize.sh
```

Build execution:

- Run a configured image from `runtime.toml` after validation.
- Mount exported source as `/workspace/source`.
- Mount only approved cache dirs such as `/var/cache/bonesremote/<project>/composer` or npm cache when needed.
- Run build scripts in lexical order.
- Treat non-zero exit as deployment failure.
- Return the mutated `/workspace/source` path to the promotion step.

Do not hardcode Laravel commands in bonesremote. Templates can scaffold Laravel-friendly scripts, but scripts remain user-owned.

Exit criteria:

- Host deploys no longer install Composer, Node, npm, or build dependencies into the `git` user's home.
- Build has no access to secrets or runtime state.
- A failing build leaves no release and does not affect `current`.

## Phase 6: Promote and Harden the Mutated Source

Promotion turns untrusted built source into a sealed release.

Input:

```text
<temp>/source
```

Output:

```text
/srv/sites/<project>/releases/<release_id>
```

Promotion rules:

- Copy regular files and safe symlinks only.
- Reject device files, FIFOs, sockets, setuid, and setgid.
- Reject absolute symlinks and symlinks that escape the release/shared boundary.
- Set directories to `0750`.
- Set regular files to `0640`.
- Preserve executable intent as `0750` only where needed.
- Set owner/group to `root:<project>`.

Exit criteria:

- Runtime user can read and execute what it needs.
- Runtime user cannot rewrite sealed release code.
- Dangerous build output is rejected before activation.

## Phase 7: Wire Shared Paths

Replace `.env`-only wiring with declared shared path wiring.

Laravel v1 paths:

```text
release/.env                     -> shared/.env
release/storage                  -> shared/storage
release/bootstrap/cache          -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
release/public/storage           -> ../storage/app/public
```

`bonesremote` should normally link existing shared paths. Base shared paths are a bonesinfra/runtime provisioning concern.

Exit criteria:

- A sealed release exposes the runtime-mutable paths Laravel needs.
- Shared targets are under registry-approved `shared_root`.
- Missing required shared paths fail loudly with an actionable error.

## Phase 8: Run Runtime Prepare as the Site User

Prepare is the first phase allowed to touch secrets and runtime state.

Run as:

```text
user: <project>
cwd:  /srv/sites/<project>/releases/<release_id>
```

Execute scripts from control-plane state:

```text
deployment/prepare/*.sh
```

Important tradeoff:

- Migrations may mutate the database before activation.
- If activation later fails, rollback is code rollback, not database rollback.
- Logs and docs must say this plainly.

Exit criteria:

- Prepare runs as `foo`, not `git` and not root.
- Failed prepare deletes the incomplete release and leaves `current` untouched.
- Successful prepare is required before activation.

## Phase 9: Activate, Restart, Prune

Privileged finalization stays in bonesremote.

Activation:

- Atomically repoint `/srv/sites/<project>/current` to the prepared release.
- Never activate a release that failed build, promotion, wiring, or prepare.

Service control:

- Restart/reload only registry-approved services.
- Do not accept service names from repo-owned config or hook input.

Pruning:

- Keep the existing simple release retention model.
- Do not add config generations or audit-history pruning.
- Never prune the active release.

Exit criteria:

- Deploy failure before activation leaves the active release running.
- Rollback repoints `current` to a previous sealed release and restarts the approved service.
- Old inactive releases are pruned according to the configured keep count.

## Phase 10: Move Secrets Placement Behind BonesRemote

This can land after the core pipeline if needed, but it must be done before calling the security model complete.

`bonesdeploy` keeps local encrypted secret editing and transport.

`bonesremote` owns final placement:

- destination path from registry/control-plane state.
- owner/group from registry/control-plane state.
- file mode from bonesremote policy.
- atomic write to `shared/.env` or equivalent declared secret target.

Exit criteria:

- No `chown root:<group>` decision uses repo-owned `runtime.toml` as authority.
- Secret delivery cannot write outside the registry-approved shared path.
- Runtime user can read the secret file through wired paths.

## Phase 11: Update Doctor and Documentation

Doctor checks should match the new boundary.

Remote checks:

- bonesremote installed.
- Podman available.
- control-plane site state exists and validates.
- `/home/git/<project>.git` exists.
- thin hook installed.
- `/srv/sites/<project>/shared`, `releases`, and `current` parent exist with expected ownership shape.
- runtime user exists and is not `git`.
- `git` is not in the site group.
- approved services exist.

Docs to update:

- `docs/PROJECT.md` deployment model.
- `docs/commands/push.md` and `pull.md` control-plane semantics.
- `docs/commands/deploy.md` deploy sequence.
- `docs/commands/secrets.md` mediated placement.
- `docs/security/*` trust-boundary and permissions docs.

Exit criteria:

- Docs no longer say `.bones/` is rsynced into the bare repo.
- Docs no longer describe host-side build scripts running as `git`.
- Docs explicitly state the migration-before-activation database tradeoff.

## Suggested Implementation Order

1. Add shared registry/control-plane structs and path helpers.
2. Add bonesremote import/export of site datasets.
3. Rewire `bonesdeploy push` and `pull` to use import/export.
4. Replace remote hook with `bonesremote hook post-receive --site <site>`.
5. Replace permanent checkout/stage code with source export.
6. Add Podman build runner for `/workspace/source`.
7. Add promotion hardening.
8. Replace shared wiring.
9. Add prepare script runner as site user.
10. Rewrite deploy orchestration around the new phases.
11. Move secrets final placement into bonesremote.
12. Update doctor checks and docs.
13. Delete obsolete stage/post-receive/old deployment-script code.

This order keeps the trust boundary moving in one direction: first move authority out of Git, then make Git a trigger, then replace the deploy pipeline.

## Small Checks to Leave Behind

Each non-trivial phase should leave one runnable check:

- Control-plane import rejects `../evil` site names and absolute user-supplied paths.
- Hook filtering ignores non-configured branches.
- Source export produces a tree without `.git`.
- Podman build cannot read a fake mounted secret path.
- Promotion rejects an absolute symlink and a FIFO.
- Shared wiring rejects targets outside `shared_root`.
- Prepare runs as the site user, verified by a tiny script writing `id -un` to a temp file.
- Deploy failure before activation leaves `current` unchanged.

Keep these checks small. The goal is not a huge test harness; it is one tripwire per boundary.
