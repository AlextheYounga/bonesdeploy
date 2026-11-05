# Tasks

The automatic Quick Tunnel implementation recorded by this plan was completed
but failed manual validation because its Unix-socket `--url` is unsupported.

Correction work is tracked in
[`../cloudflare-quick-tunnel/03-tasks.md`](../cloudflare-quick-tunnel/03-tasks.md).
That work removes automatic lifecycle coupling, adds an explicit tunnel command,
and changes the origin to supported loopback HTTP through nginx.
