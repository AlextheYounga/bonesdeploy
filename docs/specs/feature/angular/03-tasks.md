# Tasks

## Implementation

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

## Validation

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

## Completion

- [x] Review the final diff for consistent `dist/browser` ownership across
  build, configuration, permissions, runtime, manifest, and tests; remove dead
  code, accidental generated content, and unrelated changes.
- [x] Confirm framework inventories agree across Rust, Python, pruning, docs,
  and E2E registration, and confirm the committed wheel is current.
- [x] Record validation results, meaningful plan deviations, discoveries, and
  deliberately unfinished work in the completion notes.

## Completion notes

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
