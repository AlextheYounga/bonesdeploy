# Tasks

## Implementation

- [ ] Add the pinned experimental `python-ron` dependency to BonesInfra and verify the lockfile records the selected package source and version.
- [ ] Define the Python manifest entry model and RON documents for common, framework, service, and SSL-owned paths, with every path reference resolving through `DeploymentPaths`.
- [ ] Implement manifest strategy selection from `DeployContext`, including static/server framework behavior, configured services, and SSL state.
- [ ] Implement read-only remote path inspection and stable text/JSON output in the BonesInfra command boundary.
- [ ] Add the `bonesinfra manifest show --config <path> --format <format>` CLI command and connect it to the existing PyInfra runner.
- [ ] Add the public `bonesdeploy manifest` arguments, dispatch, and delegation module without duplicating manifest parsing or strategy policy.

## Validation

- [ ] Add Python tests proving every shipped manifest RON file parses through `pyron` and preserves the required named-struct and nested-value semantics.
- [ ] Add Python resolver tests for representative strategy combinations and invalid path-key rejection.
- [ ] Add Python output tests for present, missing, and wrong-kind entries in both text and JSON formats, asserting that no file contents are emitted.
- [ ] Add Rust CLI integration tests for accepted manifest formats and missing project configuration behavior.
- [ ] Run focused Python tests and `cargo test --workspace --exclude e2e`.
- [ ] Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .`; address all warnings and errors.

## Completion

- [ ] Update relevant command and architecture documentation to describe the manifest command, RON ownership, and the experimental `pyron` dependency.
- [ ] Review the final diff for duplicated path formulas, unregistered strategy artifacts, secret output, and unintended mutations.

## Completion notes

No implementation has started. Completion notes will record parser viability, package-installation results, validation commands, and any work deliberately deferred until the `pyron` license or wheel coverage is resolved.
