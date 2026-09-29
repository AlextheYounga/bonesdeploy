# Manual setup results: 2026-09-28

## Purpose

This records a sequential manual setup pass against a freshly rebuilt server
and the subsequent SSL command pass. The setup pass was observational: failed
commands were not retried, investigated on the host, or remediated.

## Test context

- BonesDeploy binary:
  `/home/alex/Work/rust/bonesdeploy/target/debug/bonesdeploy`
- BonesDeploy version: `0.9.1`
- Source branch and commit: `develop` at `dfc8814a`
- Project BonesInfra version after refresh: `0.4.1`
- BonesRemote version requested by server setup: `0.9.1`
- Target: `root@45.33.96.98`
- Project roots: `/home/alex/Work/tries/bones_testing/<project>`
- Server setup working directory: `laraveltest`
- Commands ran sequentially, never concurrently.
- Django and Rails ran last.

The fixtures initially contained `bonesinfra 0.3.4`. Before setup, each fixture
was refreshed through the normal local update path so the pass exercised the
current embedded `bonesinfra 0.4.1` wheel:

```sh
bonesdeploy update --skip-remote
```

Server setup then ran once:

```sh
bonesdeploy server setup --yes
```

Each project ran:

```sh
bonesdeploy site setup --yes
```

The site order was:

1. `laraveltest`
2. `nexttest`
3. `nexttest-static`
4. `nuxttest`
5. `nuxttest-static`
6. `sveltetest`
7. `vuetest`
8. `djangotest`
9. `railstest`

## Results

| Scope | Result | Last completed phase | Failure or notice |
| --- | --- | --- | --- |
| Server | Passed | `server doctor` | None |
| `laraveltest` | Passed | `site doctor` | Waiting for secrets and first push |
| `nexttest` | Passed | `site doctor` | Waiting for secrets and first push |
| `nexttest-static` | Passed | `site doctor` | Waiting for secrets and first push |
| `nuxttest` | Passed | `site doctor` | Waiting for secrets and first push |
| `nuxttest-static` | Passed | `site doctor` | Waiting for secrets and first push |
| `sveltetest` | Passed | `site doctor` | Waiting for secrets and first push |
| `vuetest` | Passed | `site doctor` | Waiting for secrets and first push |
| `djangotest` | Failed | Site base apply | CPython archive creation returned `ENOSPC` in the `/tmp` build tree |
| `railstest` | Failed | Site base apply | Ruby archive creation failed because `ar` could not copy `libruby-static.a` |

The "Last completed phase" column uses the CLI's phase boundaries. Django and
Rails both completed site-base provisioning and then failed during the
"Applying runtime" phase. Their site doctors did not run, and neither command
printed `Site setup complete`.

## Server setup

`bonesdeploy server setup --yes` completed baseline provisioning, installed the
requested BonesRemote release, and ended with:

```text
bonesdeploy server doctor Checking server baseline...
✓ Server baseline checks passed.
✓ Server baseline is ready.
```

No server-setup failure was observed.

## Successful site setups

The first seven projects completed site-base provisioning, runtime
provisioning, reactivation, and site doctor. Each ended with:

```text
• shared environment is missing: /srv/sites/<site>/shared/.env. Run 'bonesdeploy secrets push' first.
• repository has no refs yet. Run 'git push <remote> <branch>' before the first deploy.
✓ remote doctor

• Deployment is provisioned and waiting for the first Git push.

✓ Site setup complete.
```

These are expected pre-deploy notices on the rebuilt server, not setup
failures. The previous `bonesremote decommission` compatibility failure did not
recur with BonesDeploy and BonesRemote `0.9.1`.

## Issue 1: CPython build exhausts its filesystem

### Affected project

`djangotest`

### Observed sequence

1. Server readiness passed.
2. Site-base provisioning printed `deploy complete`.
3. Runtime provisioning downloaded and configured Python `3.14.7`.
4. The optimized source build ran under
   `/tmp/tmp.eIpvaomxOa/Python-3.14.7`.
5. Static archive creation failed while reading
   `Python/instruction_sequence.o`.

The terminal failure was:

```text
ar: libpython3.14.a: error reading Python/instruction_sequence.o: No space left on device
make[2]: *** [Makefile:1176: libpython3.14.a] Error 1
make[1]: *** [Makefile:1008: profile-gen-stamp] Error 2
make: *** [Makefile:1020: profile-run-stamp] Error 2
Error: executed 2 commands

deploy failed
bonesinfra runtime apply --request-stdin failed
```

### Relevant implementation

- `crates/bonesinfra/python/src/bonesinfra/services/languages/python.py`
  dispatches the language installer.
- `crates/bonesinfra/python/src/bonesinfra/assets/scripts/install-python.sh`
  creates the build tree with `mktemp -d`, configures CPython with
  `--enable-optimizations --with-lto`, and runs `make -j"$(nproc)"`.
- The install script removes its temporary tree through an `EXIT` trap, so the
  failed build directory is not expected to remain available.

### Investigation starting point

This exactly reproduces the earlier `2026-09-26` CPython symptom on a rebuilt
server. No capacity or inode diagnostics were run during this pass, so the
report does not assert which filesystem or limit caused `ENOSPC`.

Before the next reproduction, capture `findmnt /tmp`, `df -h /tmp`, and
`df -i /tmp`. During the build, measure the temporary tree and backing
filesystem before archive creation. Compare the measured requirement with the
supported minimum server profile and the effect of CPython PGO/LTO.

The complete command output is currently available at:

```text
/home/alex/.local/share/opencode/tool-output/tool_0e9c2fae7001dupY1KTjEQ24gw
```

## Issue 2: Ruby static archive copy fails

### Affected project

`railstest`

### Observed sequence

1. Server readiness passed.
2. Site-base provisioning printed `deploy complete`.
3. Runtime provisioning installed Ruby build dependencies.
4. Ruby `3.4.8` configured and compiled under
   `/tmp/tmp.08AJ4hXWqu/ruby-3.4.8`.
5. The configure summary reported `MFLAGS = -j1`.
6. Linking the static Ruby archive failed.

The terminal failure was:

```text
linking static-library libruby-static.a
/usr/bin/ar: unable to copy file 'libruby-static.a'; reason: Success
make: *** [Makefile:323: libruby-static.a] Error 1
make: *** Deleting file 'libruby-static.a'
Error: executed 2 commands

deploy failed
bonesinfra runtime apply --request-stdin failed
```

### Relevant implementation

- `crates/bonesinfra/python/src/bonesinfra/services/languages/ruby.py`
  dispatches the language installer.
- `crates/bonesinfra/python/src/bonesinfra/assets/scripts/install-ruby.sh`
  creates the build tree with `mktemp -d`, builds Ruby with
  `make -j "$(nproc)"`, and installs only after a successful build.
- The install script removes its temporary tree through an `EXIT` trap, so the
  failed build directory is not expected to remain available.

### Investigation starting point

The failure occurs at the same archive target as the `2026-09-26` run, but the
reported reason changed from `No space left on device` to `Success`. Do not
assume the causes are identical without capturing host state. No filesystem,
inode, kernel-log, process-limit, or `ar` diagnostics were run in this pass.

Before reproducing, capture the same `/tmp` mount, capacity, and inode data as
for Django. Also record `ar --version`, available file descriptors and process
limits, kernel messages around the failure, and filesystem usage immediately
before `libruby-static.a` is created. Preserve the failing build tree for direct
inspection if the installer is deliberately instrumented in a separate
investigation.

The complete command output is currently available at:

```text
/home/alex/.local/share/opencode/tool-output/tool_0e9e14480001Tt2Irofpyg75bd
```

## Additional update observation

Every `bonesdeploy update --skip-remote` completed and installed the expected
`bonesinfra 0.4.1` project wheel. The release-source clone also emitted this
warning before checking out the correct `v0.9.1` commit:

```text
warning: refs/tags/v0.9.1 f83f6289333ae7bdc39ebfbdd1ed0f46e45a783e is not a commit!
Note: switching to '6d35140c9b8a4d51830e5cb8c8980eedf4996a9d'.
```

The warning did not make any update fail. It is recorded so a future update
investigation does not need to rediscover the exact tag and checked-out commit.

## SSL command pass

After recording the setup results, the SSL command was run sequentially in the
seven projects whose site setup and doctor had passed:

```sh
bonesdeploy site ssl --yes
```

| Project | Result | Server contacted | Observed output |
| --- | --- | --- | --- |
| `laraveltest` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |
| `nexttest` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |
| `nexttest-static` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |
| `nuxttest` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |
| `nuxttest-static` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |
| `sveltetest` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |
| `vuetest` | Blocked locally | No | `SSL domain is missing. Pass --domain or set DOMAIN in root .env` |

The CLI validates `DOMAIN` before `EMAIL`, so this pass establishes only that
all seven root `.env` files lack a domain. It does not establish whether their
email values are configured. Validation returned before constructing or
running the BonesInfra SSL request, so no router activation, HTTP-01 challenge,
certificate request, remote mutation, or local `SSL_ENABLED` update occurred.

`djangotest` and `railstest` were not eligible for this first pass because their
site runtime setup and doctor had not completed.

After real DNS names and a Let's Encrypt registration email were provided, SSL
was run again with explicit arguments:

```sh
bonesdeploy site ssl --yes --domain <site>.bonesdeploy.com --email <registration-email>
```

All nine commands completed the full SSL workflow: challenge-router deployment,
Nginx validation and reload, certificate issuance, SSL-router deployment,
another Nginx validation and reload, and etckeeper recording. Each command
printed `HTTPS configured.` and persisted its domain, email, and enabled SSL
state locally.

| Project | Domain | SSL command | HTTPS check | TLS verification |
| --- | --- | --- | --- | --- |
| `laraveltest` | `laravel.bonesdeploy.com` | Passed | `200` | Passed |
| `nexttest` | `next.bonesdeploy.com` | Passed | `200` | Passed |
| `nexttest-static` | `next-static.bonesdeploy.com` | Passed | `200` | Passed |
| `nuxttest` | `nuxt.bonesdeploy.com` | Passed | `200` | Passed |
| `nuxttest-static` | `nuxt-static.bonesdeploy.com` | Passed | `200` | Passed |
| `sveltetest` | `svelte.bonesdeploy.com` | Passed | `200` | Passed |
| `vuetest` | `vue.bonesdeploy.com` | Passed | `200` | Passed |
| `djangotest` | `django.bonesdeploy.com` | Passed | `502` | Passed |
| `railstest` | `rails.bonesdeploy.com` | Passed | `502` | Passed |

The public checks used `curl` against each HTTPS root. A curl
`ssl_verify_result` of `0` confirmed certificate verification for every domain.
The Django and Rails `502` responses are consistent with their earlier failed
runtime provisioning: TLS and the public Nginx router are active, but their
application upstreams are unavailable.

## First deployment pass

Each project was processed sequentially. The requested Git push ran first,
followed by:

```sh
bonesdeploy deploy
```

The seven projects with existing commits successfully created their production
branches on the rebuilt server. Every deployment then exported the pushed
revision and failed in the first build script with the same version-validation
error:

```text
[bonesdeploy] Node requires an exact pinned version. Set NODE_VERSION or use .node-version, .nvmrc, .tool-versions, or package.json volta.
```

| Project | Pushed branch | Pushed revision | Push | Deployment |
| --- | --- | --- | --- | --- |
| `laraveltest` | `master` | `8a87c82c` | Passed | Failed: Node version is not pinned |
| `nexttest` | `main` | `850645be` | Passed | Failed: Node version is not pinned |
| `nexttest-static` | `main` | `b93617e4` | Passed | Failed: Node version is not pinned |
| `nuxttest` | `main` | `f55c6a53` | Passed | Failed: Node version is not pinned |
| `nuxttest-static` | `main` | `f8243a4e` | Passed | Failed: Node version is not pinned |
| `sveltetest` | `main` | `40701cfc` | Passed | Failed: Node version is not pinned |
| `vuetest` | `main` | `f5022584` | Passed | Failed: Node version is not pinned |
| `djangotest` | `main` | None | Failed: local branch has no commits | Failed: remote revision `main` does not exist |
| `railstest` | `main` | None | Failed: local branch has no commits | Failed: remote revision `main` does not exist |

For each of the first seven projects, BonesRemote removed the failed release and
cleared staged release state. None reached release activation. The Node check
failed before any later build or secrets requirement could be evaluated.

The pushed revisions do not include the local BonesInfra refresh. Those seven
worktrees still contain the uncommitted replacement of the `0.3.4` wheel with
`0.4.1` and generated template changes. No commit was created because the
deployment request authorized pushing and deploying, not committing unrelated
or generated worktree changes.

Django and Rails remain entirely uncommitted local repositories. Their pushes
returned `src refspec main does not match any`; their deployment attempts then
returned:

```text
Failed to resolve source revision 'main' to a commit
fatal: Needed a single revision
```

No deployment issue was remediated or retried during this pass.

## Deployment retry with pinned Node

The `fix/default-node-version` branch was completed at `f2e3311a` and the local
BonesDeploy debug binary was rebuilt. Because existing `.env.build` files are
application-owned and intentionally preserved, all nine fixtures were updated
explicitly to pin:

```text
NODE_VERSION=24.19.0
```

The seven existing fixtures committed that pin together with their pending
BonesInfra `0.4.1` refresh. Initial commits were created for Django and Rails.
Pushes and deployments then ran sequentially until the server became
unreachable.

| Project | Local revision | Push | Deployment retry |
| --- | --- | --- | --- |
| `laraveltest` | `b9411e9` | Passed | Failed after Node setup: no `package-lock.json` or `npm-shrinkwrap.json` |
| `nexttest` | `9238845` | Passed | Failed during `next build` with exit status 1 and no additional application error |
| `nexttest-static` | `ad3c9e3` | Passed | Blocked before staging: rootless Podman namespace unhealthy |
| `nuxttest` | `ebb238e` | Passed | Blocked before staging: rootless Podman namespace unhealthy |
| `nuxttest-static` | `2166178` | Timed out | Not run |
| `sveltetest` | `4c919d7` | Not attempted | Not run |
| `vuetest` | `83ab99e` | Not attempted | Not run |
| `djangotest` | `34091ca` | Not attempted | Not run |
| `railstest` | `92ac8e3` | Not attempted | Not run |

Laravel proved the Node fix at the live build boundary. Its release installed
and reported Node `v24.19.0`, then proceeded through PHP and Composer dependency
installation before reaching the later frontend lockfile check. The failed
release was removed and staged state was cleared.

Next also installed and reported Node `v24.19.0`, installed npm dependencies,
and entered the Next.js production build. The build command then exited with
status 1 immediately after `Creating an optimized production build ...`, with
no more specific error in the streamed output. Its failed release was removed
and staged state was cleared.

Next Static and Nuxt then failed readiness before staging:

```text
Rootless Podman is not ready for <site>-build. Its user session or Podman namespace is unhealthy; repair it before deploying.
```

The following Nuxt Static Git push timed out. Immediate and 30-second-delayed
checks found SSH timing out during banner exchange and HTTPS connections timing
out. No later push or deployment was attempted, and no server remediation or
reboot was performed. This retry remains incomplete until the host is reachable
and its resource and rootless Podman state can be inspected.

## Resulting server state

- The server baseline completed and passed doctor.
- Seven production repositories now contain their first pushed branch, but no
  project has activated a release.
- Django site-base state exists, but Python runtime provisioning did not finish.
- Rails site-base state exists, but Ruby runtime provisioning did not finish.
- No failed setup command was retried.
- No manual remediation or diagnostic commands were run on the server.
- All nine domains issued publicly verified certificates before the outage.
- Before the outage, the seven completed runtimes responded over HTTPS with
  status `200`; Django and Rails responded with status `502` because their
  application runtimes did not finish provisioning.
- The retry proved Node `24.19.0` is selected during real release builds.
- Django and Rails now have local initial commits, but they have not been pushed.
- The server became unreachable before the retry matrix could complete.
