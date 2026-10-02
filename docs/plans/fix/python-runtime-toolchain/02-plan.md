# Distribution Python Runtime Plan

## Current Behavior

`PythonRuntime` installs CPython build dependencies and invokes
`install-python.sh`, which downloads, verifies, configures with PGO/LTO, and
compiles a pinned CPython patch release under `/opt/bonesdeploy/python`.

The local Django build installs `requirements.txt` into `.python-packages` and
creates `.venv/bin` wrappers targeting that managed production interpreter.
Remote prepare assumes those packaged dependencies and wrappers already exist.

## Intended Behavior

BonesInfra installs distribution Python, venv support, development headers, and
native package build prerequisites through APT. Django artifacts contain source
and requirements but no local dependency tree or virtualenv. Remote prepare
creates `.venv` with `/usr/bin/python3`, installs requirements before all Django
commands, and then performs validation, migrations, and static collection.

## Approach

Replace the Python source installer with one APT package operation and return
`/usr/bin/python3` as the production executable. Simplify the local build so it
may resolve dependencies with its exact configured Python but removes both
`.python-packages` and `.venv` before packaging. Move virtualenv creation and pip
installation into Django prepare before validation and migration skip handling.

Remove managed-Python AppArmor paths and permit distribution Python execution.
Stop serializing `python_version` into remote descriptors while accepting and
discarding the old native field for compatibility, matching `ruby_version`.

## Responsibilities And Boundaries

- BonesInfra `PythonRuntime` owns host APT packages and the production Python
  executable path.
- The local Django build owns disposable dependency resolution and artifact
  cleanup.
- Django remote prepare owns the release virtualenv and production dependency
  installation.
- The Django AppArmor profile owns distribution-interpreter execution access.
- `bonesdeploy-core` owns compatibility at the remote descriptor boundary.

## Affected Areas

- Python runtime provisioning and its deleted source installer.
- Django build and prepare assets, runtime setup, and AppArmor template.
- Python and Rust regression tests and the embedded BonesInfra wheel.
- README, context, architecture, and this planning record.

## Decisions

- Use unversioned distribution package names and `/usr/bin/python3` so the
  supported host release selects Python.
- Use a release-owned virtualenv so pip does not modify externally managed
  distribution Python and rollback retains each release's dependencies.
- Retain local dependency resolution as a build-time compatibility check, then
  delete its output rather than shipping host-incompatible native extensions.
- Install dependencies before migration skip handling because skipping database
  migrations must not produce an unstartable release.

## Risks

- Applications requiring a newer Python than the host distribution will fail;
  documentation must make host compatibility explicit.
- Native Python dependencies compile on the target and require adequate APT
  headers; preserve the useful general and PostgreSQL build prerequisites.
- A missing `requirements.txt` still produces a clear Gunicorn validation error,
  preserving the existing application contract.

## Validation

- Python tests prove one APT operation installs distribution Python packages and
  no source installer or managed path remains.
- Asset and prepare tests prove local dependencies are removed and target venv,
  pip installation, validation, migration skip, migrations, and static collection
  occur in safe order.
- Remote config tests prove new descriptors omit Python while old native
  `python_version` fields deserialize and are discarded.
- Rebuild the embedded wheel; run focused tests, all non-E2E Python and Rust
  suites, Ruff, Clippy, Rustfmt, Shfmt, and `git diff --check`.
