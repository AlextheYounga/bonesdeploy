# Plan

## Current behavior

`bonesinfra.pyinfra.runner.run()` currently receives `config_path` so it can load root `.bones/custom.py` through `bonesinfra.cli.hooks`, validates that module before SSH, and passes it as a second argument to deployment callables. Setup, runtime, SSL, and helper commands retain hook parameters. Runtime loads `bonesinfra.cli.commands.runtime.template_runtime`, queries `bonesinfra.frameworks.get_framework()`, and derives nginx/AppArmor policy from the installed framework object.

The Python package contains `bonesinfra/frameworks/` with base, common, and seven framework implementations plus a registry and `runtime list`. The manifest engine stores project-runtime artifacts in `COMMON_ARTIFACTS` and queries the framework registry for artifacts, services, and mode. Generic nginx site rendering resolves templates by first checking a `confs/` path and then the installed asset directory.

Rust initialization embeds `assets/kit` and selected framework assets. The kit currently contains root `custom.py` and `confs/`; framework scaffolding replaces only `deployment/` after the base kit is copied. The `.bones` directory is a separate Git repository. Update synchronization currently refreshes deployment assets from source but does not represent project infrastructure.

Existing tests cover hook loading, pyinfra runner inventory setup, framework manifest behavior, nginx template behavior, framework asset content, and init filesystem effects. These tests encode the current ownership model and require replacement at the new project-infrastructure boundaries.

## Intended behavior

BonesInfra resolves the project infrastructure directory relative to the supplied `bones.toml`, imports it as a package, and validates the required callables before SSH. Runtime commands call `.bones/infra/runtime.py:deploy(ctx)` inside the active pyinfra context. Manifest commands load `.bones/infra/manifest.py`, use its artifact/service/mode declarations, and combine them with genuinely core-owned declarations and selected core service declarations.

Generated framework source owns all framework policy: language/runtime installation calls, paths, server mode, socket or TCP choice, nginx and AppArmor settings, application services, placeholders, validations, and local template selection. Generated source explicitly imports generic core operations and references `.bones/infra/templates` paths. `infra/custom.py` is ordinary project code called visibly by generated runtime source and is not recognized by BonesInfra.

The base kit generates a minimal no-framework runtime, manifest, custom module, and package initializer. Named framework scaffolds generate the same entrypoint shape plus their complete framework source and required templates. No generated project contains root `custom.py` or `confs/`.

## Approach

Implement the change in dependency order. First add the project package loader and establish its error contract, then adapt runner and CLI entrypoints to use it without module-hook plumbing. Next move generic helpers out of the framework namespace, translate each existing framework implementation into embedded `infra/` source, and move directly used templates beside that source. Then change nginx helpers and manifest inspection to accept explicit project inputs and remove registry/fallback access.

Update Rust embedded asset scaffolding so kit initialization creates the base infra package and named framework initialization copies both framework `deployment/` and framework `infra/` trees while preserving kit deployment functions. Update synchronization to refresh only the existing deployment behavior and never silently rewrite project infrastructure snapshots. Remove obsolete files, constants, imports, commands, and tests after the new path is working.

## Responsibilities and boundaries

The project loader belongs in a neutral BonesInfra core module used by the CLI; it owns path resolution, package import, callable validation, and fail-fast errors. `pyinfra/runner.py` owns SSH setup and execution-context entry, but not project policy or custom module discovery.

`cli/commands/setup`, `ssl`, `services`, and helpers own only genuinely generic operations. `cli/commands/runtime` delegates to the project runtime entrypoint. `manifest.py` owns declaration collection, inspection, deduplication, and rendering; project `infra/manifest.py` owns framework declarations.

Generated `.bones/infra/runtime.py` owns framework orchestration and local template selection. Generated `.bones/infra/manifest.py` owns the declarations for paths and services created by that orchestration. Generic language, nginx, systemd, AppArmor, validation, and path operations remain in neutral BonesInfra core modules.

Rust `infra/assets`, init scaffolding, and embedded framework assets own materialization. The `.bones` Git repository and existing stage/commit/push code remain the publishing boundary.

## Affected areas

Expected affected areas are:

- `crates/bonesinfra/python/src/bonesinfra/`: project loader, pyinfra runner, CLI commands, manifest engine, neutralized generic helpers, nginx rendering, removal of `cli/hooks.py`, `frameworks/`, and template runtime dispatch.
- `crates/bonesinfra/python/tests/`: loader fail-fast cases, runner invocation, explicit nginx templates, project manifest declarations, and adapted framework behavior tests; removal of `test_custom.py`.
- `crates/bonesdeploy/assets/kit/`: base `infra/` package and templates, removal of root `custom.py` and `confs/`.
- `crates/bonesdeploy/assets/frameworks/{django,laravel,next,nuxt,rails,sveltekit,vue}/`: generated infra packages, framework runtime/manifest/custom source, and project-owned templates.
- `crates/bonesdeploy/src/infra/assets/{kit,frameworks}.rs` and `crates/bonesdeploy/src/commands/init/scaffold.rs`: combined deployment and infra scaffolding with explicit framework snapshot boundaries.
- `crates/bonesdeploy/src/commands/update/sync.rs`: preservation of deployment synchronization without infrastructure fallback or rewrite.
- `crates/bonesdeploy-core/src/paths.rs`: removal of obsolete `confs` constants.
- `crates/bonesdeploy/tests/init.rs` and relevant Rust asset tests: generated layout, framework entrypoints/templates, `.bones` repository, and publishing expectations.
- `README.md`, `CONTEXT.md`, BonesInfra README/CONTEXT, project docs, and embedded skill documentation: new ownership model and removal of obsolete architecture references.

## Decisions

1. Keep the `bonesinfra` package name and split ownership by generated source versus installed core. This preserves the established package boundary while making framework policy project-visible.
2. Load `runtime.py` and `manifest.py` as siblings in a real `infra` package. Package imports are the native mechanism for generated source relationships and avoid per-file import hacks.
3. Validate project entrypoints before `connect_all`. Syntax, import, file, and callable errors must be local and actionable before network side effects.
4. Keep manifest declaration tuples and the existing inspector/report pipeline. Only declaration ownership changes; inspection behavior does not need a new metadata model.
5. Pass nginx template paths explicitly. A helper cannot know project template ownership and must not search packaged defaults or `confs/`.
6. Generate framework snapshots as embedded assets and do not implement an update-time infra refresh. Existing projects must remain editable and stable across BonesDeploy updates.
7. Keep generic service declarations for truly core services, while moving artifacts created by framework runtime source into each project manifest. This prevents the core inspector from retaining hidden framework knowledge.

## Risks

Framework behavior can regress while translating class inheritance into generated source, especially mode-specific paths, sockets, service units, AppArmor permissions, nginx configuration, placeholders, and validation order. Behavior-focused tests and asset-level assertions must cover those translations.

The project loader changes import timing and module identity. Relative imports, missing files, invalid syntax, import failures, and non-callable entrypoints must produce path-specific errors without opening SSH.

Moving templates into snapshots can leave a framework source reference pointing at a missing local file. Each generated runtime and local template tree must be validated by scaffolding tests and Python syntax/template tests.

Removing `confs/` and root hooks intentionally breaks old layouts. Commands requiring project infrastructure must fail clearly when `.bones/infra` is missing rather than silently using installed framework behavior.

## Validation

Validation will include:

- Python unit tests for project package loading, relative imports, missing and invalid entrypoints, fail-fast SSH ordering, runtime delegation, explicit nginx templates, and project manifest artifacts/services/mode.
- Adapted framework behavior tests proving generated source preserves current framework semantics and does not use registry dispatch or template fallback.
- Rust init and embedded-asset tests proving every named framework and the custom scaffold contain `infra/runtime.py`, `infra/manifest.py`, `infra/custom.py`, local required templates, deployment scripts, no root custom/confs paths, and a separate `.bones` Git repository.
- Repository searches for obsolete hook names, registry names, `confs`, and framework classes, with each remaining match reviewed as intentional.
- `python -m pytest` in `crates/bonesinfra/python`, `cargo test`, `cargo clippy`, `cargo fmt`, `shfmt -w .`, and the Python package checks required by its `AGENTS.md`.
- Final diff review and documentation review. The long e2e suite is not run.
