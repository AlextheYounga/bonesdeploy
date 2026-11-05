# Project Owned Infrastructure

<!-- Merged from refactor/project-infra -->

## Idea

### Request

Move framework-specific server infrastructure out of hidden registry dispatch
while keeping the canonical framework implementations in the BonesInfra Python
package. Generate a readable, editable `.bones/infra/` snapshot for each new
project, and use that project snapshot in preference to the built-in framework.

Remove the root-level `.bones/custom.py` hook system, the `.bones/confs/`
override system, the installed Python framework registry, and all compatibility
behavior for those systems.

### Problem

BonesInfra currently hides Laravel, Rails, Django, Next, Nuxt, SvelteKit, and Vue
deployment policy inside installed framework classes and dispatches through a
framework registry. The refactor moved that policy into BonesDeploy assets,
which makes BonesInfra's framework support fragmented across crates. Runtime
and manifest commands now require project source that BonesInfra cannot
materialize on its own.

The root-level custom hook and `confs/` fallback mechanisms also make project
behavior implicit: core scans for special filenames and silently selects
override templates. This makes infrastructure difficult to inspect, version,
test, and change independently of the installed package.

### Definitions

**BonesInfra core:** The installed `bonesinfra` Python package containing
framework-independent deployment primitives, pyinfra execution, setup, SSL,
database services, manifest inspection, path calculation, shared Linux
operations, canonical framework implementations, and framework scaffold
resources. Core does not use registry dispatch or recognize project
`custom.py` as a hook.

**Project infrastructure:** Trusted local Python source under `.bones/infra/`,
including `runtime.py`, `manifest.py`, `custom.py`, supporting modules, and
every template directly used by that source. It is the project's editable
infrastructure implementation.

**Canonical framework implementation:** A built-in framework runtime, manifest,
supporting Python modules, and templates maintained under the BonesInfra Python
package. It is the default implementation for projects that do not have a
local infrastructure package.

**Runtime entrypoint:** A selected `runtime.py` module with a callable
`deploy(ctx)`. It describes the selected project's server runtime and executes
framework-specific orchestration inside the active pyinfra planning context.

**Manifest entrypoint:** A selected `manifest.py` module with callable
functions `artifacts(ctx)`, `services(ctx)`, and `mode(ctx)`. It declares
framework-owned manifest entries while the core manifest engine remains
responsible for inspection, deduplication, and report rendering.

**Project template:** A template stored with the selected project or canonical
framework implementation and directly selected by its runtime source. Core does
not search packaged defaults or `confs/` paths implicitly.

**Vendored project snapshot:** A complete `.bones/infra/` package copied from a
canonical framework implementation. It is project-owned after materialization;
it may be edited and is not silently replaced by BonesInfra or BonesDeploy
updates.

### Desired outcome

Newly initialized projects contain a real vendored `.bones/infra` Python package
and no root `.bones/custom.py` or `.bones/confs/`. Named framework projects
contain readable runtime, manifest, custom, supporting modules, and required
local template files; custom projects contain a minimal usable infrastructure
package.

When the complete local package is absent, runtime and manifest commands load
the selected canonical BonesInfra framework. When the local package exists,
both commands validate and load it before opening SSH and never fall back to the
built-in framework. Invalid or incomplete local infrastructure fails with the
relevant path before any SSH connection.

Supported framework deployments retain their current package installation,
build/runtime mode, nginx, systemd, AppArmor, socket/port, validation,
writable-path, placeholder, and manifest behavior while making those decisions
visible in canonical BonesInfra source and editable project snapshots.

### Scope

The change includes:

- Canonical framework orchestration and directly used framework templates in
  BonesInfra, plus materialization of those resources into vendored project
  snapshots.
- A project infrastructure loader with strict whole-package local precedence,
  built-in framework fallback only when local infrastructure is absent, and
  explicit validation of runtime and manifest entrypoints.
- Generated `infra/` scaffolds for the base kit and every supported named
  framework: Django, Laravel, Next, Nuxt, Rails, SvelteKit, and Vue.
- Removal of hook dispatch, root-level custom loading, `confs/` lookup, the
  installed framework runtime registry, and `bonesinfra runtime list`.
- Explicit template paths in generic nginx helpers and neutral core helpers for
  operations reused by framework implementations.
- Framework-owned manifest declarations combined by the generic core inspector.
- Rust initialization/materialization tests, Python built-in and vendored
  loader/runtime/template/manifest tests, behavior-preservation coverage, and
  documentation updates.

### Constraints

Project infrastructure remains trusted arbitrary local Python and may import the
standard library, pyinfra, and installed BonesInfra core modules. Built-in
framework implementations use the same installed BonesInfra package. No
dependency-installation system is introduced.

`.bones/deployment/` remains separate from `.bones/infra/`: deployment scripts
serve the bonesremote release/build lifecycle, while infrastructure source
describes server/runtime infrastructure. The `.bones` directory remains its own
Git repository and existing publishing behavior remains in use.

The existing manifest tuple formats are retained where practical. Framework
deployment semantics are preserved rather than redesigned. Old projects
without `.bones/infra/` use the selected canonical framework; a present local
package is never bypassed by compatibility fallback.

Implementation must follow repository conventions, leave runnable focused tests
for non-trivial behavior, run the required Rust/Python/static checks, and not
run the long e2e suite.

### Exclusions

This change does not rename the entire `bonesinfra` Python package to `core`.
It does not add an `infra refresh` command or automatically rewrite existing
vendored snapshots. It does not merge deployment scripts into infrastructure,
redesign bonesremote release workflows, or add project dependency management.
It does not preserve root-level `.bones/custom.py`, `.bones/confs/`, custom hook
names, the installed framework object hierarchy, or a replacement framework
registry. It does not provide partial local-package fallback or combine local
and built-in runtime/manifest implementations.


## Plan

### Current behavior

`bonesinfra.pyinfra.runner.run()` currently receives `config_path` so it can
load root `.bones/custom.py` through `bonesinfra.cli.hooks`, validates that
module before SSH, and passes it as a second argument to deployment callables.
Setup, runtime, SSL, and helper commands retain hook parameters. Runtime loads
`bonesinfra.cli.commands.runtime.template_runtime`, queries
`bonesinfra.frameworks.get_framework()`, and derives nginx/AppArmor policy from
the installed framework object.

The Python package contains `bonesinfra/frameworks/` with base, common, and
seven framework implementations plus a registry and `runtime list`. The
manifest engine stores project-runtime artifacts in `COMMON_ARTIFACTS` and
queries the framework registry for artifacts, services, and mode. Generic nginx
site rendering resolves templates by first checking a `confs/` path and then
the installed asset directory.

Rust initialization embeds `assets/kit` and selected framework assets. The kit
currently contains root `custom.py` and `confs/`; framework scaffolding replaces
only `deployment/` after the base kit is copied. The `.bones` directory is a
separate Git repository. Update synchronization currently refreshes deployment
assets from source but does not represent project infrastructure. The current
refactor has placed generated framework infrastructure under BonesDeploy assets,
although the Python runtime machinery remains in BonesInfra.

### Intended behavior

BonesInfra owns canonical framework packages and their scaffold resources. It
materializes the selected canonical package into a new project's `.bones/infra/`
through a BonesInfra materialization boundary called by BonesDeploy. The
project loader resolves the local package relative to `bones.toml`.

When the complete local package is absent, the loader imports the selected
canonical framework package from BonesInfra. When the local package exists, the
loader imports that package as a whole and validates runtime and manifest
entrypoints before SSH. A local loader, import, syntax, or contract failure is
fatal and never falls back to the built-in package. Runtime commands call the
selected implementation's `deploy(ctx)` inside the active pyinfra context.
Manifest commands use the selected implementation's artifact, service, and mode
declarations and combine them with genuinely core-owned declarations.

Canonical framework source owns framework policy: language/runtime installation
calls, paths, server mode, socket or TCP choice, nginx and AppArmor settings,
application services, placeholders, validations, and template selection.
Vendored project source is an editable copy of that policy and references its
local templates. `infra/custom.py` is ordinary project code called visibly by
vendored runtime source and is not recognized by BonesInfra.

The base kit provides a minimal no-framework infrastructure package. Named
framework materialization provides the same entrypoint shape plus the complete
selected framework source and required templates. No generated project contains
root `custom.py` or `confs/`.

### Approach

Implement the change in dependency order. First restore canonical framework
modules and resources inside BonesInfra using explicit runtime and manifest
contracts without registry dispatch. Add a BonesInfra materialization boundary
that copies one canonical framework package into a destination. Then add the
project loader's whole-package precedence and fail-fast error contract, and
adapt runner and CLI entrypoints to use it without module-hook plumbing. Move
generic helpers into neutral core modules and make template paths explicit.

Update Rust initialization so deployment assets remain BonesDeploy-owned while
framework infrastructure is requested from BonesInfra and materialized into the
project snapshot. Preserve kit deployment functions. Update synchronization to
refresh only deployment behavior and never rewrite project infrastructure
snapshots. Remove obsolete files, constants, imports, commands, and tests after
the new path is working.

### Responsibilities and boundaries

The project loader belongs in a neutral BonesInfra core module used by the CLI;
it owns local-package detection, built-in framework selection, package import,
callable validation, and fail-fast errors. Canonical framework packages and
materialization belong to BonesInfra. `pyinfra/runner.py` owns SSH setup and
execution-context entry, but not project policy or custom module discovery.

`cli/commands/setup`, `ssl`, `services`, and helpers own only genuinely generic
operations. `cli/commands/runtime` delegates to the selected local or built-in
runtime entrypoint. `manifest.py` owns declaration collection, inspection,
deduplication, and rendering; the selected local or built-in manifest owns
framework declarations.

Canonical framework runtime modules own framework orchestration and canonical
templates. Vendored `.bones/infra/runtime.py` owns the project's editable copy
of that orchestration and local template selection. The corresponding canonical
or vendored manifest owns declarations for paths and services created by that
orchestration. Generic language, nginx, systemd, AppArmor, validation, and path
operations remain in neutral BonesInfra core modules.

BonesInfra scaffold resources and materialization own framework snapshot
creation. BonesDeploy owns framework selection, deployment asset scaffolding,
and invoking that materialization boundary. The `.bones` Git repository and
existing stage/commit/push code remain the publishing boundary.

### Affected areas

Expected affected areas are:

- `crates/bonesinfra/python/src/bonesinfra/`: canonical framework packages and
  templates, materialization boundary, project loader, pyinfra runner, CLI
  commands, manifest engine, neutralized generic helpers, nginx rendering, and
  removal of `cli/hooks.py` and template runtime dispatch.
- `crates/bonesinfra/python/tests/`: canonical framework behavior,
  materialization, built-in/local loader precedence, fail-fast cases, runner
  invocation, explicit nginx templates, project manifest declarations, and
  adapted behavior tests.
- `crates/bonesdeploy/assets/kit/`: deployment assets only; infrastructure
  resources move to BonesInfra.
- `crates/bonesdeploy/assets/frameworks/{django,laravel,next,nuxt,rails,sveltekit,vue}/`:
  deployment assets only; canonical infrastructure source moves to BonesInfra.
- `crates/bonesdeploy/src/infra/assets/{kit,frameworks}.rs` and
  `crates/bonesdeploy/src/commands/init/scaffold.rs`: deployment scaffolding,
  framework selection, and invocation of BonesInfra materialization.
- `crates/bonesdeploy/src/commands/update/sync.rs`: preservation of deployment
  synchronization without rewriting vendored infrastructure.
- `crates/bonesdeploy-core/src/paths.rs`: removal of obsolete `confs`
  constants.
- `crates/bonesdeploy/tests/init.rs` and relevant Rust asset tests:
  materialized layout, framework entrypoints/templates, `.bones` Git repository,
  and publishing expectations.
- `README.md`, `CONTEXT.md`, BonesInfra README/CONTEXT, project docs, and
  embedded skill documentation: canonical framework ownership, vendored
  snapshots, strict precedence, and removal of old architecture references.

### Decisions

1. Keep the `bonesinfra` package name and keep canonical framework policy inside
   that package. Materialize a vendored copy into `.bones/infra/` so projects
   remain readable and independently editable without fragmenting BonesInfra
   ownership.
2. Load runtime and manifest as one selected implementation: the complete local
   package takes precedence, while the selected built-in package is used only
   when local infrastructure is absent.
3. Treat a present local package as authoritative and all-or-nothing. Syntax,
   import, file, or callable errors fail before SSH and never fall back or mix
   with built-in files.
4. Keep runtime and manifest entrypoints as sibling modules in a real package.
   Package imports are the native mechanism for generated and canonical source
   relationships.
5. Keep manifest declaration tuples and the existing inspector/report pipeline.
   Only declaration ownership and implementation selection change.
6. Pass nginx template paths explicitly. A helper cannot know project template
   ownership and must not search packaged defaults or `confs/`.
7. Keep canonical framework source and materialization in BonesInfra. Keep
   deployment build/prepare assets and framework selection in BonesDeploy.
8. Keep canonical and vendored snapshots stable by default. Do not implement an
   update-time infrastructure refresh in this change.

### Risks

Framework behavior can regress while moving class implementations into
canonical function modules and preserving their vendored copies, especially
mode-specific paths, sockets, service units, AppArmor permissions, nginx
configuration, placeholders, and validation order. Built-in and materialized
behavior tests must cover those translations.

The project loader changes import timing and module identity. Relative imports,
missing files, invalid syntax, import failures, and non-callable entrypoints must
produce path-specific errors without opening SSH. Whole-package precedence must
not accidentally mix a local runtime with a built-in manifest.

Materialization can leave a vendored framework source reference pointing at a
missing local file, or canonical and copied source can drift. Materialization
tests and Python syntax/template tests must validate both source locations.

Removing `confs/` and root hooks intentionally breaks those mechanisms. Projects
without `.bones/infra/` must use the selected canonical framework, while a
present invalid local package must fail clearly rather than silently use built-in
framework behavior.

### Validation

Validation will include:

- Python unit tests for canonical framework behavior, scaffold materialization,
  whole-package built-in/local precedence, relative imports, invalid local
  infrastructure, fail-fast SSH ordering, runtime delegation, explicit nginx
  templates, and manifest artifacts/services/mode.
- Adapted framework behavior tests proving canonical and vendored source
  preserve current framework semantics and do not use registry dispatch or
  template fallback.
- Rust initialization and materialization tests proving every named framework
  and the custom scaffold receive the expected infrastructure and deployment
  files without maintaining duplicate canonical framework source in BonesDeploy.
- Repository searches for obsolete hook names, registry names, `confs`,
  framework classes, duplicate framework assets, and implicit template fallback;
  every remaining match is reviewed as intentional.
- `uv run pytest` in `crates/bonesinfra/python`, `cargo test`, `cargo clippy`,
  `cargo fmt`, `shfmt -w .`, and the Python package checks required by its
  `AGENTS.md`.
- Final diff review and documentation review. The long e2e suite is not run.


## Tasks

### Implementation

- [x] Restore Django, Laravel, Next, Nuxt, Rails, SvelteKit, and Vue as canonical BonesInfra framework packages with explicit runtime, manifest, custom, supporting module, and template resources.
- [x] Add a BonesInfra materialization boundary that copies the selected canonical framework package and base kit infrastructure into a destination without coupling the source of truth to BonesDeploy assets.
- [x] Change the project loader to resolve a complete local `.bones/infra` package first and the selected canonical framework second, failing on every present local-package error without fallback or partial mixing.
- [x] Preserve the removal of root hooks, `confs/` lookup, registry dispatch, framework object hierarchy, and template runtime dispatch while adapting CLI runtime and manifest commands to the selected implementation contract.
- [x] Keep generic framework-independent helpers in BonesInfra core, require explicit project template paths, and combine selected framework manifest declarations with core declarations.
- [x] Update BonesDeploy initialization to retain deployment assets locally and invoke BonesInfra materialization for the base or selected framework infrastructure.
- [x] Update synchronization so deployment assets may refresh while existing vendored `.bones/infra` snapshots are never rewritten.
- [x] Remove duplicate framework infrastructure assets from BonesDeploy and delete obsolete paths, imports, tests, and dead ownership-specific code.
- [x] Add Python tests for canonical framework behavior, materialization, built-in/local precedence, invalid local package failure, explicit templates, manifests, and preserved deployment semantics.
- [x] Add Rust tests proving initialization materializes every framework snapshot, preserves deployment scripts, omits old root paths, and does not maintain a second canonical framework source under BonesDeploy.

### Validation

- [x] Run `uv run pytest` from `crates/bonesinfra/python`, including canonical and vendored framework coverage.
- [x] Run affected Rust tests and workspace tests excluding `e2e`; verify initialization materializes infrastructure through BonesInfra and does not use duplicate BonesDeploy framework source.
- [x] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, and `shfmt -w .`; resolve all warnings and formatting changes.
- [x] Search the repository for obsolete hook names, registry names, framework classes, duplicate framework assets, and implicit template fallback; inspect every remaining match.
- [x] Review built-in and vendored runtime/manifest resolution, the final diff, and documentation without running the long e2e suite.

### Completion

- [x] Update `README.md`, `CONTEXT.md`, BonesInfra README/CONTEXT, project documentation, and embedded skill documentation to describe canonical BonesInfra frameworks, vendored `.bones/infra`, strict precedence, and removal of old hook/confs/registry docs.
- [x] Record implementation deviations, validation results, important discoveries, and deliberately unfinished work in the completion notes.

### Completion notes

Canonical framework Python packages and templates now live under BonesInfra and
are copied into new projects by the private `project materialize` command.
BonesDeploy retains only framework selection, defaults, and deployment assets.
The loader uses strict whole-package local precedence and built-in fallback only
when `.bones/infra` is absent.

Validation passed: 369 Python tests, workspace Rust tests excluding `e2e`, the
affected `bonesdeploy` init integration tests, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo fmt`, `ruff check`, `ruff format`,
`shfmt -w .`, and `git diff --check`. The long e2e suite was not run.


## Clarification

### Trigger

Review of the completed refactor found that framework implementations were
moved from the BonesInfra Python package into `crates/bonesdeploy/assets/`.
This makes BonesInfra unable to provide its own framework support and inverts
ownership between the two crates.

### Decision

BonesInfra owns the canonical framework implementations and their scaffold
resources under its Python package. BonesDeploy owns project initialization and
requests BonesInfra to materialize the selected framework into `.bones/infra/`.

The materialized `.bones/infra/` package is a vendored project snapshot. Newly
initialized projects receive that snapshot and execute it by default, so the
infrastructure is readable, committed, editable, and stable across BonesInfra
updates.

Runtime and manifest loading use one strict precedence rule:

- When the complete `.bones/infra/` package is absent, load the selected built-in
  framework implementation from BonesInfra.
- When `.bones/infra/` exists, load the complete local package and never consult
  the built-in implementation for that project.
- A present but incomplete, syntactically invalid, import-invalid, or contract-
  invalid local package fails with a path-specific error; it never falls back.

The local package is all-or-nothing for runtime and manifest resolution. This
prevents a project runtime from being paired accidentally with a different
manifest implementation. The framework registry, implicit template fallback,
and root hook system remain removed.

### Supersedes

This replaces the earlier ownership decision that generated framework source is
canonical under `crates/bonesdeploy/assets/frameworks/` and that BonesInfra is
framework-independent. It also replaces the requirement that projects without
`.bones/infra/` fail rather than use a built-in framework implementation.

### Effect on the record

`01-idea.md` now defines canonical framework implementations, built-in
framework loading, and vendored project snapshots. `02-plan.md` now assigns
framework source and materialization to BonesInfra, assigns initialization
coordination to BonesDeploy, and defines strict local-package precedence.
`03-tasks.md` replaces the completed snapshot-only implementation tasks with
the concrete ownership correction, loader fallback, materialization, tests, and
documentation work required by this clarification.
