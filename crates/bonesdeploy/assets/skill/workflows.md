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
