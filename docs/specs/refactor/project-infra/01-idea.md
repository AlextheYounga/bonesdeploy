# Idea

## Request

Move framework-specific server infrastructure out of hidden registry dispatch
while keeping the canonical framework implementations in the BonesInfra Python
package. Generate a readable, editable `.bones/infra/` snapshot for each new
project, and use that project snapshot in preference to the built-in framework.

Remove the root-level `.bones/custom.py` hook system, the `.bones/confs/`
override system, the installed Python framework registry, and all compatibility
behavior for those systems.

## Problem

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

## Definitions

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

## Desired outcome

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

## Scope

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

## Constraints

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

## Exclusions

This change does not rename the entire `bonesinfra` Python package to `core`.
It does not add an `infra refresh` command or automatically rewrite existing
vendored snapshots. It does not merge deployment scripts into infrastructure,
redesign bonesremote release workflows, or add project dependency management.
It does not preserve root-level `.bones/custom.py`, `.bones/confs/`, custom hook
names, the installed framework object hierarchy, or a replacement framework
registry. It does not provide partial local-package fallback or combine local
and built-in runtime/manifest implementations.
