# Distribution Python Runtime

<!-- Merged from fix/python-runtime-toolchain -->

## Distribution Python Runtime

### Request

Apply the distribution-runtime approach used for Ruby to Django so provisioning
does not compile CPython on production servers.

### Problem

BonesInfra downloads and compiles an optimized CPython release with parallel
`make` on every Django host. That can exhaust the memory available on small
production servers. Django artifacts also contain locally installed Python
packages and wrappers tied to the managed interpreter.

### Definitions

**Distribution Python:** The `python3` interpreter and supporting packages from
the supported Debian or Ubuntu APT repositories.

**Legacy Python version:** The accepted but unused `python_version` value from
existing project configuration and remote descriptors.

**Target virtualenv:** The release-owned `.venv` created with distribution
Python while preparing a staged release.

### Desired Outcome

Django host provisioning installs Python through APT without compiling CPython.
The local build does not install Python dependencies. Before activation, the
target host creates `.venv`, installs `requirements.txt`, validates Django, runs
migrations unless skipped, and collects static files.

### Scope

- Django production Python provisioning and runtime paths.
- Django artifact cleanup and target-side dependency installation.
- AppArmor, remote descriptor compatibility, tests, generated wheel, and
  documentation affected by the runtime boundary.

### Constraints

- Supported hosts remain Debian 12+ and Ubuntu 24.04+ on `x86_64`.
- Existing `python_version` values remain readable but do not select an
  interpreter and are not generated for new projects.
- Prepare scripts continue to run as the home-less site runtime user.
- Full E2E tests are not run unless explicitly requested.

### Exclusions

- Installing non-distribution Python versions on production hosts.
- Adding package managers or supporting dependency manifests other than the
  existing `requirements.txt` contract.
- Migrating old managed CPython installations.


## Distribution Python Runtime Plan

### Current Behavior

`PythonRuntime` installs CPython build dependencies and invokes
`install-python.sh`, which downloads, verifies, configures with PGO/LTO, and
compiles a pinned CPython patch release under `/opt/bonesdeploy/python`.

The local Django build installs `requirements.txt` into `.python-packages` and
creates `.venv/bin` wrappers targeting that managed production interpreter.
Remote prepare assumes those packaged dependencies and wrappers already exist.

### Intended Behavior

BonesInfra installs distribution Python, venv support, development headers, and
native package build prerequisites through APT. Django artifacts contain source
and requirements but no local dependency tree or virtualenv. Remote prepare
creates `.venv` with `/usr/bin/python3`, installs requirements before all Django
commands, and then performs validation, migrations, and static collection.

### Approach

Replace the Python source installer with one APT package operation and return
`/usr/bin/python3` as the production executable. Simplify the local build so it
does not require Python or resolve dependencies; it removes stale
`.python-packages` and `.venv` outputs before packaging. Move virtualenv creation
and pip installation into Django prepare before validation and migration skip
handling.

Remove managed-Python AppArmor paths and permit distribution Python execution.
Stop serializing `python_version` into remote descriptors while accepting and
discarding the old native field for compatibility, matching `ruby_version`.

### Responsibilities And Boundaries

- BonesInfra `PythonRuntime` owns host APT packages and the production Python
  executable path.
- The local Django build owns cleanup of stale dependency outputs.
- Django remote prepare owns the release virtualenv and production dependency
  installation.
- The Django AppArmor profile owns distribution-interpreter execution access.
- `bonesdeploy-core` owns compatibility at the remote descriptor boundary.

### Affected Areas

- Python runtime provisioning and its deleted source installer.
- Django build and prepare assets, runtime setup, and AppArmor template.
- Python and Rust regression tests and the embedded BonesInfra wheel.
- README, context, architecture, and this planning record.

### Decisions

- Use unversioned distribution package names and `/usr/bin/python3` so the
  supported host release selects Python.
- Use a release-owned virtualenv so pip does not modify externally managed
  distribution Python and rollback retains each release's dependencies.
- Retain local dependency resolution as a build-time compatibility check, then
  delete its output rather than shipping host-incompatible native extensions.
- Install dependencies before migration skip handling because skipping database
  migrations must not produce an unstartable release.

### Risks

- Applications requiring a newer Python than the host distribution will fail;
  documentation must make host compatibility explicit.
- Native Python dependencies compile on the target and require adequate APT
  headers; preserve the useful general and PostgreSQL build prerequisites.
- A missing `requirements.txt` still produces a clear Gunicorn validation error,
  preserving the existing application contract.

### Validation

- Python tests prove one APT operation installs distribution Python packages and
  no source installer or managed path remains.
- Asset and prepare tests prove local dependencies are removed and target venv,
  pip installation, validation, migration skip, migrations, and static collection
  occur in safe order.
- Remote config tests prove new descriptors omit Python while old native
  `python_version` fields deserialize and are discarded.
- Rebuild the embedded wheel; run focused tests, all non-E2E Python and Rust
  suites, Ruff, Clippy, Rustfmt, Shfmt, and `git diff --check`.


## Distribution Python Runtime Tasks

### Implementation

- [x] Replace production CPython source compilation with distribution Python and
  native dependency build packages installed through APT.
- [x] Remove managed Python runtime paths and the obsolete source installer.
- [x] Remove local Django dependency outputs from artifacts and create the target
  virtualenv before installing production requirements during prepare.
- [x] Drop `python_version` from new remote descriptors while reading and
  discarding descriptors that previously included it.

### Validation

- [x] Update focused Python and Rust coverage for provisioning, AppArmor,
  artifacts, prepare ordering, and descriptor compatibility.
- [x] Rebuild the embedded BonesInfra wheel and run all required non-E2E checks.

### Completion

- [x] Update runtime and deployment documentation for distribution Python,
  target-side pip installation, and host-version compatibility.
- [x] Review the final diff for stale managed-Python paths and unrelated changes.

### Completion Notes

Production Django now uses distribution Python from APT. Local builds do not
install Python or dependencies and remove stale dependency output from artifacts.
Prepare creates a release-owned virtualenv before installing requirements with
isolated, cacheless pip. Migration skip handling still occurs only after
dependencies and validation are ready.

The embedded wheel was rebuilt. Ruff checks, 516 Python tests, focused runtime
tests, the workspace Rust suite excluding E2E, Clippy, Rustfmt, Shfmt, and
`git diff --check` passed. Full E2E was not run.
