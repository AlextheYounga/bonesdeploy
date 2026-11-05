# Idea

The original request made accountless Cloudflare ingress automatic for every
site without a real domain. Manual validation showed that its direct Unix-socket
`--url` was unsupported and returned HTTP 502.

That design is superseded by the explicit, opt-in lifecycle defined in
[`../cloudflare-quick-tunnel/01-idea.md`](../cloudflare-quick-tunnel/01-idea.md).
Normal site setup and runtime startup no longer own a Quick Tunnel.
