# Plan

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
