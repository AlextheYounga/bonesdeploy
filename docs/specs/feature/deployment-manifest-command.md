# Deployment Manifest Command

<!-- Merged from feature/manifest -->

## Idea

### Request

Add a `manifest` command that shows the files associated with a project's deployment strategy. Define the manifest as typed Python declarations inside BonesInfra so the deployment strategy and the manifest share the same code boundary.

### Problem

Deployment files are currently created by several BonesInfra modules, but no single declaration explains which project-specific files belong to a selected runtime, service set, or SSL configuration. A command that only scans the remote filesystem cannot distinguish project-owned files from shared host files and cannot reliably identify files that should exist but are missing.

### Definitions

**Manifest:** A declarative inventory of every site-specific remote filesystem artifact and managed service that BonesInfra installs or manages for the configured deployment strategy. It describes paths, service identities, and their expected kinds or states; it does not contain file contents, secrets, or a filesystem snapshot.

**Manifest entry:** One typed Python declaration of a site-specific filesystem artifact or managed service, its expected kind or state, and the provisioning scope that owns it. A path value is resolved through `DeploymentPaths` or a project-derived runtime name, not repeated as an unrelated literal path in the declaration.

**Site-specific artifact:** A file, directory, link, AppArmor profile, systemd unit, systemd target membership link, runtime path, or managed service created or managed exclusively for the configured project. Shared host packages, global daemon state, and artifacts not attributable to one project are not site-specific artifacts.

**Deployment strategy:** The effective combination of the selected framework runtime, static or server mode, configured services, and SSL configuration.

**Manifest inspection:** A read-only remote check that resolves manifest entries, checks their actual filesystem state, and renders present, missing, or mismatched entries. Inspection never creates, changes, or deletes files.

**Manifest output:** The rendered result of inspecting declared manifest entries. Text is intended for people and JSON is intended for automation; JSON is not the internal manifest source.

### Desired outcome

`bonesdeploy manifest` loads the configured project, determines its deployment strategy, and displays every site-specific artifact and managed service that BonesInfra installs or manages. The output identifies expected paths and service state, reports missing or wrong filesystem types instead of silently omitting them, and has a machine-readable JSON form for automated checks.

BonesInfra collects typed manifest declarations, performs the remote inspection, and returns a secret-free JSON report. Rust owns the public text-tree and JSON presentation through a narrow report DTO; Python does not render public CLI output.

### Scope

This change includes:

- A typed BonesInfra manifest entry model and common declarations.
- Strategy-specific manifest declarations for every framework, service, and SSL artifact installed for the site, including project-derived systemd units, target membership links, AppArmor profiles, and runtime paths.
- Managed-service declarations and read-only status inspection for every site-specific systemd service.
- Resolution of manifest path references through `DeploymentPaths`.
- A read-only BonesInfra inspection command that returns the inspected JSON report.
- A Rust `bonesdeploy manifest` command that delegates to BonesInfra.
- Tests proving typed declarations and representative strategy manifests resolve to the expected paths.

### Constraints

Manifest declarations remain internal BonesInfra Python code and do not replace `.bones/bones.toml`, which remains the user-editable project configuration.

The manifest must use existing `DeploymentPaths` values, project-derived names, and existing PyInfra remote inspection mechanisms. It must not infer ownership by scanning all of `/etc`.

The change must leave behind runnable tests and must pass the repository's Rust and Python formatting and lint checks. End-to-end tests are excluded from local validation.

### Exclusions

This change does not replace project TOML with another configuration format, migrate the existing Core default specifications, add persistent server-side manifest registry files, or report arbitrary unregistered files on the host.

It does not print file contents or secret values, repair missing files, change provisioning behavior, or inventory shared host packages and daemons that are not specific to one site.

It does not create a cross-language manifest schema or require Rust to interpret individual manifest entries.


## Plan

### Current behavior

`bonesdeploy` defines top-level commands in `crates/bonesdeploy/src/cli/args.rs` and dispatches them directly from `crates/bonesdeploy/src/cli/dispatch.rs`. Existing remote operations such as `remote runtime` delegate to the embedded BonesInfra Python package through `bonesinfra::run`.

BonesInfra loads `.bones/bones.toml` into `DeployContext`, derives the project's `DeploymentPaths`, selects a framework through `bonesinfra.frameworks.get_framework`, and executes PyInfra operations through `pyinfra/runner.py`. Runtime, services, and SSL provisioning are separate BonesInfra command scopes.

Rust Core already embeds typed specifications under `crates/bonesdeploy-core/specs`, but those documents are consumed by Rust and are unrelated to the BonesInfra manifest. The manifest will therefore be owned directly by the Python package that selects deployment components and performs inspection.

### Intended behavior

BonesInfra will collect typed manifest entries from the enabled framework, service, and SSL components, combine them with common entries selected by `DeployContext`, and resolve each site-specific filesystem entry's `DeploymentPaths` key or project-derived name to an absolute remote path. The inventory will include every site-specific configuration file, directory, link, AppArmor profile, systemd unit, target membership link, and runtime path installed or managed by BonesInfra.

The inspection command will connect through the existing PyInfra runner and use read-only facts to classify each declared path as present, missing, or a filesystem-kind mismatch. It will inspect every declared site-specific systemd service without changing it and return a JSON report to Rust. Rust will emit the stable, colored tree for human output or pass through JSON for automation.

`bonesdeploy manifest` will add the public CLI variant and invoke BonesInfra with the project config and requested output format. Rust will not deserialize or reconstruct manifest entries.

### Approach

Extend the focused `bonesinfra.manifest` module with typed Python entries for filesystem artifacts and managed services, grouped by ownership scope. The common, framework, service, and SSL declarations will be selected from the existing `DeployContext` without an external manifest parser. Framework and service declarations must enumerate their project-derived systemd units, AppArmor profiles, target membership links, and runtime artifacts alongside their existing placeholders and configuration paths.

The typed declarations will use path-key references such as `nginx_site_available`, or project-derived names where `DeploymentPaths` does not yet expose the value, rather than unrelated absolute-path formulas. A resolver will validate that every referenced key and derived name is valid before any remote operation begins.

Add a `manifest show` BonesInfra CLI command that reuses the existing context loading and PyInfra connection lifecycle and serializes the inspection report as JSON. Keep public output generation in Rust so Python inspection does not depend on terminal styling.

Add `bonesdeploy manifest` with a `--format text|json` option and delegate to the embedded BonesInfra runtime using the existing Rust command wrapper.

### Responsibilities and boundaries

`crates/bonesinfra/python/src/bonesinfra/manifest.py` owns the typed declarations, strategy selection, filesystem and service resolution, inspection, and serialized report model.

`DeployContext` remains the owner of project configuration and `DeploymentPaths` remains the owner of reusable path derivation. The manifest may read those objects and derive only names that are inherently runtime-specific, such as a framework's project-qualified systemd service and AppArmor profile.

`bonesinfra/pyinfra/runner.py` owns remote connection and operation execution. The manifest command supplies read-only inspection operations to that runner.

`bonesdeploy/src/cli` owns public argument parsing and dispatch. The Rust command module validates the local config path, delegates inspection, deserializes the narrow report DTO, and renders public text or JSON without owning manifest policy.

### Affected areas

- `crates/bonesinfra/python/src/bonesinfra/manifest.py` for the typed declarations, resolver, inspection, and output code.
- `crates/bonesinfra/python/src/bonesinfra/cli/app.py` for `manifest show`.
- `crates/bonesinfra/python/tests/` for declaration, resolver, and output tests.
- `crates/bonesdeploy/src/cli/args.rs` and `cli/dispatch.rs` for the public command.
- `crates/bonesdeploy/src/commands/manifest.rs` for Rust delegation.
- `crates/bonesdeploy/tests/` for the observable CLI contract.
- `README.md`, `CONTEXT.md`, or the BonesInfra context documentation if the command and Python-owned manifest need user-facing documentation.

### Decisions

- The manifest source lives inside BonesInfra because framework and service strategy selection already belongs there and the embedded Python package is the runtime that can inspect the remote host.
- The v1 manifest source is typed Python code rather than RON or JSON because Rust only dispatches the subprocess and does not need to interpret manifest entries.
- Manifest paths reference `DeploymentPaths` field names instead of repeating path literals, preventing the inventory from drifting from provisioning.
- Rust interprets the secret-free inspection report only for public presentation. Python remains the sole source of manifest declarations and inspection policy.
- The command reports only declared paths. Arbitrary filesystem discovery would misclassify shared host files and cannot establish ownership.
- Every site-specific artifact and managed service installed or managed by BonesInfra belongs in the manifest, including framework application units, per-site nginx, target membership links, AppArmor profiles, and project runtime paths. Shared host packages, daemons, and other non-project artifacts do not belong in this inventory.
- JSON is an output format for automation, not the internal manifest source. Manifest output is read-only and contains path metadata only; it never emits file contents or secrets.

### Risks

- A missing or misspelled `DeploymentPaths` key can make a strategy manifest incomplete. Resolver validation must fail before connecting to the host.
- Strategy selection can report stale paths if it does not mirror the existing static/server, framework, service, and SSL conditions. Tests must cover those combinations at the manifest boundary.
- Read-only inspection may encounter inaccessible paths. The command must report inspection failures clearly without attempting permission changes.

### Validation

- Python tests resolve representative static, server, framework, service, and SSL configurations to every expected site-specific path and service, including project-derived systemd and AppArmor artifacts, and reject unknown path keys.
- Python tests verify stable text-tree and JSON output for present, missing, and wrong-kind paths without exposing contents.
- Rust CLI tests verify `manifest` argument parsing, format forwarding, and clear failure when `.bones/bones.toml` is absent.
- Run focused Python tests and Rust workspace tests excluding `e2e`.
- Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .` as required by the affected crates.
- Review the final diff for duplicate path literals, secret leakage, and any accidental provisioning mutations.


## Tasks

### Implementation

- [x] Define the typed Python manifest entry model and common, framework, service, and SSL-owned declarations, with every path reference resolving through `DeploymentPaths`.
- [x] Implement manifest strategy selection from `DeployContext`, including static/server framework behavior, configured services, and SSL state.
- [x] Implement read-only remote path inspection and stable text/JSON output in the BonesInfra command boundary.
- [x] Add the JSON-only `bonesinfra manifest show --env-file <path>` CLI command and connect it to the existing PyInfra runner.
- [x] Add the public `bonesdeploy manifest` arguments, dispatch, and delegation module without duplicating manifest parsing or strategy policy.
- [x] Expand framework, service, and runtime declarations to cover every site-specific file, directory, link, systemd unit, target membership link, AppArmor profile, and runtime path installed or managed by BonesInfra.
- [x] Add typed managed-service declarations and read-only inspection for each site-specific systemd service.

### Validation

- [x] Add Python resolver tests for representative strategy combinations and invalid path-key rejection.
- [x] Add Python output tests for present, missing, and wrong-kind entries in both text and JSON formats, asserting that no file contents are emitted.
- [x] Add Rust CLI integration tests for accepted manifest formats and missing project configuration behavior.
- [x] Add focused tests that inventory the project-derived systemd and AppArmor artifacts and verify managed-service inspection without mutations.
- [x] Run focused Python tests and `cargo test --workspace --exclude e2e`.
- [x] Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff check .`, and `ruff format .`; address all warnings and errors.

### Completion

- [x] Update relevant command and architecture documentation to describe the manifest command, Python ownership, and JSON output.
- [x] Review the final diff for duplicated path formulas, unregistered strategy artifacts, secret output, and unintended mutations.

### Completion notes

Implemented the typed Python manifest and the existing Rust-to-Python subprocess delegation. Expanded the manifest to include project-owned setup hooks and environment, framework artifacts, project systemd units and target links, AppArmor profiles, runtime files, PHP artifacts, isolated key-value service artifacts, ACME certificates, and managed-systemd-service state. Validation passed with `uv run pytest`, `uv run ruff check .`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --exclude e2e`. E2E tests were intentionally not run.

The presentation boundary was later simplified: BonesInfra now returns JSON only, while Rust renders the colored status tree or passes JSON through unchanged. Obsolete Python tree and color rendering were removed.


## Clarification

### Trigger

The user chose a Python-owned typed manifest for v1 and rejected RON and JSON as internal manifest source formats.

### Decision

Manifest entries will be ordinary typed Python declarations inside BonesInfra. BonesDeploy will invoke BonesInfra through the existing subprocess CLI boundary. JSON remains available only as rendered command output for automation. This avoids a parser dependency, duplicated cross-language schemas, and drift between manifest data and BonesInfra strategy logic.

### Supersedes

This supersedes the earlier decision to use RON documents and the experimental `python-ron` dependency as the v1 manifest source.

### Effect on the record

`01-idea.md` now defines typed Python declarations and JSON output. `02-plan.md` now assigns manifest ownership to Python code and removes parser/package work. `03-tasks.md` now tracks typed declarations, subprocess delegation, inspection, output, tests, and documentation without RON or JSON source files.


## Clarification

### Trigger

The user clarified that the manifest must map all files and services related to one site: everything installed onto the server that affects that site's operation in particular.

### Decision

The manifest is the complete inventory of site-specific artifacts and managed services installed or managed by BonesInfra. It includes every project-exclusive configuration file, directory, link, systemd unit, target membership link, AppArmor profile, runtime path, framework artifact, service artifact, and SSL artifact. It also includes each project-specific systemd service as a managed service for read-only status inspection.

Site-specific means the artifact is attributable exclusively to the configured project, normally through its `DeploymentPaths` value or project-derived name. Shared host packages, global daemon state, and files not specific to one project remain outside the manifest. The manifest remains an inventory and status check: it does not render contents or validate cgroup, hardening, or other configuration settings inside a unit or profile file.

### Supersedes

This expands the earlier v1 scope that described framework, services, and SSL declarations without requiring complete coverage of every installed site-specific artifact and managed service.

### Effect on the record

`01-idea.md` defines a manifest as a complete site-specific artifact and managed-service inventory. `02-plan.md` assigns complete systemd, AppArmor, registration-link, and runtime-artifact coverage to the manifest. `03-tasks.md` adds the remaining implementation and validation work for that coverage.


## Rust Presentation Clarity

### Trigger

The Python-rendered text tree made terminal color propagation unreliable and split public CLI presentation across the Rust and Python process boundary.

### Decision

BonesInfra returns only the inspected JSON report. The public Rust command deserializes a narrow report DTO and owns tree construction, status-only text, terminal color, and JSON passthrough. Artifact kinds, owners, and detailed service state remain available in JSON but are omitted from the human tree.

### Supersedes

This supersedes the decision that Rust delegates format selection and does not interpret manifest entries.

### Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are updated to reflect the Rust presentation boundary.
