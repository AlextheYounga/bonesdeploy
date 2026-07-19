# BonesDeploy commands

Every command below is real. Every flag is real. If a flag isn't here, it isn't
real. Don't invent.

## init

`bonesdeploy init [--non-interactive] [--project-name <name>] [--branch <b>] [--remote <r>] [--host <h>] [--port <p>]`

Claims a project. Loads `.bones/bones.toml` if present; otherwise prompts.
`--non-interactive` is for CI and AI: every required field must come from a flag.
Creates the local `.bones` symlink, updates `.gitignore`, and adds the
deployment remote. Prints next steps. Does not provision anything.

## setup

`bonesdeploy setup [--yes]`

The full first-time remote provisioning, in order: `remote bootstrap` →
`remote runtime` → `push` → `doctor`. One command. `--yes` skips the
runtime confirmation. Use `remote bootstrap` + `remote runtime` separately
only when you want to control the steps or when you're changing the
framework template on an already-provisioned box. Idempotent — re-run it
after fixing whatever made it fail.

## doctor

`bonesdeploy doctor [--local]`

Local and remote health. Pass = silent, exit 0. Warnings or errors = exit
non-zero. A pending first git push is success, not failure — an empty bare
repo before the first push is expected. `--local` skips the SSH round trip; the
`pre-push` hook uses this.

## status

`bonesdeploy status`

Live picture: current release, SSL state, services. Calls `bonesremote status`
over SSH. No flags. Read it when something feels off.

## skill

`bonesdeploy skill` — print the orientation doc you're reading right now.
`bonesdeploy skill next [--format text|json]` — the compass. Suggests the next
prompt-free command based on actual state (uninitialized → init, initialized →
setup, setup complete → ssl, ready → deploy). `--format json` is
machine-readable. This is the command AI agents should call before doing
anything.
`bonesdeploy skill list` — names of every embedded skill doc.
`bonesdeploy skill doc <name>` — print a specific skill doc by name.

## push

`bonesdeploy push`

Archives `.bones/` (secrets excluded) and streams it to `bonesremote site
import`. Atomically replaces remote site state. Does not deploy. Does not
push git refs.

## pull

`bonesdeploy pull`

Streams the current remote site dataset back into local `.bones/` and
reinstalls the `pre-push` guard. Recovery primitive.

## deploy

`bonesdeploy deploy`

`push` then SSH `bonesremote deploy --site <project>`. The whole pipeline:
stage → checkout → build → promote → wire → prepare → seal → activate →
restart → prune. On failure before activation, drops the staged release. On
failure after activation, restores and restarts the previous release.

## releases

`bonesdeploy releases`
`bonesdeploy releases kill <release>`

Lists releases and their state: `active`, `previous`, `building`,
`preparing`, `interrupted`. `kill` cancels a building or interrupted release
and cleans its build container, temp context, and staged state.

## rollback

`bonesdeploy rollback`

Repoints `current` to the previous release and restarts `<project>.target`.
No rebuild. The first answer to a bad deploy.

## secrets

`bonesdeploy secrets init`
`bonesdeploy secrets edit`
`bonesdeploy secrets push`

GPG-encrypted `.env` under `.bones/secrets/`. `init` bootstraps. `edit`
decrypts, opens `$EDITOR`, re-encrypts on save. `push` ships the decrypted
`.env` to remote `shared/.env` over SSH. Build scripts see these via
`[build].vars` in `bones.toml`.

## remote bootstrap

`bonesdeploy remote bootstrap` (alias: `remote setup`)

Provisions the host: users, groups, firewall, system packages, bare repo,
placeholder release, sudoers drop-in, and `bonesremote` itself. Delegates to
the hidden `bonesinfra` checkout. Runs as root (or
`BONES_BOOTSTRAP_SSH_USER`).

## remote runtime

`bonesdeploy remote runtime [--yes]`

Installs the framework runtime: AppArmor profile, nginx router + per-site
config, systemd service. Prompts for a template (laravel, django, next, nuxt,
sveltekit, vue, rails) when not set. Writes the selection into `bones.toml`.
Does not do TLS — that's `remote ssl`. Already included in `bonesdeploy
setup`. Run it on its own only when changing templates on a provisioned box.

## remote ssl

`bonesdeploy remote ssl [--yes] [--domain <d>] [--email <e>]`

certbot webroot challenge for the configured domain. Re-renders the nginx
router with TLS, listens on 443, redirects HTTP → HTTPS. Decoupled from
`remote runtime` because certificate concerns and runtime concerns are
different concerns.

## remote helpers

`bonesdeploy remote helpers [--yes]`

Installs convenience tools on the host (starship, neovim, aptui). Optional.
Not part of any deploy flow.

## config

`bonesdeploy config [--file <path>] [key]`

Reads `.bones/bones.toml`. No key → dump the whole file. `--file` overrides
the path. Read-only. There is no `config set`. Edit the file by hand.

## update

`bonesdeploy update [--skip-local] [--skip-remote]`

Updates `bonesdeploy` and `bonesremote` to the latest version. Skip either
side with the obvious flag.

## version

`bonesdeploy version`

Prints the installed version. That's it.
