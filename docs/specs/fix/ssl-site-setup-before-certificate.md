# SSL Site Setup Before Certificate

<!-- Merged from fix/ssl-site-setup-before-certificate -->

## Idea

### Request

During manual testing, make site setup work before site SSL so the remaining test sites can be provisioned, certified, and deployed in that order.

### Problem

`bonesdeploy site setup --yes` fails nginx validation for a project whose configuration records SSL as enabled but whose Let's Encrypt certificate is absent. The router references the absent certificate, even though the documented workflow makes certificate acquisition a later `bonesdeploy site ssl` step.

### Definitions

**Certificate-ready:** Both required Let's Encrypt certificate files for the configured domain exist on the target server.

### Desired outcome

Site setup completes with an HTTP router when a certificate is absent, allowing the later SSL command to serve the ACME challenge and enable HTTPS. A certificate-ready site receives its HTTPS router during setup without relying on a local SSL flag.

### Scope

Certificate-derived SSL state across local configuration, provisioning, native nginx setup, manifests, next-step guidance, and focused tests.

### Constraints

Use the existing pyinfra remote facts and router rendering path. Preserve the existing SSL command as the owner of certificate acquisition and HTTPS activation. The fix is developed on the Git Flow bugfix branch requested for discovered defects.

### Exclusions

This change does not alter ACME issuance, certificate renewal, router templates, or the site SSL command's domain and email validation.


## Plan

### Current behavior

`ssl_enabled` is persisted in the Rust local configuration model and passed to the Python provisioning request. `bonesinfra.services.linux.nginx.router.setup` uses it to choose TLS rendering, while manifest reporting and next-step guidance also treat it as SSL state. Nginx validation fails when that stale flag is true and `fullchain.pem` is absent. `bonesinfra.cli.commands.site.ssl.deploy_ssl` already first renders an HTTP-only router for ACME, obtains the certificate, and then renders HTTPS.

### Intended behavior

Router setup, manifest reporting, and next-step guidance will derive SSL state from the remote certificate files. The local model and provisioning request will not carry `ssl_enabled`. Setup will render the existing HTTP-only router when no certificate exists. The SSL command will continue to make the router certificate-ready and enable TLS after acquisition.

### Approach

Remove `ssl_enabled` from the Rust configuration and provisioning data structures, the Python request/context data structures, and SSL command persistence. Use the existing pyinfra `File` fact where the provisioning process already has remote access. Reuse the existing CLI remote SSL inspection for next-step guidance. Add focused coverage for absent and present certificate states.

### Responsibilities and boundaries

The Rust configuration and transport modules own removal of obsolete local state. Python request/context modules own the reduced provisioning contract. `services/linux/nginx/router.py` and `manifest.py` own remote certificate-derived provisioning behavior. `commands/skill.rs` owns remote readiness guidance. `test_runtime_nginx.py` owns the observable setup regression checks. The SSL command remains responsible for certificate acquisition and post-acquisition HTTPS activation.

### Affected areas

`crates/bonesinfra/python/src/bonesinfra/services/linux/nginx/router.py`

`crates/bonesinfra/python/src/bonesinfra/config/`

`crates/bonesinfra/python/src/bonesinfra/manifest.py`

`crates/bonesdeploy-core/src/config/`

`crates/bonesdeploy/src/commands/skill.rs`

`crates/bonesinfra/python/tests/test_runtime_nginx.py`

`docs/plans/fix/ssl-site-setup-before-certificate/`

### Decisions

The certificate files are the sole SSL state. This eliminates a stale duplicate of remote state after server rebuilds, certificate deletion, or failed setup. Both the full-chain and key files are checked because nginx requires both.

### Risks

An unavailable or incorrectly mocked pyinfra fact could cause setup to select HTTP-only routing. The focused tests cover both certificate states. An incomplete certificate pair remains HTTP-only until `site ssl` repairs it, avoiding a global nginx validation failure.

### Validation

Run focused Rust configuration tests and the runtime nginx tests to confirm no request carries the old field and that setup renders HTTP-only without a complete certificate pair and HTTPS with one. Run the Python test suite, `ruff check .`, and `ruff format .`; then regenerate the embedded wheel, run the required Rust formatting and clippy checks, and review the final diff.


## Tasks

### Implementation

- [ ] Remove `ssl_enabled` from local configuration, provisioning transport, Python request/context, and SSL persistence.
- [ ] Determine TLS router eligibility from the remote certificate pair before router setup renders nginx.
- [ ] Derive manifest and next-step SSL state from remote certificate files.
- [ ] Cover absent, incomplete, and complete certificate states in focused configuration and runtime nginx tests.
- [ ] Regenerate the embedded BonesInfra wheel after Python source changes.

### Validation

- [ ] Run the focused runtime nginx test module and verify both certificate states.
- [ ] Run the Python test suite, Ruff check, and Ruff formatter.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`.

### Completion

- [ ] Review the diff, update completion notes, and confirm no unrelated changes are included.

### Completion notes

No implementation work has begun.


## Certificate-Derived SSL State Clarification

### Trigger

The requested fix was refined: SSL readiness must be derived from the certificate files rather than guarded by the persisted `ssl_enabled` flag.

### Decision

Remove `ssl_enabled` from the local configuration model and provisioning request. Runtime setup, manifest reporting, and next-step guidance will inspect the domain's remote Let's Encrypt certificate files. The SSL command remains responsible for obtaining certificates and rendering HTTPS, but it will no longer persist a local flag.

### Supersedes

This supersedes the plan decision to combine configured SSL with certificate presence. Certificate readiness alone determines whether HTTPS is active.

### Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` now describe certificate-derived SSL state.
