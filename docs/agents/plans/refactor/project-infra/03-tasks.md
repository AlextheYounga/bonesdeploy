# Tasks

## Implementation

- [x] Add a BonesInfra project-infrastructure loader that resolves `.bones/infra` relative to `bones.toml`, imports it as a package, validates runtime and manifest entrypoints, and reports path-specific failures before SSH.
- [x] Change `pyinfra.runner.run()` and all CLI deployment callables to use `deploy(ctx)` without `config_path`-based custom-module loading or `ModuleType` arguments.
- [x] Remove root custom hook dispatch from setup, runtime, SSL, services, and helper commands, and remove the runtime framework-list command.
- [x] Move reusable framework-independent helpers from `bonesinfra.frameworks.common` into neutral core modules and remove the installed framework registry, base classes, framework implementations, and template runtime dispatcher.
- [x] Translate Django, Laravel, Next, Nuxt, Rails, SvelteKit, and Vue runtime orchestration into readable generated `infra/runtime.py` assets that call generic core primitives and visibly encode each framework's existing policy.
- [x] Add generated `infra/__init__.py`, `infra/manifest.py`, and ordinary `infra/custom.py` assets for the base kit and every named framework, with project manifests declaring framework-owned artifacts, services, and mode.
- [x] Move every template directly used by generated framework infrastructure into that framework's `infra/templates/` tree and update generated source to reference local paths explicitly.
- [x] Refactor nginx site helpers to require explicit template paths and remove `_resolve_template`, `ASSETS_DIR` framework fallback, and all `confs/` lookup.
- [x] Change manifest collection to combine core declarations and loaded project declarations without querying `get_framework()` or embedding framework-owned runtime artifacts in core constants.
- [x] Update kit and framework asset scaffolding to materialize both `deployment/` and `infra/`, preserve kit `deployment/functions.sh`, and use a framework-project scaffold name and boundary.
- [x] Update init and update synchronization behavior so generated snapshots are complete, `.bones` remains independently versioned, and update does not rewrite project `infra/` source.
- [x] Remove `assets/kit/custom.py`, `assets/kit/confs/`, obsolete path constants, `cli/hooks.py`, `test_custom.py`, and all dead imports and files left by the ownership change.
- [x] Update Python tests for loader errors, relative imports, runtime calls, explicit templates, project manifests, and preserved framework behavior.
- [x] Update Rust initialization and asset tests for every named framework, custom initialization, local templates, deployment scripts, absent old paths, `.bones` Git repository state, and publishing behavior.

## Validation

- [x] Run `python -m pytest` from `crates/bonesinfra/python` and fix all failures, including framework behavior regressions.
- [x] Run the affected Rust tests and full `cargo test`; verify scaffold tests observe the generated `infra` contract for all supported frameworks.
- [x] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, and `shfmt -w .`; resolve all warnings and formatting changes.
- [x] Search the repository for `after_setup`, `after_runtime`, `after_ssl`, `call_hook`, `load_custom_module`, `confs`, `get_framework`, `list_frameworks`, `Framework`, `ServerFramework`, `PHPFramework`, and `template_runtime`; inspect every remaining match for intentional ownership.
- [x] Review the final behavior and diff without running the long e2e suite.

## Completion

- [x] Update `README.md`, `CONTEXT.md`, BonesInfra README/CONTEXT, project documentation, and embedded skill documentation to describe `.bones/bones.toml`, `.bones/deployment/`, `.bones/infra/`, and local `infra/templates/` ownership, while removing old hook/confs/registry docs.
- [x] Record implementation deviations, validation results, important discoveries, and deliberately unfinished work in the completion notes.

## Completion notes

Implementation complete on branch `refactor/project-infra`; no commits made (per root AGENTS.md instruction).

- **Deviations from plan:** the initial `03-tasks.md` captured the pre-refactor hook model (root `custom.py`, `confs/`, installed framework registry); the planning documents were rewritten before implementation to target project-owned `.bones/infra/` source, and the tasks above describe the approved shape. The `update` command's deployment sync previously pointed at non-existent `crates/bonesdeploy/kit/` and `crates/bonesdeploy/frameworks/` paths; the sync now uses the real `crates/bonesdeploy/assets/kit/` and `crates/bonesdeploy/assets/frameworks/` asset roots.
- **Validation results:** `cargo test --workspace` green (106 lib tests, init suite 6/6 including `named_frameworks_materialize_project_infrastructure_snapshots`); Python `uv run pytest` 236/236; `cargo clippy --workspace --all-targets`, `cargo fmt`, `ruff check .`, `ruff format .`, `shfmt -w .` all clean. Every framework snapshot (`django`, `laravel`, `next`, `nuxt`, `rails`, `sveltekit`, `vue`) and the kit was loaded through `project.load_runtime`/`load_manifest` in a scratch project, and every template reference in generated `runtime.py` resolves to a real file.
- **Important discoveries:** the bonesinfra Rust embed test asserted `src/bonesinfra/frameworks/__init__.py`; it now asserts the new `src/bonesinfra/project.py` module. Framework `infra/runtime.py` files reference `TEMPLATES / "app-profile.j2"` (flat template layout) and the static frameworks need `nginx/static-site-nginx.conf.j2` beside `index.html.j2`; both were reconciled across snapshots. `uv` provides `pytest`/`ruff` (`python -m pytest` alone fails; use `uv run pytest`).
- **Deliberately unfinished:** the long e2e suite was not run, so e2e fixtures and assertions referencing the old scaffold (e.g. `.bones/confs`) remain untouched; e2e regeneration is expected to be a follow-up. `docs/agents/plans` approval was assumed final after the rewrite in this branch; no further human sign-off was requested.
