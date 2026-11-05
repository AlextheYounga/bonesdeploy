# Idea

## Request

Make Cloudflare Quick Tunnels a separate, explicitly enabled site feature.
Normal site setup and runtime startup must not install, configure, or start a
Quick Tunnel.

## Problem

Domainless sites currently receive Cloudflared automatically. The generated
service passes `unix:/run/<site>/nginx/nginx.sock` to `cloudflared --url`.
Cloudflared 2026.9.3 treats `unix` as an HTTP hostname, fails DNS resolution,
and returns HTTP 502 even though the site nginx socket itself serves HTTP 200.

Accountless Quick Tunnel mode requires `--url`, while `--unix-socket` is
exclusive with `--url`, so a direct Unix-socket Quick Tunnel is not a supported
configuration in this version.

## Definitions

**Quick Tunnel:** An explicitly started, accountless Cloudflare Tunnel that
receives an ephemeral `*.trycloudflare.com` URL and serves a site through
BonesDeploy-managed nginx.

**Loopback origin:** A project-specific HTTP listener bound only to
`127.0.0.1`. Root nginx accepts the Cloudflared request there and proxies it to
the existing per-site nginx Unix socket. It does not expose the application on
a public interface.

**Tunnel cleanup:** Stopping and disabling the project Cloudflared service,
removing its unit and loopback nginx route, reloading systemd and nginx, and
leaving the server-wide Cloudflared package installed for other sites.

## Desired Outcome

Normal setup and runtime operations provision and start the site without any
Cloudflare package, unit, route, or process. A user can explicitly start a
Quick Tunnel, inspect its current ephemeral URL, and stop it. Cloudflared uses
a supported loopback HTTP URL while nginx remains the site-facing origin.

## Scope

- Add discoverable `bonesdeploy site tunnel start`, `stop`, and `status`
  operations.
- Provision the Cloudflare package, loopback nginx route, and project service
  only during explicit tunnel startup.
- Keep Cloudflared outside the normal site systemd target.
- Preserve current journal-based discovery of the ephemeral URL.
- Remove obsolete automatic runtime, setup, SSL, manifest, and release-restart
  behavior.
- Clean up project tunnel resources explicitly on tunnel stop.
- Update focused tests and user/project documentation.

## Constraints

- Use Cloudflared's supported accountless form,
  `cloudflared tunnel --url http://127.0.0.1:<port>`.
- Do not expose application processes directly or bind the tunnel origin to a
  public interface.
- Keep the per-site nginx Unix socket as the application ingress boundary.
- Run Cloudflared as the site's runtime user with no capabilities and the
  existing systemd hardening.
- Treat the generated URL as ephemeral observed runtime state, never project
  configuration.
- Do not add Cloudflare accounts, tokens, named tunnels, or persistent URLs.
- Do not run end-to-end tests.

## Exclusions

- Production Cloudflare Tunnel management.
- Stable preview hostnames.
- Multiple concurrent Quick Tunnels for one site.
- Preview environments per release, branch, or pull request.
- Direct Cloudflared access to application processes.
- A general ingress provider abstraction.
