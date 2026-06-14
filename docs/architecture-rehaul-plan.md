Here is the simpler, opinionated plan I would hand to the code agent. This chooses one model and avoids branching.

# BonesDeploy Rehaul Plan: Simple Stable Ownership Model

## Goal

Rework BonesDeploy so normal deployments never flip ownership.

The new rule is:

```text
git owns deployment.
app user owns runtime state.
app user reads deployed releases through a release-read group.
root only provisions OS resources and restarts/reloads services.
```

This replaces the current ownership-flipping model where `bonesremote` temporarily changes ownership to the deploy user, then hardens files back afterward. That model is documented in the README and implemented through recursive `chown` / permission hardening today.   

No legacy mode. No compatibility path. No per-site deploy users in v1.

---

# Fixed Decisions

For a site named `foo`:

```text
deploy user:        git
runtime user:       foo
runtime group:      foo
release-read group: foo-release
web server user:    www-data
```

Meaning:

```text
git deploys code.
foo runs the app.
foo owns mutable runtime state.
foo-release lets foo read deployed releases.
www-data only routes traffic or reads public assets.
root provisions global OS resources.
```

Group membership:

```text
foo is in foo-release
foo is not in git
git is not in foo
git does not need to be in foo-release
```

Do not create `foo-deploy` in v1. That is scope creep.

---

# Filesystem Layout

Use this layout:

```text
/srv/sites/foo/
├── releases/                       git:foo-release 2751
│   ├── 20260613_180000/
│   ├── 20260613_190000/
│   └── 20260613_200000/
├── build/                          git:git 0700
│   └── workspace/
├── current -> releases/20260613_200000
└── shared/                         foo:foo 0711
    ├── .env                        foo:foo 0640
    ├── storage/                    foo:foo 0750
    ├── cache/                      foo:foo 0750
    ├── uploads/                    foo:foo 0750
    └── public/                     foo:foo 0755
```

Why these permissions:

```text
releases/  git can deploy, foo can read, nginx can traverse to public assets
build/     only git can see half-built workspaces
shared/    foo owns runtime state; others can only traverse if path is known
.env       private to runtime user
public/    intentionally public if nginx serves uploads/assets
```

Release files should default to:

```text
private release dirs:   git:foo-release 2750
private release files:  git:foo-release 0640
scripts/executables:    git:foo-release 0750
public dirs:            git:foo-release 0755
public files:           git:foo-release 0644
```

The `foo-release` group must never be writable on release code.

---

# Core Ownership Contract

The contract is:

```text
git can:
  create releases
  build code
  update current
  prune old releases

foo can:
  read /srv/sites/foo/current
  run the app
  write declared shared state
  write /run/foo

foo cannot:
  modify releases
  modify current
  modify build/
  modify hooks
  modify deployment scripts
  modify nginx/systemd config

www-data can:
  connect to app sockets
  read current/public if needed

www-data cannot:
  read private app code
  read .env
  own application files
```

This is the whole model.

Activation is just an atomic symlink move:

```text
current.new -> releases/20260613_200000
mv -T current.new current
```

Rollback is the same idea: point `current` back to an older release.

---

# Shared Path Wiring

Shared paths are wired by symlink only.

Example:

```text
release/.env      -> ../../shared/.env
release/storage   -> ../../shared/storage
release/cache     -> ../../shared/cache
release/public/uploads -> ../../shared/public/uploads
```

Rules:

```text
Deploy may create symlinks inside the release.
Deploy may not chown shared paths.
Deploy may not move release content into shared during normal deploy.
Deploy may not create .env during normal deploy.
Deploy may not overwrite runtime state.
```

Shared path creation happens during provisioning or explicit admin setup.

For v1, normal deploy should only validate the path strings:

```text
reject absolute paths
reject ..
reject empty paths
reject paths outside shared/
```

Deep existence and ownership validation of shared targets can be done by privileged `validate-site` / `doctor`, not by the unprivileged deploy flow.

---

# systemd Runtime Model

Generated service units should look like this:

```ini
[Service]
User=foo
Group=foo
SupplementaryGroups=foo-release

WorkingDirectory=/srv/sites/foo/current

NoNewPrivileges=yes
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict

ReadWritePaths=/srv/sites/foo/shared/storage
ReadWritePaths=/srv/sites/foo/shared/cache
ReadWritePaths=/srv/sites/foo/shared/uploads
ReadWritePaths=/srv/sites/foo/shared/public
ReadWritePaths=/run/foo

RuntimeDirectory=foo
RuntimeDirectoryMode=0750
```

Important rule:

```text
The runtime user reads releases through foo-release.
The runtime user writes only shared paths and /run/foo.
```

For PHP-FPM, verify that workers actually have access to `foo-release`. If not, the pool config needs to be adjusted deliberately. Do not guess.

---

# nginx Policy

nginx should not own app code.

nginx can do two things:

```text
1. proxy to app socket
2. serve current/public
```

Public asset policy:

```text
current/public directories: 0755
current/public files:      0644
private app directories:   2750
private app files:         0640
.env:                      0640 foo:foo
```

nginx should not need to read:

```text
app/
vendor/
routes/
config/
storage private data
.env
```

---

Do not encode `www-data` as the app ownership group.

---

# Validation Rules

Add:

```text
bonesremote validate-site --config ...
bonesremote doctor --ownership --config ...
bonesremote doctor --namespaces --config ...
```

Checks:

```text
git exists
foo exists
foo-release exists
foo is in foo-release
/srv/sites/foo is git:foo-release
releases is git:foo-release with setgid
build is git:git 0700
shared is foo:foo
current points into releases
shared symlinks do not escape shared/
runtime writable paths are declared
nginx config path is deterministic
systemd unit path is deterministic
domain is not already claimed
service restart target is site-scoped
```

Critical security test:

```text
sudo -u foo test -r /srv/sites/foo/current/app.php
sudo -u foo test ! -w /srv/sites/foo/current/app.php
sudo -u foo test ! -w /srv/sites/foo/current
sudo -u foo test -w /srv/sites/foo/shared/storage
sudo -u foo test ! -r /srv/sites/bar/current/app.php
```

That test captures the whole point.

---

# Implementation Order

Do it in this order:

```text
1. Change path defaults to /srv/sites.
2. Add explicit deploy/runtime/release-group config.
3. Create the new directory/ownership contract.
4. Make release stage/build run as git without sudo.
5. Make wire create symlinks only.
6. Make activate flip current only.
7. Make prune/rollback operate only under releases.
8. Replace privileged deploy commands with narrow service commands.
9. Update systemd/nginx/PHP-FPM templates.
10. Add doctor/validate checks.
11. Rewrite docs and tests around the new contract.
12. Delete old ownership-flipping code.
```

