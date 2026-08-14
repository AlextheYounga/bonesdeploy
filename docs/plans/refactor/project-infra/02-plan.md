# Plan

## Current behavior

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

## Intended behavior

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

## Approach

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

## Responsibilities and boundaries

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

## Affected areas

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

## Decisions

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

## Risks

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

## Validation

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
