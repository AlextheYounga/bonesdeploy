# BonesInfra Security Responsibilities

This note scopes the new security model down to the parts that belong inside
`bonesinfra`.

`bonesinfra` is the hidden provisioning engine. It prepares the host so
`bonesremote` can safely build, harden, prepare, activate, and restart a site.

It should not own deployment lifecycle behavior.

______________________________________________________________________

## Target Boundary

`bonesinfra` handles host setup.

`bonesremote` handles release lifecycle.

The security model depends on this split:

```text
git = ingress only
foo = one per-site runtime identity
podman build = temporary build environment, managed by bonesremote
shared/ = persistent runtime state, owned by foo
releases/ = sealed promoted artifacts, owned by root:foo
root/bonesremote = privileged mediator
```

Inside `bonesinfra`, the goal is only to provision the machine into this shape.

______________________________________________________________________

## BonesInfra Owns

### 1. Ingress User Provisioning

`bonesinfra` should create and maintain the global deploy ingress identity:

```text
user:  git
group: git
role:  receive pushes only
```

The `git` user should own bare repositories and SSH ingress material.

It should not own site state.

It should not own release code.

It should not own framework caches.

It should not be a member of any per-site runtime group.

For v1, `git` may remain the global ingress user. Hardening its shell to
`git-shell` is a host provisioning concern and can be handled by `bonesinfra`
when that decision is made.

### 2. Per-Site Runtime Identity

`bonesinfra` should create one Unix user/group per site:

```text
user:  <project>
group: <project>
shell: /usr/sbin/nologin
home:  /nonexistent
sudo:  none
```

This identity is the runtime identity for framework commands and per-site
services.

It should be able to read sealed release artifacts through group ownership.

It should be able to write persistent runtime state under `shared/`.

It should not be able to modify ordinary release code.

### 3. Repository Location

`bonesinfra` should provision the bare repository location as ingress-owned
state:

```text
/srv/git/<project>.git
  git:git
```

The repository is untrusted for privileged host decisions.

Repo-owned `bones.toml` and runtime config may describe app preferences, but
they must not be the source of truth for privileged operations.

### 4. Site Directory Skeleton

`bonesinfra` should create the site filesystem skeleton:

```text
/srv/sites/<project>/
/srv/sites/<project>/shared/
/srv/sites/<project>/releases/
/srv/sites/<project>/current
```

The intended ownership model is:

```text
/srv/sites/<project>/
  root:root or root:<project>

/srv/sites/<project>/shared/
  <project>:<project>

/srv/sites/<project>/releases/
  root:<project>

/srv/sites/<project>/current
  root:root symlink
```

`shared/` is runtime-owned persistent state.

`releases/` is where `bonesremote` later promotes sealed artifacts.

`current` is later flipped by `bonesremote` as root.

`bonesinfra` may seed a placeholder release if needed for first-time nginx
validation, but that placeholder must follow the same sealed-release intent:
root-owned, group-readable by the site user, not runtime-writable as code.

### 5. Shared Runtime State Directories

`bonesinfra` provisions the `shared/` parent and its permissions.

Frameworks own creation of their writable leaves and files under `shared/`.

`bonesinfra` does not pre-create framework-specific writable leaves.

`bonesremote` owns wiring those paths into each release.

### 6. Runtime Service Provisioning

`bonesinfra` owns service provisioning details:

```text
systemd units
nginx config
php-fpm pools
runtime sockets
AppArmor profiles
base package installation
runtime package installation
```

Those services should run as the per-site runtime identity where possible.

Service configs should reference the active `current` release and the `shared/`
paths that `bonesremote` wires.

They should not require nginx, php-fpm, or app runtime services to read the
git-owned repository.

### 7. AppArmor And Filesystem Policy

`bonesinfra` owns AppArmor and service sandbox policy.

The policy should match the new ownership model:

```text
runtime can read current release code
runtime can write shared runtime state
runtime cannot write normal release code
runtime does not need repo-owned bones.toml
git does not need site state access
```

Any existing AppArmor grant that lets runtime services read git-owned config
should be removed when the registry-based model is implemented.

### 8. Root-Owned Registry Path Preparation

The trusted site registry is a `bonesremote` concern, but `bonesinfra` may
prepare its parent directory and permissions:

```text
/etc/bonesdeploy/sites/
  root:root
```

`bonesremote init` should write the actual site registry file:

```text
/etc/bonesdeploy/sites/<project>.toml
```

`bonesinfra` may validate that the registry exists before provisioning runtime
services once the registry becomes required input.

### 9. Sudoers Contract Support

`bonesinfra` installs the single sudoers drop-in for BonesDeploy sites:

```text
/etc/sudoers.d/bonesdeploy
```

The commands granted through sudo belong to the `bonesremote` command contract.

The important responsibility is that sudoers rules stay narrow and registry-path
based or site-literal, not repo-config based. `bonesinfra` should not create
per-project sudoers files without a real lifecycle requirement.

Good shape:

```text
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote deploy --site <project>
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote service restart --site <project>
```

Bad shape:

```text
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote * --config *
```

The exact command surface is a `bonesremote` contract. `bonesinfra` installs
only the final agreed narrow rules and validates them with `visudo`.

______________________________________________________________________

## BonesInfra Does Not Own

These concerns should stay out of `bonesinfra`.

### 1. Git Hook Runtime Behavior

`bonesinfra` may place hook templates if the setup contract requires it, but it
should not own hook deployment logic.

The hook behavior belongs to `bonesremote` and `bonesdeploy`.

Target behavior:

```text
git receives push
git identifies repo/revision
git calls narrow sudo bonesremote deploy for the registered site
```

### 2. Source Export

Exporting source from the bare repo into a build context belongs to
`bonesremote`.

`bonesinfra` should not check out code.

`bonesinfra` should not archive source.

`bonesinfra` should not decide which revision is deployable.

### 3. Podman Build Execution

Disposable Podman builds belong to `bonesremote`.

`bonesinfra` may install Podman and any host packages required to run it.

It should not run build containers.

It should not mount build contexts.

It should not manage build output.

It should not provide build scripts with access to `shared/`, `.env`,
`current`, `releases/`, `/srv/git`, or `/etc/bonesdeploy`.

### 4. Promotion Hardening

Turning untrusted build output into a sealed `releases/<release_id>` tree belongs to `bonesremote`.

That includes:

```text
copying artifact output
rejecting unsafe artifact contents
normalizing ownership
normalizing modes
setting root:<project> ownership
```

`bonesinfra` only provisions the destination directory and the user/group model
that makes sealed releases meaningful.

### 5. Shared Symlink Wiring

Wiring each release to shared runtime state belongs to `bonesremote`.

Examples:

```text
release/.env -> shared/.env
release/storage -> shared/storage
release/bootstrap/cache -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
```

`bonesinfra` should create the baseline shared targets.

It should not mutate a release during deploy.

### 6. Runtime Prepare Commands

Runtime prepare belongs to `bonesremote`.

Examples:

```text
php artisan migrate --force
php artisan optimize
php artisan queue:restart
```

`bonesinfra` provisions the runtime user and service environment.

`bonesremote` decides when runtime prepare runs and whether activation should
continue.

### 7. Activation

Changing `current` belongs to `bonesremote` running as root.

`bonesinfra` may create the initial symlink or placeholder needed for first
setup.

It should not decide which release is active after deployment starts.

### 8. Service Restart During Deploy

Restarting services after activation belongs to `bonesremote`.

`bonesinfra` creates the systemd units.
`bonesinfra` owns sudoers installation.

It should not be part of the deploy-time restart path.

### 9. Rollback And Pruning

Rollback and old-release pruning belong to `bonesremote`.

`bonesinfra` should not select releases to keep or remove after provisioning.

### 10. Secret Push Semantics

Secret write behavior belongs to `bonesdeploy` and `bonesremote`.

`bonesinfra` may create the shared directory and expected runtime identity.

It should not trust repo-owned runtime config for privileged secret ownership
decisions.

______________________________________________________________________

## BonesInfra Setup Checklist

A v1 implementation inside `bonesinfra` should focus on this checklist:

```text
create global git ingress user
create per-site runtime user/group
ensure git is not in the site group
create /srv/git/<project>.git as git:git
create /srv/sites/<project> skeleton
create shared/ as <project>:<project>
create releases/ as root:<project>
avoid permanent site build/ directories
install base packages and runtime packages
install podman if bonesremote requires it
install nginx/php-fpm/systemd/AppArmor config
remove runtime access to git-owned config
prepare /etc/bonesdeploy/sites parent if needed
install only narrow registry-backed sudoers rules
```

Anything that happens per release should be treated as `bonesremote` work unless
it is only provisioning static host capability.

______________________________________________________________________

## One-Sentence Rule

`bonesinfra` makes the host safe for deployment; `bonesremote` performs the
deployment.
