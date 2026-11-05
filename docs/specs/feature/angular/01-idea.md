# Idea

## Request

Add an Angular framework template to BonesDeploy. The template deploys a
current Angular browser application as a native static site using the existing
local artifact build, release, nginx, and rollback lifecycle.

## Problem

BonesDeploy has no built-in Angular template. An Angular project cannot select
`angular` during initialization, receive framework build scripts, materialize
Angular infrastructure templates, or provision a matching static runtime.

Angular also cannot safely reuse the Vue defaults unchanged. The current
Angular application builder treats a string `outputPath` as a base and emits
the browser artifact beneath its `browser/` child. A template that serves
`dist` directly would therefore point nginx at the wrong directory.

## Definitions

**Angular template:** The built-in BonesDeploy framework selected with
`--template angular`. It owns Angular-specific initialization defaults, local
build assets, managed BonesInfra runtime behavior, infrastructure templates,
manifest declarations, and framework documentation.

**Static Angular application:** A single-application Angular workspace whose
browser application is built by the current `@angular/build:application`
builder and can run entirely from static files. It has no production Angular
Node server.

**Browser artifact:** The deployable files produced at `dist/browser` after
BonesDeploy invokes the local Angular CLI with `dist` as the output base. This
directory, including its `index.html`, is the Angular runtime web root.

## Desired outcome

Users can select `angular` interactively or pass `--template angular` during
non-interactive initialization. BonesDeploy scaffolds locked-package-manager
build scripts, builds the committed Angular source locally into a complete
`dist/browser` artifact, and deploys that artifact through the existing native
release lifecycle.

Site setup provisions the same isolated static nginx runtime model used by
other static frameworks. The deployed application serves assets from
`dist/browser`, resolves client-side routes through `index.html`, appears in
manifest and doctor behavior as a static site, and retains ordinary activation,
failed-activation rollback, and release pruning behavior.

## Scope

- Register Angular in the Rust framework model, initialization UX, defaults,
  environment examples, embedded framework assets, and public template docs.
- Add Angular dependency-install and production-build scripts for npm, pnpm,
  and Yarn lockfiles, with a deterministic `dist/browser` artifact contract.
- Add the managed BonesInfra Angular framework package, static nginx templates,
  manifest declarations, request validation, and template materialization and
  pruning registration.
- Add focused Rust and Python coverage for framework parsing, initialization,
  assets, defaults, materialization, pruning, runtime manifests, and embedded
  source integrity.
- Add an ignored Angular end-to-end fixture and lifecycle scenario equivalent
  to the existing Vue static-site scenario.
- Regenerate the committed BonesInfra wheel and update current framework
  inventories and user-facing documentation.

## Constraints

- The Angular template supports the current Angular application builder,
  `@angular/build:application`, and a single browser application selected by
  the workspace's ordinary `ng build` default.
- The local framework build invokes the repository-local Angular CLI with the
  production configuration and `dist` output base. The resulting
  `dist/browser/index.html` must exist before the build succeeds.
- `runtime.web_root` and the managed static runtime use `dist/browser` as one
  consistent artifact contract.
- Builds remain local Docker builds of the selected committed Git revision.
  Production receives the complete artifact and does not install Node or build
  Angular source.
- Existing native release, activation, rollback, pruning, isolation, nginx,
  and security boundaries remain unchanged.
- The implementation reuses existing framework and static-runtime extension
  points without adding a generalized frontend abstraction or dependency.
- The committed BonesInfra wheel must match the Python source and templates.
- End-to-end coverage is authored and reviewed but is not run by the agent
  unless the user explicitly requests it.
- Implementation begins only after a human reviews and approves this planning
  record.

## Exclusions

- Angular SSR, hybrid rendering, prerender orchestration, and a production
  Angular Node server.
- Legacy Angular builders and compatibility behavior for their different output
  layouts.
- Selecting among multiple Angular applications or libraries in one workspace.
- Creating or modifying application source, Angular routes, `angular.json`, or
  package-manager lockfiles in the user's project.
- Docker runtime support or changes to the generic deployment lifecycle.
- Refactoring Vue or other static frameworks into a new shared framework layer.
