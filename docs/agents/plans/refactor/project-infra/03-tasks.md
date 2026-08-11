# Tasks

## Implementation

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

## Validation

- [x] Run `uv run pytest` from `crates/bonesinfra/python`, including canonical and vendored framework coverage.
- [x] Run affected Rust tests and workspace tests excluding `e2e`; verify initialization materializes infrastructure through BonesInfra and does not use duplicate BonesDeploy framework source.
- [x] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, and `shfmt -w .`; resolve all warnings and formatting changes.
- [x] Search the repository for obsolete hook names, registry names, framework classes, duplicate framework assets, and implicit template fallback; inspect every remaining match.
- [x] Review built-in and vendored runtime/manifest resolution, the final diff, and documentation without running the long e2e suite.

## Completion

- [x] Update `README.md`, `CONTEXT.md`, BonesInfra README/CONTEXT, project documentation, and embedded skill documentation to describe canonical BonesInfra frameworks, vendored `.bones/infra`, strict precedence, and removal of old hook/confs/registry docs.
- [x] Record implementation deviations, validation results, important discoveries, and deliberately unfinished work in the completion notes.

## Completion notes

Canonical framework Python packages and templates now live under BonesInfra and
are copied into new projects by the private `project materialize` command.
BonesDeploy retains only framework selection, defaults, and deployment assets.
The loader uses strict whole-package local precedence and built-in fallback only
when `.bones/infra` is absent.

Validation passed: 369 Python tests, workspace Rust tests excluding `e2e`, the
affected `bonesdeploy` init integration tests, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo fmt`, `ruff check`, `ruff format`,
`shfmt -w .`, and `git diff --check`. The long e2e suite was not run.
