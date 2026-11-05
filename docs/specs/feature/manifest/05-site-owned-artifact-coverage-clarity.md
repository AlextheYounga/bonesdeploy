# Clarification

## Trigger

The user clarified that the manifest must map all files and services related to one site: everything installed onto the server that affects that site's operation in particular.

## Decision

The manifest is the complete inventory of site-specific artifacts and managed services installed or managed by BonesInfra. It includes every project-exclusive configuration file, directory, link, systemd unit, target membership link, AppArmor profile, runtime path, framework artifact, service artifact, and SSL artifact. It also includes each project-specific systemd service as a managed service for read-only status inspection.

Site-specific means the artifact is attributable exclusively to the configured project, normally through its `DeploymentPaths` value or project-derived name. Shared host packages, global daemon state, and files not specific to one project remain outside the manifest. The manifest remains an inventory and status check: it does not render contents or validate cgroup, hardening, or other configuration settings inside a unit or profile file.

## Supersedes

This expands the earlier v1 scope that described framework, services, and SSL declarations without requiring complete coverage of every installed site-specific artifact and managed service.

## Effect on the record

`01-idea.md` defines a manifest as a complete site-specific artifact and managed-service inventory. `02-plan.md` assigns complete systemd, AppArmor, registration-link, and runtime-artifact coverage to the manifest. `03-tasks.md` adds the remaining implementation and validation work for that coverage.
