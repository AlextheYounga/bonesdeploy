# BonesDeploy commands

Every command below is real. Every flag is real. If a flag is not here, it is
not real. Do not invent.

## init

`bonesdeploy init [--non-interactive] [--project-name <name>] [--branch <b>] [--remote <r>] [--host <h>] [--port <p>] [--template <t>] [--framework-var <key=value>]...`

Initializes one project and writes the canonical `.env`. It does not provision
the server or site.

## setup

`bonesdeploy setup [--yes]`

Composes `server setup --yes` followed by `site setup --yes`. Site setup first
checks server readiness, then runs site base provisioning, services, runtime,
and doctor. Both setup paths are idempotent.

## doctor

`bonesdeploy doctor [--verbose]`

Runs both `server doctor` and `site doctor`, reports both results, and fails if
either check fails. Use scoped doctor commands for focused checks.

## server

`bonesdeploy server setup [--yes]`
`bonesdeploy server doctor [--verbose]`
`bonesdeploy server helpers [--yes]`

Server commands establish and inspect the reusable host baseline: packages,
hardening, firewall, shared image store, global deploy identity, BonesRemote,
global roots, and sudoers. They do not read site runtime or framework state.

## site

`bonesdeploy site setup [--yes]`
`bonesdeploy site doctor [--local] [--verbose]`
`bonesdeploy site status`
`bonesdeploy site manifest [--format text|json]`
`bonesdeploy site releases [kill <release>]`
`bonesdeploy site runtime [--yes]`
`bonesdeploy site services [--yes]`
`bonesdeploy site ssl [--yes] [--domain <d>] [--email <e>]`

Site commands operate on one project. `site setup` does not push Git or
secrets, configure SSL, or deploy a release.

## skill

`bonesdeploy skill` prints the orientation document.
`bonesdeploy skill next [--format text|json]` suggests the next prompt-free
command based on `uninitialized`, `server_missing`, `site_missing`,
`ssl_missing`, and `ready` states.
`bonesdeploy skill list` lists embedded documents.
`bonesdeploy skill doc <name>` prints one embedded document.

## deploy

`bonesdeploy deploy`

Runs the BonesRemote release pipeline: stage, checkout, build, promote, wire,
prepare, seal, activate, restart, and prune.

## rollback

`bonesdeploy rollback`

Repoints `current` to the previous release and restarts the site target.

## secrets

`bonesdeploy secrets init`
`bonesdeploy secrets edit`
`bonesdeploy secrets push`

Manages encrypted local secrets and pushes them to remote `shared/.env`.

## update

`bonesdeploy update [--skip-local] [--skip-remote]`

Updates BonesDeploy, BonesRemote, and project infrastructure.

## version

`bonesdeploy version`

Prints the installed version.
