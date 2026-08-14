# Idea

## Request

Add a `manifest` command that shows the files associated with a project's deployment strategy. Define the manifest as typed Python declarations inside BonesInfra so the deployment strategy and the manifest share the same code boundary.

## Problem

Deployment files are currently created by several BonesInfra modules, but no single declaration explains which project-specific files belong to a selected runtime, service set, or SSL configuration. A command that only scans the remote filesystem cannot distinguish project-owned files from shared host files and cannot reliably identify files that should exist but are missing.

## Definitions

**Manifest:** A declarative inventory of every site-specific remote filesystem artifact and managed service that BonesInfra installs or manages for the configured deployment strategy. It describes paths, service identities, and their expected kinds or states; it does not contain file contents, secrets, or a filesystem snapshot.

**Manifest entry:** One typed Python declaration of a site-specific filesystem artifact or managed service, its expected kind or state, and the provisioning scope that owns it. A path value is resolved through `DeploymentPaths` or a project-derived runtime name, not repeated as an unrelated literal path in the declaration.

**Site-specific artifact:** A file, directory, link, AppArmor profile, systemd unit, systemd target membership link, runtime path, or managed service created or managed exclusively for the configured project. Shared host packages, global daemon state, and artifacts not attributable to one project are not site-specific artifacts.

**Deployment strategy:** The effective combination of the selected framework runtime, static or server mode, configured services, and SSL configuration.

**Manifest inspection:** A read-only remote check that resolves manifest entries, checks their actual filesystem state, and renders present, missing, or mismatched entries. Inspection never creates, changes, or deletes files.

**Manifest output:** The rendered result of inspecting declared manifest entries. Text is intended for people and JSON is intended for automation; JSON is not the internal manifest source.

## Desired outcome

`bonesdeploy manifest` loads the configured project, determines its deployment strategy, and displays every site-specific artifact and managed service that BonesInfra installs or manages. The output identifies expected paths and service state, reports missing or wrong filesystem types instead of silently omitting them, and has a machine-readable JSON form for automated checks.

BonesInfra collects typed manifest declarations and performs the remote inspection. Rust exposes the public command and passes its format selection to BonesInfra; Rust and Python do not maintain separate manifest schemas.

## Scope

This change includes:

- A typed BonesInfra manifest entry model and common declarations.
- Strategy-specific manifest declarations for every framework, service, and SSL artifact installed for the site, including project-derived systemd units, target membership links, AppArmor profiles, and runtime paths.
- Managed-service declarations and read-only status inspection for every site-specific systemd service.
- Resolution of manifest path references through `DeploymentPaths`.
- A read-only BonesInfra inspection command with tree and JSON output.
- A Rust `bonesdeploy manifest` command that delegates to BonesInfra.
- Tests proving typed declarations and representative strategy manifests resolve to the expected paths.

## Constraints

Manifest declarations remain internal BonesInfra Python code and do not replace `.bones/bones.toml`, which remains the user-editable project configuration.

The manifest must use existing `DeploymentPaths` values, project-derived names, and existing PyInfra remote inspection mechanisms. It must not infer ownership by scanning all of `/etc`.

The change must leave behind runnable tests and must pass the repository's Rust and Python formatting and lint checks. End-to-end tests are excluded from local validation.

## Exclusions

This change does not replace project TOML with another configuration format, migrate the existing Core default specifications, add persistent server-side manifest registry files, or report arbitrary unregistered files on the host.

It does not print file contents or secret values, repair missing files, change provisioning behavior, or inventory shared host packages and daemons that are not specific to one site.

It does not create a cross-language manifest schema or require Rust to interpret individual manifest entries.
