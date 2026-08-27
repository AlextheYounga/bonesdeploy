# Idea

## Request

Provide a Cloudflare Quick Tunnel for sites that do not yet have a real domain
configured.

The temporary URL must require no user-owned domain and no Cloudflare account.
It should be created and managed as part of the site's existing runtime
lifecycle and exposed to the user through normal BonesDeploy status/setup
output.

## Problem

Temporary preview access must not be represented as a configured hostname. It
is runtime ingress and should not pass through project configuration or the
public Nginx router.

The preview hostname is deterministic configuration even though its purpose is
temporary access before a real domain exists. It also requires the system Nginx
router to expose the site publicly on the server's inbound HTTP path.

Cloudflare Quick Tunnels provide the intended temporary-access behavior
directly: `cloudflared` creates a random `*.trycloudflare.com` HTTPS URL without
a Cloudflare account or user-owned domain and proxies that URL to a local
origin. BonesDeploy's per-site Nginx already exposes each site through a Unix
socket, so a Quick Tunnel can reach the site without creating a temporary DNS
name or opening a new application port.

## Definitions

**Real domain:** A non-empty domain explicitly configured for the site in
`app.dns.domain`. A real domain is owned and managed by the user and continues
to use BonesDeploy's existing public Nginx and Certbot flow.

**Temporary preview:** Public, short-lived access to a site before a real domain
is configured. A temporary preview is a convenience for setup, development,
review, and testing. It is not production ingress and does not promise a stable
hostname.

**Quick Tunnel:** An account-less Cloudflare Tunnel started by `cloudflared`
that receives a random `*.trycloudflare.com` hostname and exists for the
lifetime of that `cloudflared` process. Restarting the process can produce a new
hostname.

**Preview URL:** The current HTTPS `*.trycloudflare.com` URL assigned to the
site's running Quick Tunnel. The preview URL is runtime state, not project
configuration.

**Public router:** The root-managed system Nginx virtual host that accepts
inbound traffic for a configured real domain and proxies it to the site's
per-site Nginx Unix socket.

**Per-site Nginx:** The existing project-owned Nginx process that serves or
proxies the application through the project's Unix socket under `/run`.

## Desired outcome

A newly configured BonesDeploy site with no real domain can be set up and
reached through an automatically managed HTTPS `*.trycloudflare.com` URL
without DNS changes, a Cloudflare account, or a publicly exposed application
port.

The Quick Tunnel runs as a project-scoped systemd service and proxies directly
to the site's existing per-site Nginx Unix socket. `bonesdeploy status` reports
the current preview URL, and setup output surfaces the URL when it is available.

When a real domain is configured, BonesDeploy uses the existing public
Nginx/Certbot ingress path instead of the Quick Tunnel. Temporary ingress is
runtime state and is not written to project configuration.

## Scope

- Remove configured temporary-hostname handling from all configuration layers.
- Install `cloudflared` from Cloudflare's supported Debian/Ubuntu package
  repository when a site needs temporary preview ingress.
- Run one project-scoped Quick Tunnel service for a site that has no real
  domain.
- Proxy the Quick Tunnel directly to the site's existing per-site Nginx Unix
  socket.
- Reconcile public Nginx routing so a no-domain site does not require a
  temporary public virtual host.
- Stop and remove the project Quick Tunnel from the site lifecycle when a real
  domain takes over ingress.
- Report the current Quick Tunnel URL and service state through
  `bonesremote status` and `bonesdeploy status`.
- Surface the current preview URL during setup when the tunnel has started.
- Update manifests, focused tests, and user/project documentation for the new
  ingress behavior.

## Constraints

- A temporary preview must not require a user-owned domain, DNS record,
  Cloudflare account, API token, or Cloudflare zone.
- The Quick Tunnel must use the existing per-site Nginx Unix socket as its
  origin rather than introducing a project TCP port solely for preview access.
- `cloudflared` must run as a project-scoped service under the site's existing
  systemd lifecycle rather than as an unmanaged background process or a single
  global tunnel service.
- The generated `trycloudflare.com` hostname must be treated as runtime state
  and must not be persisted as authoritative project configuration.
- Real-domain ingress, Certbot certificate issuance, release deployment,
  rollback, and application runtime behavior must continue to use their
  existing ownership boundaries.
- Transitioning to a real HTTPS domain must activate the real-domain route
  before the temporary tunnel is removed.
- Quick Tunnels are development/testing infrastructure: they have no uptime
  SLA, currently limit a tunnel to 200 concurrent in-flight requests, and do
  not support Server-Sent Events.
- Do not run the repository end-to-end test suite.

## Exclusions

- Parallel preview environments for individual releases, branches, pull
  requests, or commits.
- Creating multiple simultaneous Quick Tunnels for one site.
- Managed or named Cloudflare Tunnels.
- Cloudflare accounts, API tokens, Access policies, DNS APIs, custom Cloudflare
  hostnames, or Cloudflare-managed production ingress.
- Stable temporary hostnames across `cloudflared` restarts.
- Replacing Nginx as the real-domain reverse proxy.
- Replacing Certbot or changing real-domain certificate management.
- Adding SSE support or attempting to work around Cloudflare Quick Tunnel
  service limits.
- Generalizing temporary ingress into a provider/plugin framework.
