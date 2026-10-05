# Tasks

## Implementation

- [x] Define the pinned mise `2026.10.0` installation policy, verified binary
  source, precompiled-only settings, local cache paths, production runtime store,
  and stable per-site runtime link paths in shared BonesDeploy/BonesInfra code.
- [x] Make exact managed runtime versions authoritative in project configuration:
  add Django Python `3.14.0`, preserve exact Rails/Node choices, derive managed
  build variables from config, and transport all three versions into PyInfra
  without broadening the BonesRemote deployment descriptor.
- [x] Replace local Ruby source compilation, Django distribution Python, and the
  custom Node archive installer with pinned mise installations in the existing
  Docker cache, retaining Node version-source precedence and package-manager
  behavior.
- [x] Replace production Ruby, Python, and Node installers with a shared
  root-owned mise runtime store and stable site links; validate exact versions
  as the runtime user and prohibit source-build fallback.
- [x] Update Rails build/runtime integration to package exact Bundler and Ruby
  identity, use the site Ruby link for Puma and migrations, and reject runtime
  mismatches before bundle validation or migrations.
- [x] Update Django build/runtime integration to install requirements with the
  managed Python, emit site-link launchers and runtime identity, and reject
  mismatches before checks, migrations, or static collection.
- [x] Update dynamic Next, Nuxt, and SvelteKit services to use the site Node link
  while preserving static framework behavior and all existing artifact layouts.
- [x] Update Rails, Django, and Node placeholder setup, framework manifests,
  validation commands, systemd inputs, and AppArmor profiles for direct managed
  runtime paths without mise activation or shims.
- [x] Remove obsolete Ruby source-build and custom Node installer code, stale
  distribution Ruby/Python assumptions, unused compiler packages, and tests made
  obsolete by the managed runtime contract.
- [x] Regenerate the embedded BonesInfra wheel after Python source changes.
- [x] Update Rails, Django, and dynamic Node E2E fixtures and assertions for
  human execution, including exact commands and expected runtime versions.

## Validation

- [x] Run focused configuration and transport tests proving exact managed
  versions feed build and provisioning while remote deployment config remains
  limited to BonesRemote-owned values.
- [x] Run focused build-asset tests proving pinned verified mise installation,
  persistent cache use, compilation fallback rejection, preserved Node resolver
  behavior, and absence of obsolete runtime installers.
- [x] Run BonesInfra language, framework, manifest, systemd, AppArmor, request,
  and orchestration tests proving stable exact runtime paths and unchanged PHP
  provisioning behavior.
- [x] Run focused Rails and Django prepare/lifecycle tests proving artifact
  mismatch rejection precedes state changes and no remote application package
  installation or compilation occurs.
- [x] Run wheel consistency and materialization checks and confirm the committed
  wheel contains the managed runtime implementation and templates.
- [x] Run all non-E2E Rust and Python tests plus `cargo clippy`, `cargo fmt`,
  `ruff check .`, `ruff format .`, `shfmt -w .`, and `git diff --check`, resolving
  every warning and failure.
- [x] Leave full Rails, Django, Next, Nuxt, and SvelteKit E2E execution for a
  human and provide the exact commands and expected runtime-version evidence.

## Completion

- [x] Update current README, context, architecture, and framework build
  documentation to describe mise-managed exact runtimes, precompiled-only
  installation, stable runtime links, and PHP's explicit exclusion.
- [x] Add clarifications to the earlier Ruby and Python runtime plans that mark
  their distribution-runtime decisions as superseded by this feature.
- [x] Review the final diff for production compilation fallback, project-owned
  mise configuration execution, floating versions and registries, broad AppArmor
  access, stale `/usr/bin/ruby` or `/usr/bin/python3` assumptions, PHP changes,
  generated-file drift, and unrelated modifications.

## Completion notes

Implementation, documentation, generated assets, and automated non-E2E
validation are complete. Rails, Django, Next, Nuxt, and SvelteKit E2E execution
remains deliberately unrun for the human validation described in `e2e/README.md`.
