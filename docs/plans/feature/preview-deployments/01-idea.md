# Idea

## Request

Add commit-specific preview deployments with a Vercel-style command surface:

```text
bonesdeploy preview setup
bonesdeploy preview deploy <revision> --alias pr-42
bonesdeploy preview list
bonesdeploy preview status <preview>
bonesdeploy preview logs <preview>
bonesdeploy preview remove <preview>
bonesdeploy preview prune
```

A committed Git revision must be able to run concurrently with production at an
immutable commit hostname. A validated branch or pull-request alias supplied to
a deploy command moves to that command's ready preview; the newest successful
serialized command for an alias wins.

The feature uses wildcard DNS, a wildcard certificate obtained through an
interactive manual Certbot DNS-01 challenge, and the existing root-managed
system Nginx. It is separate from the accountless Cloudflare Quick Tunnel
feature.

## Problem

BonesDeploy currently has one active release per site. Production services and
per-site Nginx resolve the `current` symlink, production runtime secrets are
linked from `shared/.env`, service units are site-scoped, and Docker Compose has
one stable project identity. Activating a second revision through those paths
would replace or share production runtime state rather than run concurrently.

The existing Quick Tunnel does not solve this problem. It gives the current site
a temporary process-owned URL; it does not build another revision, isolate a
runtime, provide stable hostnames, retain preview state, or route aliases.

## Definitions

**Preview deployment:** One sealed release built from one resolved commit and
published without changing the production `current` symlink. A preview has its
own runtime root, service instances, local Nginx origin, environment file, state
record, commit hostname, and expiration time.

**Preview identifier:** A lowercase hexadecimal prefix of the resolved commit.
It starts at 12 characters and is lengthened when necessary to remain unique
among retained previews. Raw revision text is never used in paths, unit names,
or Nginx configuration.

**Commit hostname:** The immutable hostname formed from the preview identifier
and configured base domain, such as `a1b2c3d4e5f6.preview.example.com`. It points
to the same preview until that preview is removed or pruned.

**Alias:** A validated lowercase DNS label, such as `pr-42`, that resolves under
the preview base domain and may be atomically moved from one ready preview to
another. An alias is routing state, not a second deployment.

**Preview base domain:** The user-owned DNS suffix under which preview commit
hostnames and aliases are published, such as `preview.example.com`.

**Preview gateway:** A project-scoped, root-owned system Nginx virtual host for
`*.preview.example.com`. It terminates wildcard TLS, returns a non-serving
default response for unknown names, and proxies only validated exact preview
hostnames to release-specific local Nginx sockets.

**Quick Tunnel:** The existing accountless `cloudflared` process that exposes
the site's current runtime through a temporary `trycloudflare.com` URL. It is
not a preview deployment, preview gateway, or alias provider.

**Preview environment:** An empty or explicitly user-supplied environment file
stored only inside one preview runtime root. It never falls back to production
`shared/.env` or production `runtime.env`.

## Desired outcome

After one convergent `preview setup`, a trusted committed revision can be built
with the existing release preparation lifecycle and run beside production. Its
commit URL remains stable while the preview exists, and an optional alias can
move to a newer ready preview without exposing a partial Nginx configuration.

List and status output identify the revision, release, lifecycle phase,
hostnames, timestamps, expiration, certificate and DNS health, route state, and
observed systemd/socket state. Logs expose build, prepare, and runtime evidence.
Remove and prune stop preview services, withdraw routes, and delete preview-only
resources without changing production.

Setup explains and verifies the permanent wildcard DNS record, obtains or
reuses the wildcard certificate, and provisions the gateway without deploying a
preview. Repeating setup reconciles missing or stale managed infrastructure and
does not request a certificate that remains usable.

## Scope

- Add the top-level `preview` command group and the requested setup, deploy,
  list, status, logs, remove, and prune operations.
- Persist preview base-domain and retention configuration in the canonical
  project configuration.
- Extend the existing per-site `SiteState` with authoritative preview records.
- Reuse source resolution, export, build, promotion, prepare, sealing, logging,
  ownership, and site mutation boundaries from the production lifecycle.
- Provision release-scoped native runtime and Nginx service templates with the
  same site identity and hardening patterns as production services.
- Keep preview environment files and mutable framework paths under each preview
  root and outside production `shared/`.
- Provision a separate wildcard system-Nginx gateway and atomically reconcile
  exact commit and alias routes.
- Guide an interactive Certbot manual DNS-01 challenge and report certificate
  renewal warnings.
- Apply age-based retention and manual pruning without scheduling background
  cleanup.
- Add focused Rust and Python tests plus documented manual server validation.
- Rename Quick Tunnel status terminology where it currently uses the generic
  word `preview`, so the two concepts remain distinct.

## Constraints

- Preview setup and every mutation are explicit commands. No Git hook, webhook,
  or push event deploys code.
- Only revisions resolvable as commits in the site's existing bare repository
  are deployable.
- Preview mutations use `SiteMutation` and the existing per-site deployment
  lock. Builds and mutations are serialized in v1.
- Production `current`, production services, production routes, and production
  shared data are never changed by a preview operation.
- Commit identifiers, aliases, hostnames, paths, and systemd instance names are
  validated before privileged use.
- The permanent DNS requirement is one wildcard A record, with AAAA or CNAME
  guidance when appropriate. Creating a preview requires no per-preview DNS
  change.
- Wildcard certificate issuance uses Certbot `--manual` DNS-01. BonesDeploy does
  not create or delete provider DNS records and does not persist challenge
  tokens.
- Setup must state that a rerun may produce a replacement TXT token and that
  obsolete `_acme-challenge` values must be replaced.
- Nginx changes use root-owned atomic writes, `nginx -t`, rollback on failure,
  and reload only after validation. Setup or route failure must leave production
  routing on the previous known-good configuration.
- A certificate with more than 30 days remaining is reusable. Setup starts the
  manual renewal flow at 30 days or fewer; status warns at the same threshold.
- Previews receive no production secrets or database credentials by default.
  Supplying preview-safe values is an explicit operator action.
- Preview URLs are public and unauthenticated in v1. Setup requires confirmation
  of that fact.
- Preview execution is not a sandbox for untrusted fork code. The operator must
  trust the revision; automated fork-PR deployment is not supported.
- Do not run the end-to-end test suite as part of this change.

## Exclusions

- Cloudflare named tunnels, Cloudflare account credentials, DNS APIs, DNS
  provider abstractions, ngrok, or any other ingress provider.
- Automatic unattended certificate renewal or automated TXT record changes.
- HTTP authentication, SSO, IP allowlists, or a generalized access-policy
  framework for preview URLs.
- GitHub webhooks, hosted control planes, provider-specific pull-request bots,
  or deploy-on-push behavior.
- Automatic database cloning, production database access, or production secret
  inheritance.
- Deployment of untrusted fork revisions.
- Parallel preview builds or concurrent site mutations.
- Docker Compose previews in v1. Compose currently shares the production
  project identity, `current`, `shared/.env`, networks, and persistent volumes;
  safely separating those is a distinct feature.
- A scheduler for preview pruning. `preview prune` is command-driven and may be
  invoked by user-managed CI or cron.
- Serving the preview base-domain apex. The wildcard certificate and gateway
  cover one-label subdomains only.
- Host-level disk quotas and hostile-code isolation between previews belonging
  to the same site. Retention bounds disk lifetime, but operators remain
  responsible for host capacity.
- Replacing the production Nginx/Certbot flow or changing Quick Tunnel behavior
  beyond correcting its status terminology.
