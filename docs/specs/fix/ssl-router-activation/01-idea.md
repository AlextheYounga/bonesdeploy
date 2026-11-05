# Idea

## Request

Correct the regression that prevents `bonesdeploy site ssl` from obtaining a
certificate for a site that was initially set up without a real domain. The SSL
flow must activate the real-domain Nginx router before Certbot attempts its
HTTP-01 challenge.

## Problem

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

## Definitions

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

## Desired outcome

A site initially provisioned without a domain can run `bonesdeploy site ssl`
with a real domain and complete certificate issuance. Before Certbot runs, the
challenge router is present in both `sites-available` and `sites-enabled`, the
active Nginx configuration validates, and the HTTP-01 path is publicly
reachable. Existing sites that already have an enabled router remain
idempotent.

## Scope

- Make public-router deployment own both rendering and symlink activation.
- Use that deployment operation from runtime provisioning and SSL provisioning.
- Preserve challenge-first ordering: activate HTTP routing before Certbot, then
  deploy HTTPS routing only after certificate issuance succeeds.
- Add regression coverage for router activation and SSL operation ordering.
- Normalize the touched SSL command implementation from a one-file package to
  the neighboring command convention, `site/ssl.py`.
- Regenerate the embedded BonesInfra wheel and manually retry the affected SSL
  workflow after the implementation is approved.

## Constraints

- Domainless runtime setup must remain valid and must not expose a public route.
- Quick Tunnels remain explicit and independent of real-domain routing.
- Nginx configuration must be validated before every reload.
- SSL state must not be persisted locally as enabled until certificate
  provisioning completes.
- Router deployment and removal must remain idempotent.
- Do not run the repository end-to-end test suite.

## Exclusions

- Restoring `nip.io` temporary hostnames.
- Changing Quick Tunnel installation, startup, routing, or lifecycle behavior.
- Changing Certbot's HTTP-01 challenge type or certificate renewal model.
- Requiring users to rerun site runtime setup before SSL setup.
- Broad Nginx, SSL, CLI, or provisioning refactors unrelated to router
  activation.
