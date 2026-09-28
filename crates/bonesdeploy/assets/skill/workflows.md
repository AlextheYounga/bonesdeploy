# BonesDeploy workflows

## First-time setup, the short path

```text
bonesdeploy init
bonesdeploy setup --yes
git push production main
bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

Root setup composes server setup and site setup. Site setup performs the
readiness check, site base, runtime, and doctor sequence. Use this convenience
path for the first project on a fresh host.

## First-time setup, the explicit path

```text
bonesdeploy init
bonesdeploy server setup --yes
bonesdeploy server doctor
bonesdeploy site setup --yes
git push production main
bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

Use the explicit path when diagnosing a host or provisioning multiple sites on
one server. Server setup is performed once per host; site setup is performed
once per project.

## Additional project on a prepared host

```text
bonesdeploy init
bonesdeploy site setup --yes
git push production main
bonesdeploy deploy
```

Do not repeat `bonesdeploy server setup` for each project and do not run it
concurrently from multiple projects. The existing host baseline is shared;
every project still requires its own site setup.

## The daily deploy

```text
git push production main
bonesdeploy deploy
```

For a Compose site, commit exactly one conventional base Compose file and at
most one conventional override. Compose performs image pull/build and waits for
declared health checks. `shared/.env` is available for interpolation but is not
injected unless the Compose file references it. Numbered BonesDeploy build and
prepare scripts are native-only.

## Secrets, end to end

```text
bonesdeploy secrets init
bonesdeploy secrets edit
bonesdeploy secrets push
bonesdeploy deploy
```

`.env.build` at the project root holds committed, non-secret build-time values
(e.g. `NEXT_PUBLIC_API_URL=https://api.example.com`). Runtime secrets come from
`shared/.env` via `bonesdeploy secrets push`. The explicit push atomically
replaces the complete remote environment; it does not merge any `.env` files.
`bones.toml` is committed; `shared/.env` is not. That's the contract.
## Recovery

Bad deploy:

```text
bonesdeploy rollback
```

Compose rollback reconciles the previous release's Compose definition under the
same `bonesdeploy-<site>` project. Named volumes remain in place, so rollback
does not reverse database migrations, volume contents, or external side effects.

Stuck build:

```text
bonesdeploy site releases
bonesdeploy site releases kill <stuck-release>
```

Wrong runtime:

```text
bonesdeploy site runtime --yes
```

Wrong SSL:

```text
bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com
```

## Inspecting state

```text
bonesdeploy skill next
bonesdeploy server doctor
bonesdeploy site doctor
bonesdeploy site status
bonesdeploy site releases
```
