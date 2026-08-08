# Plan

## Current behavior

`bonesdeploy` defines top-level commands in `crates/bonesdeploy/src/cli/args.rs` and dispatches them directly from `crates/bonesdeploy/src/cli/dispatch.rs`. Existing remote operations such as `remote runtime` delegate to the embedded BonesInfra Python package through `bonesinfra::run`.

BonesInfra loads `.bones/bones.toml` into `DeployContext`, derives the project's `DeploymentPaths`, selects a framework through `bonesinfra.frameworks.get_framework`, and executes PyInfra operations through `pyinfra/runner.py`. Runtime, services, and SSL provisioning are separate BonesInfra command scopes.

Rust Core already embeds typed RON specifications under `crates/bonesdeploy-core/specs`, but those documents are consumed by Rust and are not packaged as Python data. The manifest therefore needs a BonesInfra package-local source and a Python-owned parser boundary for this change.

## Intended behavior

BonesInfra will load the manifest RON documents shipped in its embedded Python package, combine common entries with entries selected by `DeployContext`, and resolve each entry's `DeploymentPaths` key to an absolute remote path.

The inspection command will connect through the existing PyInfra runner and use read-only facts to classify each declared path as present, missing, or a filesystem-kind mismatch. It will emit a stable tree for human output and a JSON representation containing the same entries and states.

`bonesdeploy manifest` will add the public CLI variant and invoke BonesInfra with the project config and requested output format. Rust will not deserialize or reconstruct manifest entries.

## Approach

Add a focused `bonesinfra.manifest` module with typed Python manifest entries and RON loading. Keep the RON documents organized by ownership scope so the common, framework, service, and SSL declarations are readable and can be selected from the existing `DeployContext`.

The RON documents will use path-key references such as `nginx_site_available` and strategy identifiers rather than absolute paths. A resolver will validate that every referenced key exists in `DeploymentPaths` before any remote operation begins.

Add a `manifest show` BonesInfra CLI command that reuses the existing context loading and PyInfra connection lifecycle. Keep output generation separate from path resolution so JSON tests do not depend on terminal styling.

Add `bonesdeploy manifest` with a `--format text|json` option and delegate to the embedded BonesInfra runtime using the existing Rust command wrapper.

## Responsibilities and boundaries

`crates/bonesinfra/python/src/bonesinfra/manifest/` owns the RON manifest schema, source documents, strategy selection, path resolution, inspection, and output model.

`DeployContext` remains the owner of project configuration and `DeploymentPaths` remains the owner of path derivation. The manifest may read those objects but must not duplicate their path formulas.

`bonesinfra/pyinfra/runner.py` owns remote connection and operation execution. The manifest command supplies read-only inspection operations to that runner.

`bonesdeploy/src/cli` owns public argument parsing and dispatch. The Rust command module only validates the local config path and delegates; it does not own manifest policy.

## Affected areas

- `crates/bonesinfra/python/pyproject.toml` and `uv.lock` for the experimental `python-ron` dependency.
- `crates/bonesinfra/python/src/bonesinfra/manifest/` for the RON schema, documents, resolver, inspection, and output code.
- `crates/bonesinfra/python/src/bonesinfra/cli/app.py` for `manifest show`.
- `crates/bonesinfra/python/tests/` for parser, resolver, and output tests.
- `crates/bonesdeploy/src/cli/args.rs` and `cli/dispatch.rs` for the public command.
- `crates/bonesdeploy/src/commands/manifest.rs` for Rust delegation.
- `crates/bonesdeploy/tests/` for the observable CLI contract.
- `README.md`, `CONTEXT.md`, or the BonesInfra context documentation if the command and internal RON manifest need user-facing documentation.

## Decisions

- The manifest source lives inside BonesInfra because framework and service strategy selection already belongs there and the embedded Python package is the runtime that can inspect the remote host.
- Manifest paths reference `DeploymentPaths` field names instead of repeating path literals, preventing the inventory from drifting from provisioning.
- Rust delegates the manifest operation instead of parsing the RON. This keeps one manifest schema and uses the existing Rust-to-Python process boundary.
- The command reports only declared paths. Arbitrary filesystem discovery would misclassify shared host files and cannot establish ownership.
- The initial experiment uses the pinned PyPI `python-ron` release. The fork is reserved for source-level fixes or packaging improvements required by tests.
- Manifest output is read-only and contains path metadata only; it never emits file contents or secrets.

## Risks

- `pyron` may fail to install on supported workstation platforms because the tested package currently lacks complete native wheel coverage. The Python environment setup test must expose this before production adoption.
- The upstream package currently has unresolved licensing metadata, so this branch's dependency remains an experiment and cannot be treated as a release approval.
- A missing or misspelled `DeploymentPaths` key can make a strategy manifest incomplete. Resolver validation must fail before connecting to the host.
- Strategy selection can report stale paths if it does not mirror the existing static/server, framework, service, and SSL conditions. Tests must cover those combinations at the manifest boundary.
- Read-only inspection may encounter inaccessible paths. The command must report inspection failures clearly without attempting permission changes.

## Validation

- Python tests parse every shipped manifest RON document with `pyron`, including named structs, comments, trailing commas, and nested values.
- Python tests resolve representative static, server, framework, service, and SSL configurations to exact expected paths and reject unknown path keys.
- Python tests verify stable text-tree and JSON output for present, missing, and wrong-kind paths without exposing contents.
- Rust CLI tests verify `manifest` argument parsing, format forwarding, and clear failure when `.bones/bones.toml` is absent.
- Run focused Python tests and Rust workspace tests excluding `e2e`.
- Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .` as required by the affected crates.
- Review the final diff for duplicate path literals, secret leakage, and any accidental provisioning mutations.
