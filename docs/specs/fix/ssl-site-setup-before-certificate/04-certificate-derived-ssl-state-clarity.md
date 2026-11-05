# Certificate-Derived SSL State Clarification

## Trigger

The requested fix was refined: SSL readiness must be derived from the certificate files rather than guarded by the persisted `ssl_enabled` flag.

## Decision

Remove `ssl_enabled` from the local configuration model and provisioning request. Runtime setup, manifest reporting, and next-step guidance will inspect the domain's remote Let's Encrypt certificate files. The SSL command remains responsible for obtaining certificates and rendering HTTPS, but it will no longer persist a local flag.

## Supersedes

This supersedes the plan decision to combine configured SSL with certificate presence. Certificate readiness alone determines whether HTTPS is active.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` now describe certificate-derived SSL state.
