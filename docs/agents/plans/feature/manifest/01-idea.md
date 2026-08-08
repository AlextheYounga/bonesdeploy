# Idea

## Request

Add a `manifest` command that shows the files associated with a project's deployment strategy. Define the manifest in RON files under BonesInfra so the deployment strategy and the manifest use one readable declaration.

## Problem

Deployment files are currently created by several BonesInfra modules, but no single declaration explains which project-specific files belong to a selected runtime, service set, or SSL configuration. A command that only scans the remote filesystem cannot distinguish project-owned files from shared host files and cannot reliably identify files that should exist but are missing.

## Definitions

**Manifest:** A declarative list of project-related remote paths expected by BonesInfra for the configured deployment strategy. It describes paths and their kinds; it does not contain file contents, secrets, or a filesystem snapshot.

**Manifest entry:** One named declaration in a manifest RON document. An entry identifies a path value, its filesystem kind, and the provisioning scope that owns it. A path value is resolved through `DeploymentPaths`, not repeated as a literal path in the manifest.

**Deployment strategy:** The effective combination of the selected framework runtime, static or server mode, configured services, and SSL configuration.

**Manifest inspection:** A read-only remote check that resolves manifest entries, checks their actual filesystem state, and renders present, missing, or mismatched entries. Inspection never creates, changes, or deletes files.

**RON manifest source:** A versioned RON document shipped inside the embedded BonesInfra Python package. It is an internal strategy specification, not a user-editable project configuration file.

## Desired outcome

`bonesdeploy manifest` loads the configured project, determines its deployment strategy, and displays a tree of the declared project-related remote files. The output identifies expected paths and reports missing or wrong filesystem types instead of silently omitting them. The command has a machine-readable JSON form for automated checks.

BonesInfra parses the RON manifest source and performs the remote inspection. Rust exposes the public command and passes its format selection to BonesInfra; Rust and Python do not maintain separate manifest schemas.

## Scope

This change includes:

- A BonesInfra RON manifest schema and common manifest declarations.
- Strategy-specific manifest declarations for framework, services, and SSL artifacts.
- Resolution of manifest path references through `DeploymentPaths`.
- A read-only BonesInfra inspection command with tree and JSON output.
- A Rust `bonesdeploy manifest` command that delegates to BonesInfra.
- Tests proving the shipped RON documents parse and representative strategy manifests resolve to the expected paths.

## Constraints

RON manifest files remain internal embedded BonesInfra assets and do not replace `.bones/bones.toml`, which remains the user-editable project configuration.

The manifest must use existing `DeploymentPaths` values and existing PyInfra remote inspection mechanisms. It must not infer ownership by scanning all of `/etc`.

`pyron` is used as an experimental dependency pinned to the tested release while its licensing and distribution status are evaluated. The experiment must not be represented as a production release guarantee.

The change must leave behind runnable tests and must pass the repository's Rust and Python formatting and lint checks. End-to-end tests are excluded from local validation.

## Exclusions

This change does not replace project TOML with RON, migrate the existing Core default specifications, add persistent server-side manifest registry files, or report arbitrary unregistered files on the host.

It does not print file contents or secret values, repair missing files, or change provisioning behavior beyond making the declared manifest paths the source used by inspection.

It does not publish or release `pyron` as a BonesDeploy dependency until its license and supported wheel coverage are resolved.
