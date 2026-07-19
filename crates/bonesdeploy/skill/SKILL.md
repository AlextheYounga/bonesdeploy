# BonesDeploy: the skill

You're an AI agent. You're about to operate a deployment tool. Read this first.
Then run `bonesdeploy skill next` and let the tool tell you what to do.

BonesDeploy ships releases to plain Debian/Ubuntu servers. Not Kubernetes. Not
ECS. Not Nomad. A real Linux box, a dedicated runtime user per project, systemd,
nginx, and a rootless Podman build container. That's the whole stage. Everything
else is a recovery or inspection move.

The beauty is in the constraints. There are exactly five moves that matter.
Everything else is recovery or inspection. Learn the moves and you can operate
any bonesdeploy project without reading a single line of YAML.

## The five moves

1. `bonesdeploy init` — claim a project, point it at a fresh VPS.
2. `bonesdeploy setup --yes` — provision the server in one shot: bootstrap
   (users, bare repo, `bonesremote`), runtime (nginx, app service, AppArmor),
   `.bones/` sync, and `doctor`. One command. Don't split it unless something
   fails. If it does, re-run `setup` after you fix the cause — it's idempotent.
3. `git push <remote> <branch>` — publish the source so `bonesremote` has
   something to build. Required once, before the first deploy.
4. `bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com`
   — TLS. Separate from `setup` because certificate concerns and runtime
   concerns are different concerns.
5. `bonesdeploy deploy` — ship the release.

That's a deployment. In between, you repeat move five. Nothing else matters
until move five works.

Note: `bonesdeploy setup` already runs `remote runtime` for you. You only run
`bonesdeploy remote runtime --yes` on its own when you're changing the
framework template on an already-provisioned box.

## What you actually own

A directory called `.bones/` in your repo. It's a symlink to
`~/.config/bonesdeploy/<project>.bones/`. Inside:

- `bones.toml` — the project's configuration. Edit it by hand. It's the source of truth.
- `deployment/build/NN_*.sh` — runs in a container during build.
- `deployment/prepare/NN_*.sh` — runs as the runtime user before activation.

That's it. You don't write Kubernetes YAML. You don't write Dockerfiles. You
write shell scripts, numbered, in lexical order. The constraint is the feature.

## How to read state

- `bonesdeploy skill next` — the next prompt-free command to run. This is your
  compass. It knows whether you're uninitialized, half-provisioned, missing
  TLS, or ready to ship. Ask it first. Ask it often.
- `bonesdeploy status` — the live picture: current release, SSL, services.
- `bonesdeploy doctor` — local + remote health. Exit code tells you everything.
- `bonesdeploy releases` — what's on the box: `active`, `previous`, `building`,
  `preparing`, `interrupted`.

## How to recover

- `bonesdeploy rollback` — repoint `current` to the previous release. One command.
- `bonesdeploy releases kill <release>` — cancel a stuck build.
- `bonesdeploy pull` — restore local `.bones/` from remote site state.

## How to push secrets

- `bonesdeploy secrets init` — bootstrap GPG-encrypted `.env`.
- `bonesdeploy secrets edit` — decrypt, edit, re-encrypt.
- `bonesdeploy secrets push` — ship the decrypted `.env` to remote `shared/.env`.

Never commit plaintext secrets. Never put secret values in `bones.toml`. The
`[build].vars` list names env vars pulled from `shared/.env` — names only, not
values.

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
