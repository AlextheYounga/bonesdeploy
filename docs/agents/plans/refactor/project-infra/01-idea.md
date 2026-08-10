# Idea

## Request

Move framework-specific server infrastructure from the installed BonesInfra Python package into generated `.bones/infra/` source. Keep BonesInfra core as immutable, generic deployment machinery and make project infrastructure a readable, editable snapshot generated with each framework scaffold.

Remove the root-level `.bones/custom.py` hook system, the `.bones/confs/` override system, the installed Python framework registry, and all compatibility behavior for those systems. Generate `infra/__init__.py`, `infra/runtime.py`, `infra/manifest.py`, `infra/custom.py`, and the framework-specific templates required by the generated infrastructure.

## Problem

BonesInfra currently hides Laravel, Rails, Django, Next, Nuxt, SvelteKit, and Vue deployment policy inside installed framework classes and dispatches through a framework registry. Runtime and manifest commands therefore depend on framework code that is not visible in the project being deployed.

The current root-level custom hook and `confs/` fallback mechanisms also make project behavior implicit: core scans for special filenames and silently selects override templates. This makes infrastructure difficult to inspect, version, test, and change independently of the installed package.

## Definitions

**BonesInfra core:** The installed `bonesinfra` Python package containing framework-independent deployment primitives, pyinfra execution, setup, SSL, database services, manifest inspection, path calculation, and shared Linux operations. Core does not contain runtime implementations for supported web frameworks or knowledge of project `custom.py`.

**Project infrastructure:** Trusted local Python source under `.bones/infra/`, including `runtime.py`, `manifest.py`, `custom.py`, and every template directly used by that source. It owns framework policy and is loaded from the project supplied to a BonesInfra command.

**Runtime entrypoint:** `.bones/infra/runtime.py` with a callable `deploy(ctx)`. It describes the selected project's server runtime and executes framework-specific orchestration inside the active pyinfra planning context.

**Manifest entrypoint:** `.bones/infra/manifest.py` with callable functions `artifacts(ctx)`, `services(ctx)`, and `mode(ctx)`. It declares project-owned manifest entries while the core manifest engine remains responsible for inspection, deduplication, and report rendering.

**Project template:** A template stored under `.bones/infra/templates/` and directly selected by generated project infrastructure. It is the authoritative template for that project; core does not fall back to a packaged or `confs/` template for it.

**Framework snapshot:** The generated framework source copied into a project's `.bones/` repository at initialization. Updating BonesDeploy does not rewrite an existing snapshot.

## Desired outcome

Newly initialized projects contain a real `.bones/infra` Python package and no root `.bones/custom.py` or `.bones/confs/`. Named framework projects contain readable runtime, manifest, custom, and required local template files; custom projects contain a minimal usable infrastructure package.

Runtime and manifest commands validate and load the project's explicit entrypoints before opening SSH. Runtime invokes the project's `deploy(ctx)`; manifest combines core declarations with the project's artifact, service, and mode declarations. Invalid or missing project infrastructure fails with the relevant path before any SSH connection.

Supported framework deployments retain their current package installation, build/runtime mode, nginx, systemd, AppArmor, socket/port, validation, writable-path, placeholder, and manifest behavior while making those decisions visible in generated project source.

## Scope

The change includes:

- A project infrastructure loader with package-relative imports and explicit validation of runtime and manifest entrypoints.
- Generated `infra/` scaffolds for the base kit and every supported named framework: Django, Laravel, Next, Nuxt, Rails, SvelteKit, and Vue.
- Migration of framework orchestration and directly used framework templates into embedded project assets.
- Removal of hook dispatch, root-level custom loading, `confs/` lookup, the installed framework runtime registry, and `bonesinfra runtime list`.
- Explicit template paths in generic nginx helpers and neutral core helpers for operations reused by generated infrastructure.
- Project-owned manifest declarations combined by the generic core inspector.
- Rust initialization/scaffolding tests, Python loader/runtime/template/manifest tests, behavior-preservation coverage, and documentation updates.

## Constraints

Project infrastructure remains trusted arbitrary local Python and may import the standard library, pyinfra, and installed BonesInfra core modules. No dependency-installation system is introduced.

`.bones/deployment/` remains separate from `.bones/infra/`: deployment scripts serve the bonesremote release/build lifecycle, while infra source describes server/runtime infrastructure. The `.bones` directory remains its own Git repository and existing publishing behavior remains in use.

The existing manifest tuple formats are retained where practical. Framework deployment semantics are preserved rather than redesigned. Old projects are not automatically migrated and are not served by compatibility fallbacks.

Implementation must follow repository conventions, leave runnable focused tests for non-trivial behavior, run the required Rust/Python/static checks, and not run the long e2e suite.

## Exclusions

This change does not rename the entire `bonesinfra` Python package to `core`. It does not add an `infra refresh` command or automatically rewrite existing framework snapshots. It does not merge deployment scripts into infrastructure, redesign bonesremote release workflows, or add project dependency management. It does not preserve root-level `.bones/custom.py`, `.bones/confs/`, custom hook names, the installed framework object hierarchy, or a replacement framework registry.
