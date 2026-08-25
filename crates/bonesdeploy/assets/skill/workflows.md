# BonesDeploy workflows

## First-time setup, the short path

```text
bonesdeploy init
bonesdeploy setup --yes
git push production master
bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

Root setup composes server setup and site setup. Site setup performs the
readiness check, site base, services, runtime, and doctor sequence.

## First-time setup, the explicit path

```text
bonesdeploy init
bonesdeploy server setup --yes
bonesdeploy server doctor
bonesdeploy site setup --yes
git push production master
bonesdeploy site ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

Use the explicit path when diagnosing a host or provisioning multiple sites on
one server. Server setup is performed once per host; site setup is performed
once per project.

## The daily deploy

```text
git push production master
bonesdeploy deploy
```

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

Stuck build:

```text
bonesdeploy site releases
bonesdeploy site releases kill <stuck-release>
```

Wrong runtime or services:

```text
bonesdeploy site runtime --yes
bonesdeploy site services --yes
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
