# Distribution Python Runtime Plan

## Current Behavior

`PythonRuntime` installs CPython build dependencies and invokes
`install-python.sh`, which downloads, verifies, configures with PGO/LTO, and
compiles a pinned CPython patch release under `/opt/bonesdeploy/python`.

The local Django build installs `requirements.txt` into `.python-packages` and
creates `.venv/bin` wrappers targeting that managed production interpreter.
Remote prepare assumes those packaged dependencies and wrappers already exist.

## Intended Behavior

Superseded by `feature/mise-runtimes`. Pinned mise installs the exact configured
Python precompiled-only for local builds and into the root-owned production
store. Django artifacts retain their dependency tree and site-linked launchers.
Prepare rejects runtime mismatch before validation, migrations, or static
collection and does not install packages.

## Approach

No further work is planned under this record. `feature/mise-runtimes` owns the
replacement implementation and validation.

## Responsibilities And Boundaries

- BonesInfra `PythonRuntime` owns host APT packages and the production Python
  executable path.
- The local Django build owns cleanup of stale dependency outputs.
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

The distribution-runtime decision is superseded. `feature/mise-runtimes` makes
the exact configured Python version authoritative for both build and production,
uses pinned mise `2026.10.0` with precompiled-only installation, and exposes a
stable site link. Django packages dependencies locally; production never invokes
pip or compiles application dependencies.

## Risks

- The selected Python release must have a mise precompiled artifact.
- A runtime or artifact identity mismatch must fail before activation and leave
  the current release running.

## Validation

Validation is now defined by `feature/mise-runtimes`, including managed-runtime,
packaged-dependency, direct-site-link, and human-run E2E checks.
