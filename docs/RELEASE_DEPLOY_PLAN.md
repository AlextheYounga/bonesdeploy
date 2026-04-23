# Release Deploy Plan

## Goal

Replace the current in-place checkout flow with a release-based deployment flow:

- build/check out each deploy into a new release directory under `/srv/deployments/<project>/releases/`
- only switch traffic after a successful deploy by updating a symlink
- allow rollbacks by repointing that symlink to an older release
- keep persistent files in a shared location outside individual releases

This plan intentionally does not include `redeploy`. That command belongs to the old deploy model and should be removed from scope.

## Target Layout

For a project named `lawsnipe`:

```text
/srv/deployments/lawsnipe/
├── current -> /srv/deployments/lawsnipe/releases/20260419_121530
├── releases/
│   ├── 20260419_115900/
│   └── 20260419_121530/
└── shared/
    ├── .env
    └── storage/

/var/www/lawsnipe -> /srv/deployments/lawsnipe/current
```

Definitions:

- `deploy_root`: release management root, for example `/srv/deployments/lawsnipe`
- `live_root`: public app path, for example `/var/www/lawsnipe`
- `current`: symlink to the active release
- `shared`: persistent files and directories that survive across releases

## Deployment Flow

### Current Flow

Today `kit/hooks/post-receive` checks out directly into the configured worktree:

```bash
git --work-tree="$WORKTREE" --git-dir="$GIT_DIR" checkout -f "$BRANCH"
```

That means a failed deploy can leave the live directory partially updated.

### New Flow

1. `pre-receive`
   - run `bonesremote doctor`
   - run `bonesremote release stage --config ...`
2. `stage-release`
   - create `deploy_root/releases/` and `deploy_root/shared/` if missing
   - create a new timestamped release directory
   - ensure `live_root` is a symlink to `deploy_root/current`
   - write the chosen release name to a state file under `git_dir/bones/`
3. `post-receive`
   - read the staged release name from the state file
   - `git checkout -f` into `deploy_root/releases/<release>/`
   - run `bonesremote release wire --config ...`
   - call `deploy`
4. `wire-release`
   - seed and symlink configured shared paths into the staged release
   - make the release ready for framework-specific build steps
5. `deploy`
   - `cd` into the new release directory
   - run deployment scripts from `bones/deployment/`
   - call `bonesremote release activate --config ...`
6. `activate-release`
   - atomically update `deploy_root/current`
   - leave `live_root` pointing at `deploy_root/current`
   - prune old releases beyond the configured retention count
   - clean up staged release state
7. `post-deploy`
   - run `bonesremote hooks post-deploy --config ...`
   - harden permissions on the active release and shared paths

If deployment scripts fail before activation:

- `current` still points at the previous release
- `live_root` still serves the previous release
- the failed release directory should be deleted

## Shared Paths

Shared paths are not copied into each release. They are stored once in `deploy_root/shared/` and then symlinked into the staged release before deployment scripts run.

Example:

```text
/srv/deployments/lawsnipe/releases/20260419_121530/.env -> /srv/deployments/lawsnipe/shared/.env
/srv/deployments/lawsnipe/releases/20260419_121530/storage -> /srv/deployments/lawsnipe/shared/storage
```

This keeps persistent files stable across deploys and rollbacks.

First wire behavior:

- if the checked-out release contains a configured shared path and `shared/<path>` does not exist yet, move that path into `shared/`
- if neither exists, create an empty target suitable for the path type
- replace the release path with a symlink to the shared path

Initial default shared paths should remain conservative. Good starter examples:

- `.env`
- `storage`

## Config Changes

Both config structs currently use `data.worktree`:

- `crates/bonesdeploy/src/config.rs`
- `crates/bonesremote/src/config.rs`

Replace that with explicit release-oriented fields.

### Proposed Shape

```toml
[data]
remote_name = "production"
project_name = "lawsnipe"
host = "deploy.example.com"
port = "22"
git_dir = "/home/git/lawsnipe.git"
live_root = "/var/www/lawsnipe"
deploy_root = "/srv/deployments/lawsnipe"
branch = "master"

[permissions.defaults]
deploy = "git"
owner = "applications"
group = "www-data"
dir_mode = "750"
file_mode = "640"

[releases]
keep = 5
shared_paths = [".env", "storage"]
```

### Rust Struct Changes

In both config files:

- remove `worktree`
- add `live_root`
- add `deploy_root`
- add `releases: Releases`

Suggested struct shape:

```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Releases {
    #[serde(default = "default_keep")]
    pub keep: usize,
    #[serde(default)]
    pub shared_paths: Vec<String>,
}
```

Also update `is_configured()` in `crates/bonesdeploy/src/config.rs` to require both `live_root` and `deploy_root`.

## Prompt Changes

Update `crates/bonesdeploy/src/prompts.rs`:

- replace the `Worktree path on remote:` prompt
- add `Live root on remote:` with default `/var/www/<project_name>`
- add `Deploy root on remote:` with default `/srv/deployments/<project_name>`
- add `Releases to keep:` with default `5`
- add `Shared paths:` with a simple comma-separated prompt

The generated `kit/bones.toml` should include comments explaining:

- `live_root` is the path your web server or service points at
- `deploy_root` stores `releases/`, `shared/`, and `current`
- `shared_paths` are symlinked into each release

## Remote Command Changes

Current remote commands live in `crates/bonesremote/src/commands/` and only cover:

- `doctor`
- `pre_deploy`
- `post_deploy`

Add these commands:

- `stage_release.rs`
- `wire_release.rs`
- `activate_release.rs`
- `rollback.rs`
- `drop_failed_release.rs`

Update `crates/bonesremote/src/commands/mod.rs` to register them.

### stage-release

Responsibilities:

1. load config
2. create `deploy_root`, `deploy_root/releases`, and `deploy_root/shared`
3. create a new release directory name, for example `YYYYMMDD_HHMMSS`
4. create that directory
5. make sure `live_root` is a symlink to `deploy_root/current`
6. chown the new release directory and `shared/` to the deploy user
7. write the staged release name to `git_dir/bones/.staged_release`

### activate-release

Responsibilities:

1. load config
2. read `.staged_release`
3. resolve the release directory under `deploy_root/releases/`
4. atomically switch `deploy_root/current` to the new release
5. prune old releases, excluding the newly active one
6. remove `.staged_release`

### wire-release

Responsibilities:

1. load config
2. read `.staged_release`
3. resolve the release directory under `deploy_root/releases/`
4. for each `shared_paths` entry:
   - if the release path exists and the shared path does not, move it into `shared/`
   - if neither exists, create an empty target suitable for the intended path
   - replace the release path with a symlink to the shared path

This command must run after checkout and before deployment scripts. Without it, frameworks like Laravel will not have access to `.env`, `storage/`, or other persistent paths during build and migration steps.

Atomic switch should use the standard pattern:

1. create a temporary symlink
2. rename it into place as `current`

### drop-failed-release

Responsibilities:

1. load config
2. read `.staged_release` if present
3. delete the matching release directory
4. remove `.staged_release`

This keeps failed release attempts from accumulating.

### rollback

Responsibilities:

1. load config
2. resolve the release currently pointed to by `deploy_root/current`
3. list releases in sorted order
4. choose the previous release
5. atomically repoint `current`
6. print old and new release names

## Permissions Changes

`crates/bonesremote/src/permissions.rs` currently assumes a single deployment root via `cfg.data.worktree`.

That file needs to be refactored so permission changes can target specific paths.

Suggested API split:

- `chown_paths_to_deploy_user(cfg, paths: &[&Path])`
- `harden_paths(cfg, paths: &[&Path])`
- `harden_active_release(cfg)`

Rules:

- before deploy, ownership should be granted only where needed: staged release dir, `shared/`, and symlink management paths
- after deploy, harden the active release and shared paths, not the whole `deploy_root`
- do not recursively chmod old releases during a normal deploy

## Hook Changes

Update the embedded hook templates in `kit/hooks/`.

### pre-deploy

Replace the old call to `bonesremote pre-deploy` with:

```bash
sudo bonesremote release stage --config "$BONES_TOML"
```

### post-receive

Stop checking out into `worktree`. Instead:

1. read `deploy_root` and `branch` from `bones.toml`
2. read `.staged_release`
3. checkout into `deploy_root/releases/<release>`
4. run `sudo bonesremote release wire --config "$BONES_TOML"`

### deploy

Change the working directory from `worktree` to the staged release path.

If deployment scripts succeed:

```bash
sudo bonesremote release activate --config "$BONES_TOML"
exec "$HOOKS_DIR/post-deploy"
```

If a deployment script fails:

```bash
sudo bonesremote release drop-failed --config "$BONES_TOML"
exit 1
```

### post-deploy

Keep the same high-level behavior, but the remote command should now harden the active release plus shared paths rather than an old single worktree.

## Laravel Fit

The Laravel template fits this design, but it exposed one important correction to the original plan.

### Shared paths must exist before deployment scripts

`templates/laravel/deployment/01_run_deployment_concerns.sh` runs Laravel and build commands that depend on runtime files already being present in the release:

- `composer install`
- `pnpm install`
- `php artisan key:generate --force`
- `php artisan migrate --force`
- cache rebuild commands

That means `.env` and `storage/` cannot wait until final activation. They must be attached to the staged release immediately after checkout. That is why this plan now includes `wire-release`.

### Maintenance mode timing should move later

`templates/laravel/hooks/post-receive` currently enters maintenance mode before checkout. In a release-based model that is too early, because the old release should stay live while the new release is being prepared.

For Laravel, maintenance mode should happen only if needed and as late as possible, ideally in the deployment script near migration or activation time.

### Laravel shared path defaults

For the Laravel template, these are good shared path defaults:

- `.env`
- `storage`

When SQLite is used, this should also be configurable as shared:

- `database/database.sqlite`

`bootstrap/cache` should stay release-local, but writable in the active release.

## Local CLI Changes

### Remove redeploy

Remove `Redeploy` from:

- `crates/bonesdeploy/src/commands/mod.rs`
- `crates/bonesdeploy/src/commands/redeploy.rs`

### Add rollback

Add a local `rollback` command that SSHes to the remote and runs:

```bash
sudo bonesremote release rollback --config <remote bones.toml path>
```

This should follow the same SSH execution pattern as the existing local commands.

## Documentation Updates

After implementation, update these docs:

- `docs/PROJECT.md`
- `README.md`

Areas to update:

- config examples
- hook descriptions
- deployment flow descriptions
- remote command list
- nginx or service target examples should point at `live_root`, which itself points to `deploy_root/current`

## Suggested Implementation Order

1. update config structs in both crates
2. update prompt collection and `kit/bones.toml`
3. refactor remote permissions helpers to accept target paths
4. add remote release-management commands
5. update hook templates to use staged release flow
6. remove `redeploy` and add local `rollback`
7. update docs
8. run `cargo check`

## Verification Checklist

1. `cargo check`
2. run `bonesdeploy init` and verify generated config comments are clear
3. run `bonesdeploy push` and confirm remote hooks are updated
4. push a deployment and verify:
   - a new release directory is created
   - `deploy_root/current` points to the new release
   - `live_root` points to `deploy_root/current`
   - shared paths are symlinked into the release
   - old site stays live if a deploy script fails
5. push multiple times and verify old releases are pruned to `keep`
6. run `bonesdeploy rollback` and verify `current` points to the previous release
