# BonesDeploy: the skill

You're an AI agent. You're about to operate a deployment tool. Read this first.
Then run `bonesdeploy skill next` and let the tool tell you what to do.

BonesDeploy ships releases to plain Debian/Ubuntu servers. Not Kubernetes. Not
ECS. Not Nomad. A real Linux box, a dedicated runtime user per project, systemd,
nginx, and a rootless Podman build container. That's the whole stage. Everything
else is a recovery or inspection move.

The beauty is in the constraints. There are exactly six moves that matter.
Everything else is recovery or inspection. Learn the moves and you can operate
any bonesdeploy project without reading a single line of YAML.

## The six moves

1. `bonesdeploy init` — claim a project, point it at a fresh VPS, pick a
   framework template. Non-interactive agents: pass `--template <name>` and
   `--framework-var key=value` (see `bonesdeploy skill doc templates` for
   every template and every variable).
2. `bonesdeploy server setup --yes` — provision the shared host baseline once:
   packages, hardening, image store, deploy identity, BonesRemote, and sudoers.
   It is independent of every site's framework and runtime settings.
3. `bonesdeploy site setup --yes` — verify server readiness, then provision one
   site in this exact order: site base, services, runtime, and doctor. It never
   pushes Git or secrets, configures SSL, or deploys a release.
4. `git push <remote> <branch>` — publish the source so `bonesremote` has
   something to build. Required once, before the first deploy.
5. `bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com`
   — TLS. Separate from `setup` because certificate concerns and runtime
   concerns are different concerns.
6. `bonesdeploy deploy` — ship the release.

That's a deployment. In between, you repeat move five. Nothing else matters
until move five works.

`bonesdeploy setup --yes` remains an idempotent convenience command that runs
server setup and site setup in sequence. On an already prepared shared host,
start directly with site setup. Run
`bonesdeploy site runtime --yes` separately when reapplying an existing site.

## What you actually own

A root `.env` holds local connection and site inputs. `infra/` holds the
committed project infrastructure, and `deployment/{build,prepare}/NN_*.sh`
holds the ordered build and prepare scripts.

That's it. You don't write Kubernetes YAML. You don't write Dockerfiles. You
write shell scripts, numbered, in lexical order. The constraint is the feature.

## How to read state

- `bonesdeploy skill next` — the next prompt-free command to run. This is your
  compass. It knows whether you're uninitialized, half-provisioned, missing
  TLS, or ready to ship. Ask it first. Ask it often.
- `bonesdeploy site status` — the live picture: current release, SSL, services.
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
- `bonesdeploy secrets push` — ship the decrypted `.env` to remote `shared/.env`.

Never commit plaintext secrets. Never put secret values in `.env`. Use
`.env.build` for committed public build values; use `shared/.env` for runtime
secrets via `bonesdeploy secrets push`.

## What this tool will not do

- Will not widen permissions just because a later step might need it. Mutations
  happen at the last responsible moment. That's not preference; it's doctrine.
  Read `bonesdeploy skill doc methodology` before you "fix" a permissions
  problem by chmodding everything.
- Will not run as a shared `www-data` user. Each project gets its own runtime
  user. Isolation is at the OS level, not just the application level.
- Will not silently reset Podman. A broken build namespace is reported, not
  papered over.
- Will not deploy to non-Debian/Ubuntu hosts. Don't ask.

## Going deeper

- `bonesdeploy skill doc commands` — every command, every flag, every exit.
- `bonesdeploy skill doc templates` — every framework template and its `--framework-var` keys.
- `bonesdeploy skill doc workflows` — the end-to-end flows, including git-triggered deploy.
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
