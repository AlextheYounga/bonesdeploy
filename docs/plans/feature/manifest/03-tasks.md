# Tasks

## Implementation

- [x] Define the typed Python manifest entry model and common, framework, service, and SSL-owned declarations, with every path reference resolving through `DeploymentPaths`.
- [x] Implement manifest strategy selection from `DeployContext`, including static/server framework behavior, configured services, and SSL state.
- [x] Implement read-only remote path inspection and stable text/JSON output in the BonesInfra command boundary.
- [x] Add the `bonesinfra manifest show --env-file <path> --format <format>` CLI command and connect it to the existing PyInfra runner.
- [x] Add the public `bonesdeploy manifest` arguments, dispatch, and delegation module without duplicating manifest parsing or strategy policy.
- [x] Expand framework, service, and runtime declarations to cover every site-specific file, directory, link, systemd unit, target membership link, AppArmor profile, and runtime path installed or managed by BonesInfra.
- [x] Add typed managed-service declarations and read-only inspection for each site-specific systemd service.

## Validation

- [x] Add Python resolver tests for representative strategy combinations and invalid path-key rejection.
- [x] Add Python output tests for present, missing, and wrong-kind entries in both text and JSON formats, asserting that no file contents are emitted.
- [x] Add Rust CLI integration tests for accepted manifest formats and missing project configuration behavior.
- [x] Add focused tests that inventory the project-derived systemd and AppArmor artifacts and verify managed-service inspection without mutations.
- [x] Run focused Python tests and `cargo test --workspace --exclude e2e`.
- [x] Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .`; address all warnings and errors.

## Completion

- [x] Update relevant command and architecture documentation to describe the manifest command, Python ownership, and JSON output.
- [x] Review the final diff for duplicated path formulas, unregistered strategy artifacts, secret output, and unintended mutations.

## Completion notes

Implemented the typed Python manifest and the existing Rust-to-Python subprocess delegation. Expanded the manifest to include project-owned setup hooks and environment, framework artifacts, project systemd units and target links, AppArmor profiles, runtime files, PHP artifacts, isolated key-value service artifacts, ACME certificates, and managed-systemd-service state. Validation passed with `uv run pytest`, `uv run ruff check .`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --exclude e2e`. E2E tests were intentionally not run.
