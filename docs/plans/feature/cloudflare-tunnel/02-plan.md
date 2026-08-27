# Plan

## Current behavior

The site's DNS configuration contains only explicitly configured real-domain
values. Temporary ingress is selected at runtime when that domain is empty.

The remote provisioning request and project scaffold do not contain temporary
hostnames.

BonesInfra reads the real domain into `DnsConfig` in
`crates/bonesinfra/python/src/bonesinfra/config/context.py`. The router uses
that value as the public Nginx `server_name` and does not require a hostname for
per-site Nginx. The router also owns creation of the per-site
Nginx systemd service, its runtime directories, the root-managed public Nginx
site, and the site target startup sequence.

`assets/nginx/router.conf.j2` proxies the configured public hostname to the
project's per-site Nginx Unix socket. The Quick Tunnel connects directly to
that socket and does not use the public router.

The per-site Nginx configuration already listens on
`paths.runtime_nginx_socket`. The socket is the stable local ingress boundary
used by the root-managed public Nginx router. The site's systemd target already
provides a project-scoped lifecycle for `project-nginx.service` and other
runtime services.

`crates/bonesinfra/python/src/bonesinfra/manifest.py` treats site Nginx as a
runtime-managed service and associates SSL artifacts only with a real domain.

`crates/bonesremote/src/commands/status.rs` reports the current release, SSL
state, and project services. `crates/bonesdeploy/src/commands/status.rs`
renders that report but has no temporary-ingress state or URL.

`crates/bonesdeploy/src/commands/setup.rs` applies bootstrap, services, state,
and runtime, then directs a site without configured SSL toward
`bonesdeploy remote ssl`. It does not distinguish a no-domain temporary preview
from a site that is ready for real-domain TLS.

Cloudflare Quick Tunnels can create an account-less random
`*.trycloudflare.com` HTTPS hostname and proxy to a local service. Cloudflared
supports Unix-socket HTTP origins, so the existing per-site Nginx socket can be
used directly without adding a loopback port or routing the temporary hostname
through system Nginx.

## Intended behavior

A site with an empty `app.dns.domain` uses a project-scoped
`<project>-cloudflared.service` as its public temporary ingress. The service
starts `cloudflared` in Quick Tunnel mode with the per-site Nginx Unix socket as
its origin. It is required by the site's existing systemd target and ordered
after the site's Nginx service so the origin socket exists before the tunnel
starts.

The root-managed public Nginx router is not required for a no-domain site.
BonesInfra removes any project public-router config when no real domain exists
while retaining the machine's default-deny Nginx configuration.

A site with a non-empty real domain uses the existing public Nginx route.
Runtime reconciliation ensures the project Quick Tunnel unit is stopped,
removed from the site target, and removed from disk when a real domain owns
ingress. During `remote ssl`, the Quick Tunnel remains available through the
certificate challenge and real-domain Nginx activation, then is removed after
the HTTPS router has been successfully enabled.

The current Quick Tunnel hostname is discovered from the active project's
`cloudflared` systemd journal. `bonesremote status` reports preview state and
the most recent valid `https://*.trycloudflare.com` URL emitted by the active
service. `bonesdeploy status` renders that URL. Setup reuses the same remote
status path to print the preview URL after runtime setup without persisting the
hostname into project configuration.

New configuration contains no temporary hostname. BonesInfra does not carry a
temporary hostname in its typed context or template data.

## Approach

Remove temporary-hostname semantics from the shared configuration model. Remove
the field from the scaffold and from BonesInfra's `DnsConfig` and template
context.

Add a focused `bonesinfra.services.linux.cloudflared` module. It owns the
Cloudflare package repository/package installation and the project Quick Tunnel
systemd lifecycle. Install the Cloudflare signing key and APT source
idempotently, install the `cloudflared` package, render a dedicated
project-scoped systemd unit, register that unit with the existing site target,
and reconcile its absence for real-domain sites.

The Quick Tunnel unit runs as the site's runtime user and executes
`/usr/bin/cloudflared tunnel --no-autoupdate --url
unix:<runtime_nginx_socket>`. The unit declares the project Nginx service as an
ordering/dependency requirement and restarts on failure. It uses journald for
cloudflared output; no Cloudflare config file, credential file, account token,
PID file, or generated URL file is introduced.

Separate "site Nginx must exist" from "public Nginx router must exist" inside
the existing Nginx runtime flow. The per-site Nginx service, runtime
directories, and site target registration remain unconditional. Public-router
rendering/enabling becomes conditional on a real domain. When no real domain
exists, remove the project router symlink/config left by prior runs, validate
the remaining Nginx configuration, and reload system Nginx only when router
state changes.

Have `services/linux/runtime.py` reconcile exactly one ingress owner after the
per-site Nginx pieces are prepared: real domain means public Nginx and no Quick
Tunnel; empty domain means Quick Tunnel and no project public router. Keep
default-deny Nginx setup intact.

Extend the SSL deploy sequence so successful real-domain activation is the
handoff point from temporary ingress to permanent ingress. Obtain the
certificate and enable the HTTPS Nginx router first, then stop/unregister/remove
the project's Quick Tunnel unit.

Extend `bonesremote status` with a preview section. It checks the
project-cloudflared unit and only reports a URL while the unit is active. Read
the unit's current-boot journal and select the last token that is a valid HTTPS
`trycloudflare.com` hostname. Keep this parser small and unit-tested; do not
depend on the surrounding human log message. A restart naturally changes the
reported URL because the newest journal hostname wins.

Extend the local status representation/rendering for the preview section and
expose the existing remote-status fetch operation for setup reuse. Setup prints
the current preview URL for a no-domain site after runtime application and no
longer sends that site toward `remote ssl`. Real-domain setup guidance retains
the existing SSL/deploy behavior.

Update the BonesInfra manifest so `project-cloudflared.service` is expected only
for a no-domain site. Remove preview-domain-based SSL artifact resolution; SSL
artifacts are associated only with a real domain.

## Responsibilities and boundaries

`bonesdeploy-core` owns the persisted configuration contract and stores only
real domain state.

`bonesdeploy::commands::setup` remains the setup coordinator. It does not start
or inspect cloudflared directly; it uses the runtime provisioning flow and the
same remote status contract used by `bonesdeploy status`.

`bonesdeploy::commands::status` owns user-facing rendering of the preview URL
and the SSH request for the structured `bonesremote status` report.

`bonesinfra.services.linux.runtime` remains the coordinator for site runtime
infrastructure and chooses the ingress mode from the settled rule: a real
domain uses the public Nginx router; no real domain uses a Quick Tunnel.

`bonesinfra.services.linux.nginx` owns per-site Nginx and real-domain public
Nginx configuration. It no longer invents or interprets temporary hostnames.

`bonesinfra.services.linux.cloudflared` owns Cloudflare package installation,
the project Quick Tunnel unit, site-target registration, and removal of that
unit when temporary ingress is not applicable.

`bonesinfra.cli.commands.ssl` owns the safe ingress handoff during real-domain
TLS activation: permanent HTTPS is activated before the Quick Tunnel is
removed.

`bonesremote::commands::status` owns observation of remote runtime state. It
reports the active Quick Tunnel URL from the project service journal but does
not create, restart, or configure the tunnel.

## Affected areas

- `crates/bonesdeploy-core/src/app.rs`
- `crates/bonesdeploy-core/src/config.rs`
- Rust configuration tests covering the real-domain configuration contract
- `crates/bonesdeploy/assets/kit/bones.toml`
- `crates/bonesdeploy/src/commands/remote/data.rs`
- `crates/bonesdeploy/src/commands/setup.rs`
- `crates/bonesdeploy/src/commands/status.rs`
- `crates/bonesremote/src/commands/status.rs`
- `crates/bonesinfra/python/src/bonesinfra/config/context.py`
- `crates/bonesinfra/python/src/bonesinfra/services/linux/runtime.py`
- `crates/bonesinfra/python/src/bonesinfra/services/linux/nginx/router.py`
- `crates/bonesinfra/python/src/bonesinfra/assets/nginx/router.conf.j2`
- New `crates/bonesinfra/python/src/bonesinfra/services/linux/cloudflared.py`
- New project-scoped Quick Tunnel systemd unit template under
  `crates/bonesinfra/python/src/bonesinfra/assets/systemd/`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/ssl/__init__.py`
- `crates/bonesinfra/python/src/bonesinfra/manifest.py`
- `crates/bonesinfra/python/tests/test_context.py`
- `crates/bonesinfra/python/tests/test_runtime_nginx.py`
- New focused BonesInfra tests for cloudflared package/unit/lifecycle behavior
- Focused Rust status/config/setup tests
- `README.md`, `CONTEXT.md`, and `crates/bonesinfra/python/CONTEXT.md`

## Decisions

- Cloudflare Quick Tunnels provide no-domain ingress rather than becoming a
  second configurable preview provider. The request is for a cleaner no-domain
  temporary URL, and a provider framework would add unsupported generality.
- Temporary ingress is inferred from the absence of a real domain. No new
  `preview_enabled`, provider, hostname, token, or tunnel configuration is
  added.
- The Quick Tunnel connects directly to the per-site Nginx Unix socket. This
  reuses the existing local ingress boundary and avoids allocating ports or
  passing a random Cloudflare Host header through system Nginx.
- Each site gets its own systemd-managed `cloudflared` process. A global
  cloudflared service would mix project lifecycles and origins.
- The generated URL is observed from journald rather than stored as
  configuration. The URL belongs to the current process and can change after a
  restart.
- No generated URL state file is introduced. Remote status derives current
  state from the service that owns it, avoiding stale hostname persistence.
- Temporary ingress is never accepted as persisted configuration or serialized
  project state.
- The real-domain SSL flow disables the Quick Tunnel only after the HTTPS Nginx
  route is active, preserving temporary access during the handoff.
- The `cloudflared` package remains installed after a site switches to a real
  domain because package ownership is server-wide and another site can require
  it; only the project-specific service is removed.
- Quick Tunnel limitations are accepted because this feature is explicitly
  temporary preview infrastructure, not production ingress.

## Risks

- Cloudflare can change the human-readable startup log around the generated
  hostname. Status parsing mitigates this by recognizing the URL token rather
  than matching the surrounding sentence, but a format that stops emitting the
  hostname would require adaptation.
- `cloudflared` package repository availability or outbound network failures can
  prevent a no-domain runtime from obtaining temporary ingress.
- Unix-socket origin support or permissions can fail if the generated unit runs
  under a user that cannot traverse or connect to the project's Nginx socket.
- Incorrect systemd ordering can start cloudflared before the per-site Nginx
  socket exists and cause transient origin failures.
- Reconciliation can leave a stale Nginx site enabled if the no-domain
  path removes only the symlink or only the available config.
- The real-domain handoff can create downtime if the Quick Tunnel is removed
  before certificate issuance, Nginx validation, and HTTPS reload have
  succeeded.
- Applications that require Server-Sent Events cannot be fully exercised
  through Quick Tunnels.

## Validation

- Add Rust configuration tests proving temporary hostnames are not part of
  persisted configuration.
- Add BonesInfra context tests proving temporary hostnames are not part of the
  typed DNS/template context.
- Add focused cloudflared provisioning tests proving a no-domain site installs
  and renders the project Quick Tunnel service, uses the per-site Nginx Unix
  socket, runs under the runtime user, orders after project Nginx, and
  registers with the site target.
- Add reconciliation tests proving a real-domain site removes/stops the
  project Quick Tunnel and a no-domain site removes the project public Nginx
  router.
- Update Nginx tests to prove public router configuration requires a real
  domain while per-site Nginx remains available without one.
- Add SSL-flow tests proving the Quick Tunnel removal occurs only after the
  final HTTPS router render/reload.
- Add manifest tests proving the Quick Tunnel service is expected only for
  no-domain sites and SSL artifacts are tied only to a real domain.
- Add `bonesremote` unit tests for extracting the newest valid
  `https://*.trycloudflare.com` URL from representative journal output and for
  suppressing stale URLs when the service is inactive.
- Add local status/setup tests proving the preview URL is rendered for
  no-domain sites and setup no longer recommends `remote ssl` merely because
  SSL is disabled.
- Manually smoke-test a Quick Tunnel against a BonesDeploy per-site Nginx Unix
  socket and verify the assigned HTTPS URL serves the placeholder or deployed
  application.
- Manually restart the project cloudflared service and verify
  `bonesdeploy status` reports the newly assigned URL rather than the old one.
- Manually configure a real domain, run the SSL flow, verify HTTPS works on the
  real domain, and verify the project Quick Tunnel is removed afterward.
- Run focused Rust and Python tests, `ruff check .`, `ruff format .`,
  `uv run pytest` from `crates/bonesinfra/python`, `cargo fmt`,
  `cargo clippy`, and `shfmt -w .`.
- Do not run end-to-end tests.
- Review the final diff to confirm temporary hostname behavior is gone, no
  provider framework or persistent generated-hostname state
  was introduced, and the real-domain path remains Nginx/Certbot-owned.
