# SSL Router Activation

<!-- Merged from fix/ssl-router-activation -->

## Idea

### Request

Correct the regression that prevents `bonesdeploy site ssl` from obtaining a
certificate for a site that was initially set up without a real domain. The SSL
flow must activate the real-domain Nginx router before Certbot attempts its
HTTP-01 challenge.

### Problem

Before temporary `nip.io` hostnames were removed, runtime setup always rendered
and enabled a public Nginx router. The SSL flow therefore only replaced that
router's configuration and relied on its existing `sites-enabled` symlink.

Domainless runtime setup now intentionally omits public routing. When
`bonesdeploy site ssl --domain <domain>` later runs, BonesInfra renders the
HTTP-only challenge configuration under `/etc/nginx/sites-available` but does
not enable it under `/etc/nginx/sites-enabled`. Nginx validation and reload both
succeed because they inspect only the default-deny server. Public challenge
requests then receive Nginx's empty `444` response, and Certbot cannot issue the
certificate.

This breaks the documented workflow of provisioning a site first and adding a
real domain and HTTPS after DNS is ready.

### Definitions

**Internal-only site:** A provisioned site with per-site Nginx and application
services but no configured real domain and no root-managed public Nginx route.

**Public router:** The root-managed system Nginx virtual host that accepts
requests for a configured real domain and proxies them to the site's per-site
Nginx origin.

**Router deployment:** Rendering a public-router configuration, enabling its
`sites-enabled` symlink, validating the complete active Nginx configuration,
and optionally reloading Nginx. A rendered file in `sites-available` alone is
not a deployed router.

**Challenge router:** The HTTP-only public router used to serve Certbot's
HTTP-01 token before a certificate exists.

### Desired outcome

A site initially provisioned without a domain can run `bonesdeploy site ssl`
with a real domain and complete certificate issuance. Before Certbot runs, the
challenge router is present in both `sites-available` and `sites-enabled`, the
active Nginx configuration validates, and the HTTP-01 path is publicly
reachable. Existing sites that already have an enabled router remain
idempotent.

### Scope

- Make public-router deployment own both rendering and symlink activation.
- Use that deployment operation from runtime provisioning and SSL provisioning.
- Preserve challenge-first ordering: activate HTTP routing before Certbot, then
  deploy HTTPS routing only after certificate issuance succeeds.
- Add regression coverage for router activation and SSL operation ordering.
- Normalize the touched SSL command implementation from a one-file package to
  the neighboring command convention, `site/ssl.py`.
- Regenerate the embedded BonesInfra wheel and manually retry the affected SSL
  workflow after the implementation is approved.

### Constraints

- Domainless runtime setup must remain valid and must not expose a public route.
- Quick Tunnels remain explicit and independent of real-domain routing.
- Nginx configuration must be validated before every reload.
- SSL state must not be persisted locally as enabled until certificate
  provisioning completes.
- Router deployment and removal must remain idempotent.
- Do not run the repository end-to-end test suite.

### Exclusions

- Restoring `nip.io` temporary hostnames.
- Changing Quick Tunnel installation, startup, routing, or lifecycle behavior.
- Changing Certbot's HTTP-01 challenge type or certificate renewal model.
- Requiring users to rerun site runtime setup before SSL setup.
- Broad Nginx, SSL, CLI, or provisioning refactors unrelated to router
  activation.


## Plan

### Current behavior

`crates/bonesdeploy/src/commands/site/ssl.rs` loads the local configuration,
applies command-line domain and email overrides to an in-memory configuration,
and sends that configuration to BonesInfra. It writes the real domain and
`SSL_ENABLED=true` locally only after BonesInfra succeeds.

`crates/bonesinfra/python/src/bonesinfra/cli/commands/site/ssl/__init__.py`
creates the ACME webroot, installs the default-deny server, calls
`nginx_router.render_router_config` for the HTTP challenge, runs Certbot, and
calls the same rendering function for HTTPS.

`crates/bonesinfra/python/src/bonesinfra/services/linux/nginx/router.py` keeps
rendering and activation separate. `render_router_config` writes
`nginx_site_available`, optionally validates Nginx, and optionally reloads it.
Only runtime `setup` creates the `nginx_site_enabled` symlink, and it does so
only when the persisted configuration already has a real domain. Runtime
reconciliation correctly removes the public router for an internal-only site.

The separation was safe while the removed `preview_domain`/`nip.io` behavior
guaranteed that runtime setup always enabled a router. Commit `9d6b5b44` made
internal-only setup valid and moved activation behind the real-domain
condition without updating the SSL transition.

The Python SSL tests mock `render_router_config` and assert call order but do
not prove that its output is enabled. Runtime tests independently prove that an
internal-only site has no public router. No test covers the transition from
those two states.

### Intended behavior

Deploying a public router means rendering its configuration and idempotently
enabling its symlink before any requested validation or reload. Runtime setup
uses this operation when a real domain is already configured. SSL setup uses it
to activate an HTTP-only challenge router before Certbot runs, then uses it
again to replace that active configuration with HTTPS after Certbot succeeds.

Internal-only runtime setup continues to omit and reconcile away public
routing. If Certbot fails, the valid HTTP-only challenge router remains active
and SSL remains disabled locally. Rerunning SSL safely redeploys the same router
and retries issuance.

### Approach

Rename `render_router_config` to `deploy_router_config` and make the operation
render `nginx_site_available`, create or refresh the `nginx_site_enabled`
symlink with PyInfra's idempotent `files.link`, then perform the existing
optional validation and reload. The name and behavior will share one invariant:
a successfully deployed router is part of Nginx's active configuration.

Update runtime setup to call `deploy_router_config` and remove its duplicate
inline `files.link` operation. Update the SSL flow to call the same operation
for both the challenge and HTTPS stages. This places symlink activation between
rendering and validation without adding flags, duplicate activation code, or a
second router lifecycle.

Move the touched SSL command implementation from `site/ssl/__init__.py` to
`site/ssl.py`. The Python import path remains
`bonesinfra.cli.commands.site.ssl`, matching neighboring one-file site commands
without requiring caller changes.

Extend focused tests to prove operation order inside router deployment and to
prove SSL requests challenge-router deployment before certificate acquisition.
Retain the existing domainless runtime and Quick Tunnel independence tests.

### Responsibilities and boundaries

`bonesdeploy::commands::site::ssl` remains the local coordinator. It validates
CLI inputs through existing configuration loading, invokes BonesInfra, and
persists successful SSL state. It does not manipulate remote Nginx directly.

`bonesinfra.cli.commands.site.ssl` owns the certificate-provisioning sequence:
challenge router, Certbot, HTTPS router, and final etckeeper commit.

`bonesinfra.services.linux.nginx.router` owns the complete public-router
lifecycle. `deploy_router_config` renders and activates routing;
`remove_project_router` disables and removes it. Runtime setup decides whether
a configured real domain requires deployment, but it does not implement
activation separately.

The Nginx router template remains responsible for HTTP challenge and upstream
behavior. Certbot remains responsible for certificate files.

### Affected areas

- `crates/bonesinfra/python/src/bonesinfra/services/linux/nginx/router.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/site/ssl.py`
- `crates/bonesinfra/python/tests/test_runtime_nginx.py`
- `crates/bonesinfra/python/tests/test_ssl_validation.py`
- `crates/bonesinfra/python/tests/test_provisioning_boundaries.py`
- `crates/bonesinfra/assets/bonesinfra-0.4.0-py3-none-any.whl`

### Decisions

- Router rendering and activation become one operation because no current
  caller needs a rendered-but-disabled public router, and splitting them caused
  this regression.
- Activation occurs before validation so `nginx -t` checks the configuration
  that the subsequent reload will actually load.
- The SSL flow owns the transition rather than persisting the domain early or
  rerunning all runtime provisioning. This keeps the entry point focused and
  avoids marking failed SSL setup as successful configuration.
- An HTTP-only router remains after Certbot failure. It is valid, supports a
  direct retry, does not claim HTTPS is enabled, and will be removed by a later
  domainless runtime reconciliation.
- `files.link(..., force=True)` remains the activation mechanism because it is
  already used by runtime setup and is idempotent for existing sites.
- No user documentation change is required: the fix restores the workflow the
  README already documents. Planning and context documentation will only be
  adjusted if implementation reveals inaccurate behavior descriptions.
- The SSL command uses a module rather than a package because it has no
  submodules; this matches the neighboring site command layout while preserving
  its import path.

### Risks

- Enabling the router before validation means an invalid file is included in
  future Nginx validation until corrected or removed, although the running
  Nginx process is not reloaded when validation fails. The managed template and
  existing safety validation limit this risk.
- A Certbot failure leaves public HTTP routing active while local configuration
  still has an empty domain. A subsequent runtime apply removes it; an SSL retry
  recreates it. Status and tests must not incorrectly report HTTPS as enabled.
- Renaming the internal operation can leave a stale caller or test patch path.
  Repository-wide search and the full Python suite will detect this.
- The embedded wheel can drift from Python source if it is not regenerated
  after the source change.

### Validation

- A focused router test observes render, symlink activation, validation, and
  reload in that exact order.
- Focused SSL tests prove challenge-router deployment completes before Certbot,
  HTTPS-router deployment occurs only after Certbot succeeds, and failed
  issuance does not persist or deploy HTTPS state.
- Existing runtime tests continue proving an internal-only site has no public
  router and a real-domain runtime uses the common deployment operation.
- The full BonesInfra Python suite passes along with `ruff check .` and
  `ruff format .` from `crates/bonesinfra/python`.
- `cargo build-wheel` regenerates the embedded wheel, and repository checks
  confirm the generated artifact is the intended binary change.
- `cargo clippy`, `cargo fmt`, and `shfmt -w .` complete without warnings or
  errors.
- Manual validation starts from one of the already provisioned domainless test
  sites, runs `bonesdeploy site ssl --domain <domain> --email <email>`, confirms
  the router symlink and active Nginx server exist before HTTP-01 validation,
  and receives a valid HTTPS response after issuance. Remaining doctor-green
  manual sites are then provisioned sequentially.
- The end-to-end suite is not run.


## Tasks

### Implementation

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

### Validation

- [x] Run focused Python tests for runtime Nginx, SSL validation, and
  provisioning boundaries; confirm they cover the internal-only-to-SSL
  transition without changing Quick Tunnel behavior.
- [x] Run `ruff check .`, `ruff format .`, and `uv run pytest` from
  `crates/bonesinfra/python` and address every warning or failure.
- [x] Run `cargo clippy`, `cargo fmt`, and `shfmt -w .` and address every warning
  or failure without running the end-to-end suite.
- [ ] Update the eligible manual projects with the rebuilt wheel, retry Laravel
  first, and confirm Certbot can fetch the HTTP-01 challenge through the active
  router and the resulting HTTPS endpoint responds.
- [ ] Run SSL provisioning sequentially for the remaining doctor-green manual
  sites and record each HTTPS result; continue skipping sites whose doctor is
  not green.

### Completion

- [x] Search for stale `render_router_config` callers and confirm public-router
  activation has one implementation.
- [x] Review the final diff for unrelated changes, generated-artifact drift,
  accidental Quick Tunnel changes, and obsolete tests or documentation.
- [x] Update related context or README documentation only if implementation
  changes behavior beyond restoring the already documented SSL workflow.

### Completion notes

Implemented the approved shared router deployment operation and the later
approved SSL module-layout clarification. The SSL command moved from the
single-file `site/ssl` package to `site/ssl.py` without changing its import path.
Focused tests passed (16), the full BonesInfra suite passed (509), and Ruff,
Clippy, Rust formatting, shell formatting, wheel consistency, and diff checks
completed successfully. No README or context changes were required because the
fix restores the documented SSL workflow. Live SSL validation remains
uncompleted because no eligible manual project, domain, or email target was
provided. The end-to-end suite was not run.


## SSL Module Layout Clarification

### Trigger

Review of the SSL command implementation found that
`bonesinfra.cli.commands.site.ssl` remained a package only because an earlier
implementation used `ssl/plan.py`. The current package contains no submodules
and is inconsistent with the neighboring single-file site commands.

### Decision

Move the SSL command implementation from `site/ssl/__init__.py` to
`site/ssl.py`. Preserve the public import path
`bonesinfra.cli.commands.site.ssl`, so callers and test patch paths do not
change.

### Supersedes

This adds a small command-layout cleanup to the affected area without changing
the approved router-activation behavior or any public interface.

### Required Authoritative Updates

- `01-idea.md`: include normalization of the touched SSL command module in
  scope.
- `02-plan.md`: identify `site/ssl.py` as the SSL command boundary and record
  the package-to-module move.
- `03-tasks.md`: track the move and its validation.
