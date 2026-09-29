# Tasks

## Implementation

- [x] Replace the render-only public-router operation with
  `deploy_router_config`, which renders the available configuration, enables
  the site symlink, and then performs requested validation and reload in order.
- [x] Update real-domain runtime setup to use the common router deployment
  operation and remove its duplicate inline symlink activation.
- [x] Update SSL provisioning to deploy the HTTP challenge router before
  Certbot and the HTTPS router only after certificate acquisition succeeds.
- [x] Move the single-file SSL command package to `site/ssl.py` while
  preserving its public Python import path.
- [x] Update focused router, SSL, and provisioning-boundary tests to assert the
  new operation name, activation behavior, and challenge/certificate ordering.
- [x] Regenerate the embedded `bonesinfra-0.4.0-py3-none-any.whl` after the
  Python source and tests pass.

## Validation

- [x] Run focused Python tests for runtime Nginx, SSL validation, and
  provisioning boundaries; confirm they cover the internal-only-to-SSL
  transition without changing Quick Tunnel behavior.
- [x] Run `ruff check .`, `ruff format .`, and `uv run pytest` from
  `crates/bonesinfra/python` and address every warning or failure.
- [x] Run `cargo clippy`, `cargo fmt`, and `shfmt -w .` and address every warning
  or failure without running the end-to-end suite.
- [x] Update the eligible manual projects with the rebuilt wheel, retry Laravel
  first, and confirm Certbot can fetch the HTTP-01 challenge through the active
  router and the resulting HTTPS endpoint responds.
- [x] Run SSL provisioning sequentially for the remaining doctor-green manual
  sites and record each HTTPS result; continue skipping sites whose doctor is
  not green.

## Completion

- [x] Search for stale `render_router_config` callers and confirm public-router
  activation has one implementation.
- [x] Review the final diff for unrelated changes, generated-artifact drift,
  accidental Quick Tunnel changes, and obsolete tests or documentation.
- [x] Update related context or README documentation only if implementation
  changes behavior beyond restoring the already documented SSL workflow.

## Completion notes

Implemented the approved shared router deployment operation and the later
approved SSL module-layout clarification. The SSL command moved from the
single-file `site/ssl` package to `site/ssl.py` without changing its import path.
Focused tests passed (16), the full BonesInfra suite passed (509), and Ruff,
Clippy, Rust formatting, shell formatting, wheel consistency, and diff checks
completed successfully. No README or context changes were required because the
fix restores the documented SSL workflow. Live validation on 2026-09-28 passed
for all seven doctor-green projects: each completed challenge-router activation,
certificate issuance, SSL-router activation, and returned HTTP `200` over a
publicly verified TLS connection. Django and Rails also completed SSL setup and
certificate verification when explicitly requested, but returned HTTP `502`
because their language runtimes had failed to provision earlier. The
end-to-end suite was not run.
