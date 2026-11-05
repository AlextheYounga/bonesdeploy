# Cloudflare Tunnel

<!-- Merged from feature/cloudflare-tunnel -->

## Idea

The original request made accountless Cloudflare ingress automatic for every
site without a real domain. Manual validation showed that its direct Unix-socket
`--url` was unsupported and returned HTTP 502.

That design is superseded by the explicit, opt-in lifecycle defined in
[`../cloudflare-quick-tunnel/01-idea.md`](../cloudflare-quick-tunnel/01-idea.md).
Normal site setup and runtime startup no longer own a Quick Tunnel.


## Plan

The original automatic no-domain Quick Tunnel design is superseded by
[`../cloudflare-quick-tunnel/02-plan.md`](../cloudflare-quick-tunnel/02-plan.md).

Manual validation found that Cloudflared 2026.9.3 interprets
`--url unix:/run/<site>/nginx/nginx.sock` as an HTTP origin with hostname
`unix`, causing DNS failures and HTTP 502 responses. Accountless Quick Tunnel
mode requires `--url`, which cannot be combined with `--unix-socket`.

The corrected design makes Quick Tunnels opt-in through
`bonesdeploy site tunnel start|stop|status`, removes Cloudflared from normal
site/runtime/SSL lifecycle management, keeps it outside the site target, and
uses a loopback-only nginx HTTP origin that proxies to the existing per-site
nginx Unix socket.


## Tasks

The automatic Quick Tunnel implementation recorded by this plan was completed
but failed manual validation because its Unix-socket `--url` is unsupported.

Correction work is tracked in
[`../cloudflare-quick-tunnel/03-tasks.md`](../cloudflare-quick-tunnel/03-tasks.md).
That work removes automatic lifecycle coupling, adds an explicit tunnel command,
and changes the origin to supported loopback HTTP through nginx.
