Yes. The v1 target should be:

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
```

The point is not perfect theoretical isolation. The point is a sane and explainable boundary:

```text
A build can produce an artifact.
A build cannot touch runtime state.
Runtime can touch runtime state.
Runtime cannot rewrite sealed release code.
git cannot directly touch site state.
Only bonesremote performs privileged host operations.
```

## Phase 1: simplify the identity model

Move away from `foo-build`, `foo-run`, `foo-release`, `foo-shared` for v1.

Use one per-site Unix identity:

```text
user:  foo
group: foo
shell: /usr/sbin/nologin
home:  /nonexistent or controlled non-login path
sudo:  none
```

`git` remains global ingress, but it should not be in `foo`. It should not own releases, shared files, build output, runtime state, or framework caches.

Conceptual rule:

```text
git owns repos.
foo owns mutable site state.
root owns host control and sealed releases.
```

## Phase 2: establish the filesystem model

Target layout:

```text
/srv/git/foo.git
  git:git
  bare repo, ingress only

/srv/sites/foo/
  root:root or root:foo
  site boundary

/srv/sites/foo/shared/
  foo:foo
  persistent runtime state

/srv/sites/foo/releases/
  root:foo
  promoted release artifacts

/srv/sites/foo/releases/<release_id>/
  root:foo
  readable by foo, not writable by foo

/srv/sites/foo/current
  root:root symlink to active release
```

Laravel shared paths should be explicit:

```text
shared/.env
shared/storage/
shared/bootstrap/cache/
shared/database/database.sqlite
```

Release symlinks:

```text
release/.env                    -> shared/.env
release/storage                 -> shared/storage
release/bootstrap/cache         -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
release/public/storage          -> ../storage/app/public
```

Do not put a permanent `build/` directory under `/srv/sites/foo` for v1. Builds should be throwaway.

## Phase 3: make builds disposable

The deploy flow should become:

```text
1. git receives push.
2. git invokes bonesremote through a narrow sudo rule.
3. bonesremote reads root-owned site registry.
4. bonesremote exports source into a temporary build context.
5. podman runs the build.
6. podman receives no shared/, no .env, no SQLite, no current release.
7. podman produces an artifact directory.
8. container is destroyed.
9. temporary build context is deleted after promotion/failure.
```

The build container should see only something like:

```text
/workspace/source
/workspace/output
/cache/composer
/cache/npm
/tmp
```

It should not see:

```text
/srv/sites/foo/shared
/srv/sites/foo/current
/srv/sites/foo/releases
/srv/git/foo.git
/etc/bonesdeploy
/root
```

For Laravel, sandbox build commands are probably:

```sh
composer install --no-dev --prefer-dist --no-interaction --optimize-autoloader

if [ -f package.json ]; then
  npm ci
  npm run build
fi
```

Do not run migrations, config caching, storage linking, queue restarts, or service restarts inside the build container.

## Phase 4: promote artifacts through bonesremote

After the Podman build succeeds, BonesRemote should promote the artifact.

Conceptually:

```text
artifact output -> /srv/sites/foo/releases/<release_id>
```

But BonesRemote must treat artifact output as untrusted. It should normalize and seal it:

```text
owner: root
group: foo
dirs:  0750
files: 0640
executables/scripts: 0750
```

It should reject or carefully handle dangerous artifact contents:

```text
absolute symlinks
symlinks escaping the release/shared boundary
setuid/setgid bits
device files
FIFOs/sockets
unexpected ownership/modes
```

This matters because root is promoting files produced by project-controlled build code.

## Phase 5: wire shared paths before runtime prepare

Before Laravel runtime commands run, BonesRemote wires the release:

```text
.env
storage/
bootstrap/cache/
database/database.sqlite
public/storage
```

Those writable paths point into `shared/`, which is owned by `foo:foo`.

The sealed release itself remains `root:foo`, so `foo` can read the app but cannot rewrite normal code.

## Phase 6: run runtime prepare as foo

This is the phase where Laravel touches runtime state.

Run as `foo`, not root:

```sh
cd "$BONES_RELEASE_PATH"

php artisan migrate --force
php artisan optimize
```

Potentially:

```sh
php artisan package:discover --ansi
php artisan queue:restart
```

But keep the conceptual split clear:

```text
build phase:
  no secrets, no shared state

runtime prepare phase:
  sees .env
  sees SQLite
  sees storage
  writes framework caches
  runs migrations

root phase:
  flips current
  restarts system services
```

This is the key Laravel baseline. Other frameworks can use the same phase model even if some phases are noops.

## Phase 7: activate only after prepare succeeds

After runtime prepare succeeds:

```text
current -> releases/<release_id>
```

Activation should be done by BonesRemote/root.

Then restart/reload the relevant service.

Conceptual order for Laravel v1:

```text
build artifact
promote release
wire shared paths
runtime prepare as foo
activate current
restart service
```

The important tradeoff: migrations may mutate the DB before activation. That is normal in many deployments, but it means rollback is code rollback, not database rollback. Document this clearly.

## Phase 8: add a root-owned site registry

Privileged operations should not trust git-owned config.

Add:

```text
/etc/bonesdeploy/sites/foo.toml
```

This should be the trusted source for:

```text
site name
repo path
site root
runtime user/group
shared path
release path
service names
framework
allowed deploy paths
```

Repo-owned config can still describe app preferences, but privileged decisions should come from the root-owned registry.

## Phase 9: update the hook model

Current problem: `git` currently does too much.

Target:

```text
git post-receive hook:
  identify repo + revision
  call narrow sudo bonesremote deploy for that registered site
```

Not:

```text
git checks out code
git runs composer/npm
git writes releases
git writes shared
git restarts services
```

`git` becomes a trigger, not a deploy user.

## Phase 10: keep v1 intentionally scoped

For v1, do not solve everything.

Include:

```text
one per-site foo user/group
git removed from site groups
root-owned site registry
ephemeral Podman builds
no shared/.env mounted into build
artifact promotion
root:foo sealed releases
foo-owned shared state
Laravel shared symlinks
runtime prepare phase as foo
root activation/restart
```

Defer:

```text
perfect rollback of database migrations
per-site build users
per-site ingress users
full systemd sandboxing
per-framework advanced queue/octane/reverb handling
www-data global socket isolation
complex ACL/group designs
```

## The one-sentence implementation path

First make `git` ingress-only, then make `foo` the only site identity, then move builds into temporary Podman containers with no runtime mounts, then promote only the artifact into root-owned sealed releases, then run Laravel runtime commands as `foo` against shared state, then activate through BonesRemote.

That is the cleanest v1 we have arrived at.
