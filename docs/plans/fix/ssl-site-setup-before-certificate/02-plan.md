# Plan

## Current behavior

`ssl_enabled` is persisted in the Rust local configuration model and passed to the Python provisioning request. `bonesinfra.services.linux.nginx.router.setup` uses it to choose TLS rendering, while manifest reporting and next-step guidance also treat it as SSL state. Nginx validation fails when that stale flag is true and `fullchain.pem` is absent. `bonesinfra.cli.commands.site.ssl.deploy_ssl` already first renders an HTTP-only router for ACME, obtains the certificate, and then renders HTTPS.

## Intended behavior

Router setup, manifest reporting, and next-step guidance will derive SSL state from the remote certificate files. The local model and provisioning request will not carry `ssl_enabled`. Setup will render the existing HTTP-only router when no certificate exists. The SSL command will continue to make the router certificate-ready and enable TLS after acquisition.

## Approach

Remove `ssl_enabled` from the Rust configuration and provisioning data structures, the Python request/context data structures, and SSL command persistence. Use the existing pyinfra `File` fact where the provisioning process already has remote access. Reuse the existing CLI remote SSL inspection for next-step guidance. Add focused coverage for absent and present certificate states.

## Responsibilities and boundaries

The Rust configuration and transport modules own removal of obsolete local state. Python request/context modules own the reduced provisioning contract. `services/linux/nginx/router.py` and `manifest.py` own remote certificate-derived provisioning behavior. `commands/skill.rs` owns remote readiness guidance. `test_runtime_nginx.py` owns the observable setup regression checks. The SSL command remains responsible for certificate acquisition and post-acquisition HTTPS activation.

## Affected areas

`crates/bonesinfra/python/src/bonesinfra/services/linux/nginx/router.py`

`crates/bonesinfra/python/src/bonesinfra/config/`

`crates/bonesinfra/python/src/bonesinfra/manifest.py`

`crates/bonesdeploy-core/src/config/`

`crates/bonesdeploy/src/commands/skill.rs`

`crates/bonesinfra/python/tests/test_runtime_nginx.py`

`docs/plans/fix/ssl-site-setup-before-certificate/`

## Decisions

The certificate files are the sole SSL state. This eliminates a stale duplicate of remote state after server rebuilds, certificate deletion, or failed setup. Both the full-chain and key files are checked because nginx requires both.

## Risks

An unavailable or incorrectly mocked pyinfra fact could cause setup to select HTTP-only routing. The focused tests cover both certificate states. An incomplete certificate pair remains HTTP-only until `site ssl` repairs it, avoiding a global nginx validation failure.

## Validation

Run focused Rust configuration tests and the runtime nginx tests to confirm no request carries the old field and that setup renders HTTP-only without a complete certificate pair and HTTPS with one. Run the Python test suite, `ruff check .`, and `ruff format .`; then regenerate the embedded wheel, run the required Rust formatting and clippy checks, and review the final diff.
