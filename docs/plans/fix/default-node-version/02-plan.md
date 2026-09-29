# Default Node Version Plan

## Current behavior

`bonesdeploy_core::config::Runtime` owns the canonical Node default through
`default_node_version()`, currently `24.19.0`. Framework runtime defaults use
that value, but the seven framework `.env.build.example` assets declare an
empty `NODE_VERSION=`.

`Framework::build_environment_example` receives the selected `Runtime`. Django,
Laravel, and Rails use it to render language-specific values; Next, Nuxt,
SvelteKit, and Vue currently return their assets unchanged. Scaffolding writes
this rendered content only when `.env.build` does not already exist.

At deployment time, BonesRemote passes `.env.build` values into the isolated
build. `node_resolve_version` also supports repository version files, while
`node_assert_exact_version` rejects an empty or non-exact result. This strict
validation produced the first-deployment failures recorded in the manual setup
report.

## Intended behavior

Each framework build-environment asset will declare
`NODE_VERSION={node_version}`. The shared framework boundary will render that
placeholder from `Runtime::node_version` after any framework-specific
rendering. Runtime defaults therefore produce `NODE_VERSION=24.19.0`, while an
explicit runtime value produces that exact value.

Existing `.env.build` files remain untouched. Deployment continues to prefer
the committed `NODE_VERSION`, then repository version files when that value is
absent, and continues rejecting non-exact versions.

## Approach

Change the framework assets to contain one shared Node placeholder and render
it centrally in `Framework::build_environment_example`. Keep each framework's
existing language-specific rendering unchanged. Update the integration tests
to assert both the canonical default and a configured override in generated
`.env.build` content.

This uses the existing runtime default and scaffolding path instead of adding a
second shell fallback or duplicating the version literal across assets.

## Responsibilities and boundaries

- `bonesdeploy-core` remains the owner of the canonical exact Node default.
- Framework `.env.build.example` assets declare where the committed build input
  appears.
- `Framework::build_environment_example` owns common rendering shared by all
  framework assets.
- Framework scaffolding tests own the observable generated-file contract.
- Deployment scripts remain responsible for source precedence and exact-version
  validation.

## Affected areas

- `crates/bonesdeploy/src/frameworks.rs`
- `crates/bonesdeploy/assets/frameworks/*/*.env.build.example`
- `crates/bonesdeploy/tests/assets.rs`
- `CONTEXT.md`
- `docs/architecture/reference.md`
- The generated embedded BonesInfra wheel if repository consistency tooling
  reports asset drift.

## Decisions

- Pin `24.19.0`, not floating `24`, because the deployment boundary requires
  exact reproducible versions and the repository already defines that release
  as its Node 24 default.
- Render from `Runtime::node_version` rather than hard-coding the release in
  seven assets, preserving one source of truth and configured overrides.
- Do not add a deployment-time fallback. A committed `.env.build` keeps build
  inputs explicit and reviewable, consistent with the existing architecture.
- Do not overwrite existing `.env.build` files. Existing preservation behavior
  protects application-owned build settings.

## Risks

- A framework omitted from common rendering could retain the empty value. Tests
  will enumerate every supported framework.
- Common rendering could accidentally disturb language-specific placeholders.
  Existing framework configuration tests and full crate tests cover those
  values.
- Embedded assets could diverge from generated project infrastructure. Wheel
  consistency checks will detect this.

## Validation

- Focused BonesDeploy asset tests show every framework scaffolds
  `NODE_VERSION=24.19.0` from `Runtime::default()`.
- A focused override assertion shows a configured exact version is rendered.
- The existing preservation test shows an existing `.env.build` remains
  unchanged.
- BonesDeploy and workspace tests pass without running the end-to-end package.
- `cargo clippy`, `cargo fmt`, `shfmt -w .`, wheel consistency checks, and
  `git diff --check` pass.
- Final diff review confirms deployment version precedence and validation were
  not weakened and unrelated existing documentation changes remain untouched.
