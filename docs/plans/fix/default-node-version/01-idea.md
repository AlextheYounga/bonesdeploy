# Default Node Version

## Request

Create a fix branch that pins a default Node 24 version so newly initialized
projects do not fail their first deployment when no project-specific Node
version has been selected.

## Problem

BonesDeploy already defines `24.19.0` as its canonical runtime Node default,
but every framework's generated `.env.build` contains an empty
`NODE_VERSION=` entry. Deployment intentionally requires an exact version, so
fresh projects fail before dependency installation unless users discover and
fill that entry manually.

## Definitions

**Default Node version:** The exact Node release written to a newly generated
`.env.build` when the project has not selected another runtime value. It is the
existing canonical `Runtime::node_version` default, currently `24.19.0`, not a
floating major-version selector.

**Project-specific Node version:** An exact version selected in project runtime
configuration and rendered into `.env.build`. It takes precedence over the
default because it is the value committed with that project.

## Desired outcome

Every newly initialized supported framework project has an exact
`NODE_VERSION=24.19.0` in `.env.build` by default. A configured exact Node
version is rendered instead. Deployment retains its exact-version validation.

## Scope

- Render the canonical runtime Node version into every framework's generated
  `.env.build`.
- Cover default and configured Node versions through the framework scaffolding
  boundary.
- Keep architecture and context documentation consistent with the generated
  build environment.

## Constraints

- Use the existing `default_node_version()` source of truth.
- Keep Node versions exact and reproducible; do not make `24` a floating
  production build input.
- Preserve existing `.env.build` files during init and update workflows.
- Preserve all existing project-specific version discovery and validation.
- Do not run the end-to-end suite unless explicitly requested.

## Exclusions

- Automatically modifying or committing existing projects' `.env.build` files.
- Retrying the nine manual deployment fixtures.
- Changing Node download, cache, Corepack, or version-file precedence behavior.
- Remediating the separate Django and Rails runtime setup failures.
