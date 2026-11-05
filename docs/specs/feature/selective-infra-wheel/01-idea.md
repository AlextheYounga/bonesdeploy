# Idea

## Request

Keep one universal BonesInfra wheel that installs normally. After BonesDeploy
materializes the complete `infra/` directory, selectively prune files and
folders that belong to frameworks other than the project's selected framework.
Define the framework-owned paths in a ruleset represented as a map in
BonesInfra.

## Problem

BonesInfra currently materializes templates for every built-in framework into
each project's `infra/` directory. A project uses one native framework, or no
native framework when it uses Docker, so the other framework-specific files are
irrelevant and obscure which managed infrastructure applies to the project.

## Definitions

**Universal wheel:** The single release-built `bonesinfra-*.whl` containing all
shared and framework-specific Python code. Every project receives the same wheel
bytes. The wheel is not tailored, rewritten, or pruned.

**Materialized infrastructure:** The files written under a project's `infra/`
directory by BonesDeploy and BonesInfra. This includes the universal wheel,
managed templates, deployment scripts, and project-owned directories.

**Framework ruleset:** A BonesInfra map from each supported framework name to
the exact `infra/`-relative managed files or directories owned by that
framework. Only paths named by this map are eligible for framework pruning.

**Pruning:** Removing framework-owned paths from the materialized `infra/`
directory after normal materialization. Pruning does not inspect or alter wheel
contents.

## Desired outcome

Every initialized or updated project contains the unchanged universal
BonesInfra wheel, shared infrastructure files, and only the materialized files
associated with its selected native framework. Docker projects contain no
framework-owned materialized files. Re-running materialization after a
framework or backend change removes paths belonging to the previous selection.

## Scope

- Add the framework-to-path ruleset to BonesInfra.
- Prune unselected framework paths after normal project materialization.
- Pass the finalized backend and framework selection through init and update.
- Cover native, custom, Docker, repeated materialization, and profile-change
  behavior with tests.
- Update documentation that currently describes complete framework template
  materialization.

## Constraints

- Keep exactly one canonical universal wheel and copy it unchanged into every
  project.
- Materialize the complete managed infrastructure before pruning it.
- Delete only exact paths declared in the framework ruleset.
- Keep shared infrastructure, the selected framework's paths, project-owned
  `infra/custom/`, `infra/secrets/`, and the universal wheel.
- Treat a native empty template as `custom`; treat Docker as having no selected
  framework.
- Use the finalized runtime configuration rather than independently parsing
  partially written project files.

## Exclusions

- Tailoring, rebuilding, or editing wheel archives and wheel metadata.
- Deleting framework packages from the cached Python environment.
- Static dependency or import-graph analysis.
- Pruning shared language runtimes, server logic, Compose support, or other
  paths not explicitly associated with a framework in the ruleset.
- Introducing a plugin system or user-configurable pruning rules.
