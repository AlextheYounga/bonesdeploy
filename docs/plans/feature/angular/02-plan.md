# Plan

## Current behavior

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

## Intended behavior

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

## Approach

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

## Responsibilities and boundaries

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

## Affected areas

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

## Decisions

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

## Risks

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

## Validation

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
