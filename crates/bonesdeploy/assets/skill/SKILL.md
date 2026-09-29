# BonesDeploy: the skill

You're an AI agent. You're about to operate a deployment tool. Read this first.
Then run `bonesdeploy skill next` and let the tool tell you what to do.

BonesDeploy ships releases to Debian 12+ and Ubuntu 24.04+ `x86_64` servers.
Native sites use a dedicated runtime user, systemd, nginx, and local Docker
artifact builds. Compose sites use a project-owned stack through rootful Docker
Compose.

The beauty is in the constraints. There are exactly five moves that matter.
Everything else is recovery or inspection. Learn the moves and you can operate
any bonesdeploy project without reading a single line of YAML.

## The six moves

1. `bonesdeploy init` — claim a project, point it at a fresh VPS, pick a
   framework template. Non-interactive agents: pass `--template <name>` and
   `--framework-var key=value` (see `bonesdeploy skill doc templates` for
   every template and every variable).
2. `bonesdeploy server setup --yes` — provision the shared host baseline once
   per host, not once per project:
    packages, hardening, deploy identity, BonesRemote, and sudoers.
   It is independent of every site's framework and runtime settings.
3. `bonesdeploy site setup --yes` — once per project, verify server readiness,
   then provision that site in this exact order: site base, runtime, and doctor.
   It never pushes Git or secrets, configures SSL, or deploys a release.
4. `bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com`
   — TLS. Separate from `setup` because certificate concerns and runtime
   concerns are different concerns.
5. `bonesdeploy deploy` — build the configured local revision locally and ship the artifact release.

That's a deployment. In between, you repeat the deploy command. Nothing else
matters until it works.

`bonesdeploy setup --yes` remains an idempotent convenience command for a first
project on a fresh host. It runs server setup and site setup in sequence. For
every additional project on that host, run only `bonesdeploy site setup --yes`.
Do not run host-wide server setup concurrently from multiple projects. Run
`bonesdeploy site runtime --yes` separately when reapplying an existing site.

## What you actually own

A root `.env` holds local connection and site inputs. `infra/` holds the
committed project infrastructure, and `deployment/{build,prepare}/NN_*.sh`
holds the ordered build and prepare scripts.

Native projects use numbered shell scripts in lexical order. Docker Compose
projects own their Compose file, Dockerfiles, and supporting services.
Compose is reduced-guarantee mode: its file is trusted privileged input, and
BonesDeploy does not enforce native isolation for project-selected images,
mounts, capabilities, networks, users, or ports. It never adds a Docker socket
mount. Named volumes persist across rollback, but their data is not rolled back.

## How to read state

- `bonesdeploy skill next` — the next prompt-free command to run. This is your
  compass. It knows whether you're uninitialized, half-provisioned, missing
  TLS, or ready to ship. Ask it first. Ask it often.
- `bonesdeploy site status` — the live picture: current release, SSL, services.
- `bonesdeploy site tunnel status` — the optional Quick Tunnel state and current ephemeral URL.
- `bonesdeploy doctor` — server + site health. Exit code tells you everything.
- `bonesdeploy site releases` — what's on the box: `active`, `previous`, `building`,
  `preparing`, `interrupted`.

## How to recover

- `bonesdeploy rollback` — repoint `current` to the previous release. One command.
- `bonesdeploy site releases kill <release>` — cancel a stuck build.
- `bonesdeploy pull` — restore local `.bones/` from remote site state.

## How to push secrets

- `bonesdeploy secrets init` — bootstrap GPG-encrypted `.env` (also performed by `bonesdeploy init`).
- `bonesdeploy secrets edit` — decrypt, edit, re-encrypt.
- `bonesdeploy secrets push` — atomically replace remote `shared/.env` with the decrypted complete `.env`.

Never commit plaintext secrets. Never put secret values in `.env`. Use
`.env.build` for committed public build values; use `shared/.env` for runtime
secrets via `bonesdeploy secrets push`.

## What this tool will not do

- Will not widen permissions just because a later step might need it. Mutations
  happen at the last responsible moment. That's not preference; it's doctrine.
  Read `bonesdeploy skill doc methodology` before you "fix" a permissions
  problem by chmodding everything.
- Will not run native applications as a shared `www-data` user. Compose
  container users and isolation are controlled by the project.
- Will not run native application builds on production or fall back to a server
  build when a local Docker build fails.
- Will not deploy to hosts outside Debian 12+/Ubuntu 24.04+ on `x86_64`. Don't ask.

## Going deeper

- `bonesdeploy skill doc commands` — every command, every flag, every exit.
- `bonesdeploy skill doc templates` — every framework template and its `--framework-var` keys.
- `bonesdeploy skill doc workflows` — the end-to-end local-artifact flows.
- `bonesdeploy skill doc methodology` — permission model, just-in-time mutations, identity classes.
- `bonesdeploy skill list` — names of every embedded doc.

## For AI agents

You are operating a real system that ships real releases to real servers. Act
like it.

1. Run `bonesdeploy skill next` before suggesting commands. It's authoritative.
2. Run `bonesdeploy doctor` before deploying. Non-zero exit means stop.
3. Never invent flags. Every flag is in `bonesdeploy skill doc commands`. If it
   isn't there, it doesn't exist.
4. Never `chmod 777`, never `chown -R` on shared state, never edit files under
   `/root/.config/bonesremote/` on the host. Those are owned by `bonesremote`.
5. If a deploy fails, `bonesdeploy rollback` is the first answer, not a
   human-readable essay about what might have gone wrong.
6. The `--yes` flag skips *confirmation* prompts, not safety. Use it when you
   already ran `doctor` and `skill next` says you're clear.

Constraint is liberating. The five moves are the whole game.
