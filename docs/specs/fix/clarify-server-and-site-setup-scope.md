# Clarify Server And Site Setup Scope

<!-- Merged from fix/clarify-setup-scope -->

## Idea

### Request

Improve the CLI help and agent documentation so agents do not confuse the
host-wide server setup with per-project site setup.

### Problem

The CLI describes `bonesdeploy setup`, `bonesdeploy server setup`, and
`bonesdeploy site setup` without consistently stating their operational scope.
After initialization, the CLI specifically directs every project to run server
setup. On a shared host this encourages redundant concurrent host provisioning,
which can contend on host-wide resources before any sites are provisioned.

### Definitions

**Server setup:** Host-wide provisioning performed once for a shared host.

**Site setup:** Project-scoped provisioning performed once for each project on
that host.

**Combined setup:** `bonesdeploy setup`, a convenience command for a project's
first setup on a fresh host that runs server setup and then site setup.

### Desired outcome

CLI users and agents can determine from command help and bundled guidance that
server setup runs once per host, site setup runs once per project, and additional
projects on a prepared host should run site setup directly.

### Scope

Update setup command help, initialization follow-up output, embedded agent
documentation, and the README. Add command-level regression tests for the
rendered guidance.

### Constraints

Keep existing command names, flags, orchestration, and provisioning behavior.
Use the existing Clap help and integration-test patterns. Do not run e2e tests.

### Exclusions

Do not alter setup idempotency, add concurrency controls, change tunnel
provisioning, or redesign setup orchestration.


## Plan

### Current behavior

`crates/bonesdeploy/src/cli/args.rs` gives setup commands short descriptions
that omit once-per-host and once-per-project scope. The combined setup in
`commands/setup.rs` always runs both phases. `commands/init/mod.rs` directs every
initialized project to `bonesdeploy server setup`, even when its host is already
prepared. Embedded skill documents contain pieces of the correct model, but do
not present a prominent additional-project workflow. README setup instructions
also omit explicit execution frequency.

### Intended behavior

Rendered help names the scope and frequency of each setup command. Initialization
recommends combined setup for a first project while explicitly directing projects
on prepared hosts to site setup. Agent and user documentation show both fresh-host
and additional-project workflows using consistent terminology.

### Approach

Revise existing descriptions rather than adding commands or detection logic.
Keep combined setup as the fresh-host convenience path. Add focused integration
tests that execute the binary and assert the scope wording in root, server, and
site help, plus the two initialization next steps.

### Responsibilities and boundaries

Clap declarations own command help. Initialization owns immediate next-step
guidance. Embedded skill assets own agent operating instructions. README owns
the human setup workflow. Integration tests validate visible CLI contracts.

### Affected areas

- `crates/bonesdeploy/src/cli/args.rs`
- `crates/bonesdeploy/src/commands/init/mod.rs`
- `crates/bonesdeploy/assets/skill/SKILL.md`
- `crates/bonesdeploy/assets/skill/commands.md`
- `crates/bonesdeploy/assets/skill/workflows.md`
- `crates/bonesdeploy/tests/cli.rs`
- `crates/bonesdeploy/tests/init.rs`
- `README.md`

### Decisions

Do not change setup execution semantics because the defect is guidance, not
orchestration. Recommend combined setup only for a project's first setup on a
fresh host. Recommend site setup directly for every additional project on an
already prepared host.

### Risks

Overly terse help could remain ambiguous, while overly detailed summaries could
make root help noisy. Exact-string tests could become brittle, so assertions
will target the essential scope phrases rather than entire help snapshots.

### Validation

Run focused BonesDeploy CLI and init tests and manually inspect rendered help.
Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`. Review the final diff for
consistent scope terminology and unintended changes. Do not run e2e tests.


## Tasks

### Implementation

- [x] Make combined, server, and site setup help state their distinct scopes.
- [x] Make initialization output distinguish fresh-host and prepared-host next steps.
- [x] Align embedded agent docs and README workflows with once-per-host and once-per-project setup.
- [x] Add CLI integration coverage for rendered setup help and initialization guidance.

### Validation

- [x] Run focused BonesDeploy CLI and init tests successfully.
- [x] Inspect root, server setup, and site setup help for the required scope wording.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without warnings or errors.

### Completion

- [x] Review the final diff for consistent terminology and unintended changes.

### Completion notes

Implemented without changing setup orchestration or provisioning behavior.
`cargo test -p bonesdeploy --test cli --test init` passed 17 tests. Rendered
help distinguishes the combined fresh-host path from once-per-host server setup
and once-per-project site setup. `cargo fmt`, `cargo clippy`, `shfmt -w .`, and
`git diff --check` completed successfully. E2E tests were not run.


## Manual setup results: 2026-09-26

### Purpose

This records a sequential manual setup pass against a freshly rebuilt server.
The pass was observational: failures were not retried, diagnosed on the host, or
remediated.

### Test context

- BonesDeploy binary: `/home/alex/Work/rust/bonesdeploy/target/debug/bonesdeploy`
- Reported version: `bonesdeploy 0.8.7`
- Target: `root@45.33.96.98`
- Project roots: `/home/alex/Work/tries/bones_testing/<project>`
- Server setup working directory: `laraveltest`
- Commands were run sequentially, never concurrently.
- Django and Rails were intentionally run last.

The commands were:

```sh
bonesdeploy server setup --yes
bonesdeploy site setup --yes
```

Each site command was run from its project's root directory. The site order was:

1. `laraveltest`
2. `nexttest`
3. `nexttest-static`
4. `nuxttest`
5. `nuxttest-static`
6. `sveltetest`
7. `vuetest`
8. `djangotest`
9. `railstest`

### Results

| Scope | Result | Last completed phase | Failure |
| --- | --- | --- | --- |
| Server | Passed | `server doctor` | None |
| `laraveltest` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `nexttest` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `nexttest-static` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `nuxttest` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `nuxttest-static` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `sveltetest` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `vuetest` | Failed | Runtime apply | Missing `bonesremote decommission` subcommand |
| `djangotest` | Failed | Site base apply | CPython archive creation returned `ENOSPC` in the `/tmp` build tree |
| `railstest` | Failed | Site base apply | Ruby archive creation returned `ENOSPC` in the `/tmp` build tree |

The "Last completed phase" column uses the CLI's phase boundaries. For the first
seven projects, both site-base and runtime provisioning printed `deploy complete`,
but the overall site setup still failed before site doctor and before the final
`Site setup complete` message. For Django and Rails, site-base provisioning
completed, then runtime provisioning failed while installing the language runtime.

### Server setup

`bonesdeploy server setup --yes` completed all baseline provisioning. Its final
output was:

```text
bonesdeploy server doctor Checking server baseline...
✓ Server baseline checks passed.
✓ Server baseline is ready.
```

The setup reported `Install bonesremote binary Success`. Server setup passes the
local package version to BonesInfra as the requested BonesRemote version in
`crates/bonesdeploy/src/commands/server/setup.rs`.

### Issue 1: installed BonesRemote lacks `decommission`

#### Affected projects

`laraveltest`, `nexttest`, `nexttest-static`, `nuxttest`, `nuxttest-static`,
`sveltetest`, and `vuetest` all failed identically.

#### Observed sequence

For every affected project:

1. The initial server doctor passed.
2. Site-base provisioning printed `deploy complete`.
3. Framework runtime provisioning printed `deploy complete`.
4. The post-runtime reactivation command failed.
5. Site doctor did not run, and the CLI did not print `Site setup complete`.

The exact failing command and error, with only the site name varying, were:

```text
Remote command failed: sudo -n bonesremote decommission reactivate --site '<site>'
stderr:
error: unrecognized subcommand 'decommission'

Usage: bonesremote <COMMAND>
```

#### Relevant code

- `crates/bonesdeploy/src/commands/site/setup.rs:25-27` connects to the server
  after runtime apply and executes `bonesremote decommission reactivate`.
- `crates/bonesremote/src/cli/dispatch.rs:17-21` shows that the current source
  tree defines `decommission`, including `reactivate`.
- `crates/bonesdeploy/src/commands/server/setup.rs` requests a BonesRemote release
  matching `env!("CARGO_PKG_VERSION")` during server setup.

#### Working hypothesis

The locally built BonesDeploy executable and the downloaded `0.8.7` BonesRemote
release may not contain matching command sets even though they share the same
reported package version. This is not confirmed. The server binary's version,
help output, checksum, and release provenance were not inspected during this pass.

#### Investigation starting point

1. Capture `sudo -n bonesremote version` and `sudo -n bonesremote --help` from a
   freshly provisioned host.
2. Compare the installed binary checksum and command list with the published
   `0.8.7` release artifact.
3. Determine which commit produced the local BonesDeploy executable and whether
   its `site setup` expects post-`0.8.7` BonesRemote behavior without a version
   change.
4. After restoring version compatibility, rerun one affected site and confirm
   that reactivation, site doctor, and the final success message execute.

### Issue 2: CPython build cannot complete in `/tmp`

#### Affected project

`djangotest`

#### Observed sequence

1. The initial server doctor passed.
2. Site-base provisioning printed `deploy complete`.
3. Runtime provisioning installed the CPython build dependencies.
4. The Python 3.14.7 source build ran under
   `/tmp/tmp.BvQbZ1NgdF/Python-3.14.7`.
5. The build failed while creating `libpython3.14.a`.

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

#### Relevant code

`crates/bonesinfra/python/src/bonesinfra/assets/scripts/install-python.sh`:

- Creates the source/build directory with `mktemp -d` at line 17.
- Cleans it on exit with an `EXIT` trap at line 18.
- Configures CPython with `--enable-optimizations --with-lto` at line 25.
- Runs `make -j"$(nproc)"` at line 26.

#### Investigation starting point

No filesystem capacity or inode checks were run during this pass, so this report
does not assert why the filesystem serving `/tmp` returned `ENOSPC`.

1. Before reproducing, record `findmnt /tmp`, `df -h /tmp`, and `df -i /tmp`.
2. During the build, measure the build tree and filesystem usage before archive
   creation.
3. Confirm whether the failure is byte capacity, inode capacity, or another
   limit reported as `ENOSPC`.
4. Compare the measured requirement with the supported minimum server profile.
5. Rerun `djangotest` only after selecting the intended installer behavior for
   supported hosts.

### Issue 3: Ruby build cannot complete in `/tmp`

#### Affected project

`railstest`

#### Observed sequence

1. The initial server doctor passed.
2. Site-base provisioning printed `deploy complete`.
3. Runtime provisioning installed Ruby build dependencies.
4. The Ruby 3.4.8 source build ran under `/tmp/tmp.roXYs5E6iw/ruby-3.4.8`.
5. The build failed while creating `libruby-static.a`.

The terminal failure was:

```text
/usr/bin/ar: unable to copy file 'libruby-static.a'; reason: No space left on device
make: *** [Makefile:323: libruby-static.a] Error 1
make: *** Deleting file 'libruby-static.a'
Error: executed 2 commands

deploy failed
bonesinfra runtime apply --request-stdin failed
```

#### Relevant code

`crates/bonesinfra/python/src/bonesinfra/assets/scripts/install-ruby.sh`:

- Creates the source/build directory with `mktemp -d` at line 44.
- Cleans it on exit with an `EXIT` trap at line 45.
- Builds Ruby from source with `make -j "$(nproc)"` at line 59.
- Installs under `/opt/bonesdeploy/ruby/3.4.8` only after a successful build.

#### Investigation starting point

Use the same capacity and inode measurements described for the CPython failure.
The two failures share a source-build directory under `/tmp` and fail during
static archive creation, but they should remain separate reproductions until the
underlying resource limit is confirmed.

### Resulting server state

- The server baseline is complete and passed doctor.
- Site-base provisioning completed for all nine projects.
- Runtime provisioning completed for the first seven projects, but their overall
  setup commands failed before reactivation and site doctor.
- Django runtime provisioning stopped during CPython installation.
- Rails runtime provisioning stopped during Ruby installation.
- No failed command was retried.
- No manual server changes or remediation commands were performed after failures.

The complete Django and Rails terminal captures from this run are available on
the test machine at:

```text
/home/alex/.local/share/opencode/tool-output/tool_0e07e8840001uxt5I5xSeeE4dy
/home/alex/.local/share/opencode/tool-output/tool_0e08413d7001aGzCQFmKp34GOW
```
