# Plan

## Current behavior

`bonesdeploy` defines top-level commands in `crates/bonesdeploy/src/cli/args.rs` and dispatches them directly from `crates/bonesdeploy/src/cli/dispatch.rs`. Existing remote operations such as `remote runtime` delegate to the embedded BonesInfra Python package through `bonesinfra::run`.

BonesInfra loads `.bones/bones.toml` into `DeployContext`, derives the project's `DeploymentPaths`, selects a framework through `bonesinfra.frameworks.get_framework`, and executes PyInfra operations through `pyinfra/runner.py`. Runtime, services, and SSL provisioning are separate BonesInfra command scopes.

Rust Core already embeds typed specifications under `crates/bonesdeploy-core/specs`, but those documents are consumed by Rust and are unrelated to the BonesInfra manifest. The manifest will therefore be owned directly by the Python package that selects deployment components and performs inspection.

## Intended behavior

BonesInfra will collect typed manifest entries from the enabled framework, service, and SSL components, combine them with common entries selected by `DeployContext`, and resolve each site-specific filesystem entry's `DeploymentPaths` key or project-derived name to an absolute remote path. The inventory will include every site-specific configuration file, directory, link, AppArmor profile, systemd unit, target membership link, and runtime path installed or managed by BonesInfra.

The inspection command will connect through the existing PyInfra runner and use read-only facts to classify each declared path as present, missing, or a filesystem-kind mismatch. It will inspect every declared site-specific systemd service without changing it. It will emit a stable tree for human output and a JSON representation containing the same entries and states.

`bonesdeploy manifest` will add the public CLI variant and invoke BonesInfra with the project config and requested output format. Rust will not deserialize or reconstruct manifest entries.

## Approach

Extend the focused `bonesinfra.manifest` module with typed Python entries for filesystem artifacts and managed services, grouped by ownership scope. The common, framework, service, and SSL declarations will be selected from the existing `DeployContext` without an external manifest parser. Framework and service declarations must enumerate their project-derived systemd units, AppArmor profiles, target membership links, and runtime artifacts alongside their existing placeholders and configuration paths.

The typed declarations will use path-key references such as `nginx_site_available`, or project-derived names where `DeploymentPaths` does not yet expose the value, rather than unrelated absolute-path formulas. A resolver will validate that every referenced key and derived name is valid before any remote operation begins.

Add a `manifest show` BonesInfra CLI command that reuses the existing context loading and PyInfra connection lifecycle. Keep output generation separate from path resolution so JSON tests do not depend on terminal styling.

Add `bonesdeploy manifest` with a `--format text|json` option and delegate to the embedded BonesInfra runtime using the existing Rust command wrapper.

## Responsibilities and boundaries

`crates/bonesinfra/python/src/bonesinfra/manifest.py` owns the typed declarations, strategy selection, filesystem and service resolution, inspection, and output model.

`DeployContext` remains the owner of project configuration and `DeploymentPaths` remains the owner of reusable path derivation. The manifest may read those objects and derive only names that are inherently runtime-specific, such as a framework's project-qualified systemd service and AppArmor profile.

`bonesinfra/pyinfra/runner.py` owns remote connection and operation execution. The manifest command supplies read-only inspection operations to that runner.

`bonesdeploy/src/cli` owns public argument parsing and dispatch. The Rust command module only validates the local config path and delegates; it does not own manifest policy.

## Affected areas

- `crates/bonesinfra/python/src/bonesinfra/manifest.py` for the typed declarations, resolver, inspection, and output code.
- `crates/bonesinfra/python/src/bonesinfra/cli/app.py` for `manifest show`.
- `crates/bonesinfra/python/tests/` for declaration, resolver, and output tests.
- `crates/bonesdeploy/src/cli/args.rs` and `cli/dispatch.rs` for the public command.
- `crates/bonesdeploy/src/commands/manifest.rs` for Rust delegation.
- `crates/bonesdeploy/tests/` for the observable CLI contract.
- `README.md`, `CONTEXT.md`, or the BonesInfra context documentation if the command and Python-owned manifest need user-facing documentation.

## Decisions

- The manifest source lives inside BonesInfra because framework and service strategy selection already belongs there and the embedded Python package is the runtime that can inspect the remote host.
- The v1 manifest source is typed Python code rather than RON or JSON because Rust only dispatches the subprocess and does not need to interpret manifest entries.
- Manifest paths reference `DeploymentPaths` field names instead of repeating path literals, preventing the inventory from drifting from provisioning.
- Rust delegates the manifest operation instead of interpreting manifest entries. This keeps one manifest source and uses the existing Rust-to-Python process boundary.
- The command reports only declared paths. Arbitrary filesystem discovery would misclassify shared host files and cannot establish ownership.
- Every site-specific artifact and managed service installed or managed by BonesInfra belongs in the manifest, including framework application units, per-site nginx, target membership links, AppArmor profiles, and project runtime paths. Shared host packages, daemons, and other non-project artifacts do not belong in this inventory.
- JSON is an output format for automation, not the internal manifest source. Manifest output is read-only and contains path metadata only; it never emits file contents or secrets.

## Risks

- A missing or misspelled `DeploymentPaths` key can make a strategy manifest incomplete. Resolver validation must fail before connecting to the host.
- Strategy selection can report stale paths if it does not mirror the existing static/server, framework, service, and SSL conditions. Tests must cover those combinations at the manifest boundary.
- Read-only inspection may encounter inaccessible paths. The command must report inspection failures clearly without attempting permission changes.

## Validation

- Python tests resolve representative static, server, framework, service, and SSL configurations to every expected site-specific path and service, including project-derived systemd and AppArmor artifacts, and reject unknown path keys.
- Python tests verify stable text-tree and JSON output for present, missing, and wrong-kind paths without exposing contents.
- Rust CLI tests verify `manifest` argument parsing, format forwarding, and clear failure when `.bones/bones.toml` is absent.
- Run focused Python tests and Rust workspace tests excluding `e2e`.
- Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .` as required by the affected crates.
- Review the final diff for duplicate path literals, secret leakage, and any accidental provisioning mutations.
