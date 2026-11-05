# Angular Static Site

<!-- Merged from feature/angular -->

## Idea

### Request

Add an Angular framework template to BonesDeploy. The template deploys a
current Angular browser application as a native static site using the existing
local artifact build, release, nginx, and rollback lifecycle.

### Problem

BonesDeploy has no built-in Angular template. An Angular project cannot select
`angular` during initialization, receive framework build scripts, materialize
Angular infrastructure templates, or provision a matching static runtime.

Angular also cannot safely reuse the Vue defaults unchanged. The current
Angular application builder treats a string `outputPath` as a base and emits
the browser artifact beneath its `browser/` child. A template that serves
`dist` directly would therefore point nginx at the wrong directory.

### Definitions

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

### Desired outcome

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

### Scope

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

### Constraints

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

### Exclusions

- Angular SSR, hybrid rendering, prerender orchestration, and a production
  Angular Node server.
- Legacy Angular builders and compatibility behavior for their different output
  layouts.
- Selecting among multiple Angular applications or libraries in one workspace.
- Creating or modifying application source, Angular routes, `angular.json`, or
  package-manager lockfiles in the user's project.
- Docker runtime support or changes to the generic deployment lifecycle.
- Refactoring Vue or other static frameworks into a new shared framework layer.


## Plan

### Current behavior

`crates/bonesdeploy/src/frameworks.rs` is the local CLI's explicit framework
registry. It owns framework wire names, display names, initialization questions,
runtime defaults, and environment examples through sibling modules such as
`frameworks/vue.rs`. Vue is the closest static framework: it has no questions,
defaults `web_root` to `dist`, applies recursive release permissions to `dist`,
and uses the workspace's default Node version.

`crates/bonesdeploy/assets/frameworks/<framework>/` contains embedded
project-scaffold assets. Vue supplies environment examples and two numbered
build scripts. The scripts install dependencies according to the committed npm,
pnpm, or Yarn lockfile, run the project's build command, and remove disposable
dependencies and build caches before artifact packaging. The framework asset
loader and scaffolder are generic once a framework is registered.

The canonical provisioning implementations live in the committed BonesInfra
wheel under `crates/bonesinfra/python/src/bonesinfra/frameworks/`. Vue's
`runtime.py` delegates to `deploy_static`, has no shared directories, and serves
`dist`; its manifest declares the placeholder and current static roots, nginx
artifacts, runtime socket, and nginx service. Framework-specific Jinja templates
are fully materialized into `infra/templates/`, and
`crates/bonesinfra/src/framework_paths.rs` removes template paths belonging to
unselected frameworks.

`bonesinfra.project.FRAMEWORKS` controls dynamic framework loading, while
`bonesinfra.config.request._FRAMEWORKS` validates framework names at the request
boundary. The Rust and Python registries currently include Django, Laravel,
Next, Nuxt, Rails, SvelteKit, Vue, and the custom fallback, but not Angular.

Rust integration tests cover framework parsing, defaults, embedded assets,
initialization, and infrastructure materialization. Python tests cover manifests
and parse every managed Jinja template. The ignored Incus E2E suite has a Vue
static scenario that provisions a site, deploys two artifacts, verifies failed
activation rollback, and prunes an old release. Python source changes require
regenerating the committed BonesInfra wheel before Rust builds pass its embedded
source integrity check.

### Intended behavior

Angular is an ordinary built-in framework named `angular`. It appears in the
interactive template list and is accepted by non-interactive initialization.
Its runtime defaults select the native backend, the workspace Node default,
`dist/browser` as the web root, and static-release permissions covering `dist`.
It has no framework-specific initialization questions.

The Angular build assets install locked dependencies, invoke
`./node_modules/.bin/ng build --configuration production --output-path dist`,
and fail clearly unless `dist/browser/index.html` exists. This uses Angular's
current application-builder output contract without deriving an Angular project
name or modifying `angular.json`. The completed artifact retains
`dist/browser` and removes `node_modules`, `.angular/cache`, and the consumed
build-script directory.

BonesInfra loads the managed Angular framework, provisions the established
static nginx runtime with `dist/browser` as its static root, and reports the
matching placeholder and current web roots in the manifest. Nginx serves normal
assets and falls back to `index.html` for Angular client-side routes. Angular has
no shared directories and no long-running application process beyond the
per-site nginx service.

The Angular templates are materialized only for Angular native sites and are
pruned from other native sites. Docker-runtime projects retain the existing
behavior of pruning native framework templates. User and agent documentation
lists Angular as a supported static template and states its browser artifact
contract.

### Approach

Add an `Angular` variant and sibling `frameworks/angular.rs` module through the
same explicit dispatch points used by Vue. Define `dist/browser` directly in
the Angular defaults and use a recursive `dist` directory permission so the
parent and browser artifact remain traversable after release sealing. Add the
Angular environment examples and numbered build scripts to the existing
embedded framework asset collection; no new asset loader is required.

Keep package-manager-specific dependency installation consistent with the
existing Node templates. After installation, call the repository-local `ng`
binary directly so npm, pnpm, and Yarn projects share one deterministic Angular
build invocation. Pass the production configuration and `dist` output base,
validate the current builder's `dist/browser/index.html` result, then remove
build-only content while preserving the browser artifact.

Create a canonical `bonesinfra.frameworks.angular` package matching the Vue
static framework shape: focused `runtime.py`, `manifest.py`, `custom.py`, and
Angular-owned Jinja templates. Adapt the static root and labels to
`dist/browser`; retain the existing `deploy_static` orchestration and SPA nginx
fallback. Register Angular in both Python framework allowlists and in the Rust
framework-template pruning map.

Extend existing test matrices and observable integration tests rather than
adding new test infrastructure. Add a minimal current Angular single-application
fixture and an ignored E2E module that uses the common native lifecycle harness.
Regenerate the universal wheel after the managed Python package and templates
are complete, then update framework inventories and artifact-contract docs.

### Responsibilities and boundaries

The Rust Angular framework module owns initialization defaults, release
permissions, and environment examples. `frameworks.rs` owns public registration,
wire parsing, display, and dispatch. The init command and generic framework asset
scaffolder remain unchanged apart from their visible supported-template list.

Angular scaffold assets own local dependency installation, the Angular CLI
build invocation, browser-artifact validation, and build-only cleanup. Angular
itself remains responsible for compiling application source; BonesDeploy only
selects the production build and deterministic output base.

The managed BonesInfra Angular package owns host runtime provisioning and
manifest declarations. Existing `deploy_static` behavior owns static nginx and
systemd setup. Angular's Jinja directory owns its materialized nginx and
placeholder templates, while `framework_paths.rs` owns selection-time pruning.

Existing Rust and Python integration-test boundaries own registry, scaffold,
materialization, and manifest assertions. The E2E package owns the ignored
full-lifecycle Angular fixture. Documentation owns the public support status and
the `dist/browser` contract.

### Affected areas

- `crates/bonesdeploy/src/frameworks.rs`, new
  `crates/bonesdeploy/src/frameworks/angular.rs`, and
  `crates/bonesdeploy/src/cli/args.rs` for framework registration, defaults, and
  CLI help.
- New `crates/bonesdeploy/assets/frameworks/angular/` environment examples,
  build documentation, and numbered build scripts.
- New
  `crates/bonesinfra/python/src/bonesinfra/frameworks/angular/` runtime,
  manifest, custom hook, and Jinja templates.
- `crates/bonesinfra/python/src/bonesinfra/project.py` and
  `crates/bonesinfra/python/src/bonesinfra/config/request.py` for Python loading
  and request-boundary validation.
- `crates/bonesinfra/src/framework_paths.rs` and the regenerated
  `crates/bonesinfra/assets/bonesinfra-*-py3-none-any.whl` for materialization,
  pruning, and embedded source integrity.
- `crates/bonesdeploy/tests/{config_frameworks,assets,init}.rs`,
  `crates/bonesinfra/tests/{project_core,embedded_source}.rs`, and relevant
  BonesInfra Python tests for observable Angular behavior.
- `e2e/fixtures/angular.md`, `e2e/tests/setup/angular.rs`, `e2e/tests/setup.rs`,
  `e2e/tests/setup/harness.rs`, `e2e/mdunpack.sh`, and `e2e/README.md` for the
  ignored Angular lifecycle scenario.
- `README.md`, `crates/bonesdeploy/assets/skill/templates.md`, `CONTEXT.md`,
  `docs/ARCHITECTURE.md`, `docs/architecture/reference.md`, and
  `tests/ASSERTIONS.md` where current framework inventories or the native
  artifact coverage list are maintained.

### Decisions

- Angular is static-only. This fits the requested template into the complete
  existing static runtime instead of introducing an unrequested Node server and
  SSR lifecycle.
- The supported build contract is the current `@angular/build:application`
  builder. Supporting legacy output layouts would add compatibility branches
  without a stated requirement.
- The build passes `--output-path dist` and serves `dist/browser`. Angular
  defines a string output path as the base and `browser` as the default browser
  subdirectory, producing a stable path independent of the Angular project name.
- The local `ng` binary is invoked directly after locked dependency
  installation. This gives every supported package manager the same flags and
  avoids package-manager-specific argument forwarding behavior.
- The build validates `dist/browser/index.html` before cleanup. A clear build
  failure is safer than uploading an artifact that site setup cannot serve.
- Angular receives its own small managed framework package and templates,
  following the repository's explicit framework ownership and pruning model.
  Vue is not refactored into a generalized static-framework abstraction.
- Angular uses the existing workspace Node default rather than introducing an
  Angular-specific version policy. The E2E fixture pins its Angular dependencies
  to versions compatible with that Node contract.

### Risks

- A mismatch between Rust defaults, build output, Python runtime static root,
  manifest paths, and permission paths would produce an artifact that builds but
  cannot be served. Tests must assert `dist/browser` across all boundaries.
- Missing one explicit framework registry can make init accept Angular while
  BonesInfra rejects it, or can prune Angular's templates after materialization.
  Registry and pruning matrices must include Angular together.
- A project using a legacy builder, a custom target that rejects the selected
  CLI options, or a workspace without one default build application will fail
  during the local build. The error must remain clear; these workspace shapes
  are outside this template's contract.
- Angular projects configured for SSR or hybrid rendering can emit server
  output that this static runtime intentionally does not execute. Documentation
  and the fixture must represent a browser-only application.
- Python source or template changes without wheel regeneration will fail the
  Rust embedded-source check. Wheel regeneration must occur before final Rust
  validation.
- Adding Angular to the shared E2E harness can accidentally run it as part of
  ordinary validation. The scenario must remain ignored and must not be
  executed without explicit user instruction.

### Validation

- Rust framework and init tests prove `angular` parses and displays, has no
  framework variables, defaults to `dist/browser`, scaffolds executable Angular
  build assets, writes Angular environment examples, and rejects unknown
  templates as before.
- Asset tests prove the build uses the repository-local Angular CLI, selects a
  production build with `dist` as output base, requires
  `dist/browser/index.html`, preserves that artifact, and removes Angular caches,
  dependencies, and consumed build scripts.
- BonesInfra Rust tests prove Angular templates are embedded, materialized for
  Angular, pruned for every other native framework, and removed for Docker
  runtime projects without changing project-owned infrastructure.
- Python tests prove Angular is accepted at the request boundary, dynamically
  loads, renders every managed template, reports static mode, and declares
  placeholder and current `dist/browser` artifacts plus the existing nginx
  service and socket resources.
- `cargo build-wheel` regenerates a wheel accepted by the embedded-source check.
  Focused Rust tests, the non-E2E workspace test suite, and the full Python suite
  pass afterward.
- `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .`
  complete without warnings or errors; shell scripts also pass syntax checks.
- The ignored Angular E2E definition is reviewed to confirm it covers setup,
  browser routing, first and second artifact deployment, failed activation
  rollback, and release pruning. It is not run by the agent without explicit
  permission.
- Final diff review confirms no SSR runtime, production Node installation,
  generic deployment-lifecycle change, unrelated static-framework refactor, or
  stale framework inventory was introduced.


## Tasks

### Implementation

- [x] Register `angular` in the Rust framework model and CLI help, with a
  focused Angular module that has no questions and defaults Node, release
  permissions, and `web_root` to the settled static browser-artifact contract.
- [x] Add Angular environment examples and scaffolded build documentation and
  scripts that install locked npm, pnpm, or Yarn dependencies, run the local
  Angular CLI production build with `dist` as its output base, require
  `dist/browser/index.html`, and remove only build-time content.
- [x] Add the managed BonesInfra Angular framework package and Angular-owned
  Jinja templates so runtime provisioning and manifest declarations use
  `dist/browser`, static SPA routing, and the existing per-site nginx service.
- [x] Register Angular in the BonesInfra framework loader, request-boundary
  allowlist, and Rust framework-template pruning map so selected Angular
  templates survive materialization and all unselected framework templates are
  removed.
- [x] Extend Rust and Python integration coverage for Angular parsing, defaults,
  assets, initialization, environment examples, framework loading, request
  validation, template rendering, manifests, materialization, and pruning.
- [x] Regenerate the committed universal BonesInfra wheel after the Angular
  Python package and templates are complete, and verify its embedded source
  matches the working tree.
- [x] Add a minimal current single-application Angular fixture and ignored E2E
  setup scenario covering static routing and the common native artifact
  lifecycle; register it with the pinned-Node harness and fixture tooling.
- [x] Update README, embedded skill documentation, architecture and context
  inventories, E2E documentation, and assertion coverage to list Angular as a
  static template and document the `dist/browser` artifact contract.

### Validation

- [x] Run the focused BonesDeploy and BonesInfra Rust integration tests and
  verify they prove the Angular registry, scaffold, `dist/browser` build
  contract, manifest, materialization, and pruning behavior.
- [x] Run the full non-E2E Rust workspace tests and full BonesInfra Python test
  suite, and resolve every failure without executing the ignored E2E scenario.
- [x] Run shell syntax checks, `cargo fmt`, `cargo clippy`, `shfmt -w .`,
  `ruff check .`, and `ruff format .`, and resolve every warning or error.
- [x] Review the ignored Angular E2E fixture and test definition to confirm they
  cover site setup, client-side route fallback, two deployments, failed
  activation rollback, and release pruning without running the scenario.

### Completion

- [x] Review the final diff for consistent `dist/browser` ownership across
  build, configuration, permissions, runtime, manifest, and tests; remove dead
  code, accidental generated content, and unrelated changes.
- [x] Confirm framework inventories agree across Rust, Python, pruning, docs,
  and E2E registration, and confirm the committed wheel is current.
- [x] Record validation results, meaningful plan deviations, discoveries, and
  deliberately unfinished work in the completion notes.

### Completion notes

- Implemented the approved static Angular template with the current
  `@angular/build:application` builder, locked npm/pnpm/Yarn installation, the
  repository-local Angular CLI, and one `dist/browser` contract across build,
  configuration, permissions, runtime provisioning, manifests, and tests.
- Added Angular-owned static nginx and placeholder templates. The static runtime
  does not copy Vue's unused application-service and AppArmor templates because
  `deploy_static` only consumes the nginx configuration and placeholder page.
- Regenerated `bonesinfra-0.5.7-py3-none-any.whl`; embedded-source,
  materialization, and pruning tests confirm that the wheel and source agree.
- `cargo clippy --workspace --all-targets --all-features`,
  `cargo test --workspace --exclude e2e`, `cargo fmt`, `shfmt -w .`, shell syntax
  checks, `ruff check .`, `ruff format .`, and the 530-test Python suite passed.
- `cargo test -p e2e --test fixture_consistency` passed, and
  `cargo test -p e2e --test setup --no-run` compiled the ignored Angular
  lifecycle scenario. The Angular fixture was also built locally with `npm ci`
  and produced `dist/browser/index.html`.
- The ignored Incus E2E scenario was not executed. Its definition covers the
  root page, a non-root SPA fallback route, first and second deployments, failed
  activation rollback, and release pruning through the shared native harness.
