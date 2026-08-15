# BonesDeploy workflows

## First-time setup, the short path

```
bonesdeploy init
bonesdeploy setup --yes
git push production master
bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

`setup --yes` does bootstrap + runtime + doctor in one shot. You do
not need `remote runtime` separately — it's already inside `setup`. After
setup, push your source to the bare repo once (so `bonesremote` has
something to build), then `deploy`. That's the whole dance.

## First-time setup, the explicit path

Use this only when you want each step to fail independently — the first
time, you probably do, because you'll want to see where it breaks.

```
bonesdeploy init
bonesdeploy remote bootstrap
bonesdeploy remote runtime --yes
git push production master
bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

`setup --yes` is just these first four steps in sequence. There's no shame in
being explicit while you're learning the shape of the thing.

## The daily deploy

```
bonesdeploy deploy
```

The application source is pushed separately with `git push production master`,
then `bonesdeploy deploy` runs the explicit remote pipeline.

## Secrets, end to end

```
bonesdeploy secrets init     # already performed by bonesdeploy init
bonesdeploy secrets edit     # add NEXT_PUBLIC_API_URL=... etc.
bonesdeploy secrets push
bonesdeploy deploy
```

`.env.build` at the project root holds committed, non-secret build-time values
(e.g. `NEXT_PUBLIC_API_URL=https://api.example.com`). Runtime secrets come from
`shared/.env` via `bonesdeploy secrets push`. `bones.toml` is committed;
`shared/.env` is not. That's the contract.

## Recovery

Bad deploy:

```
bonesdeploy rollback
```

Stuck build:

```
bonesdeploy releases
bonesdeploy releases kill <stuck-release>
```

Lost local `.bones/`:

```
bonesdeploy pull
```

Wrong runtime on the host:

```
bonesdeploy remote runtime --yes
```

Wrong SSL:

```
bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
```

## Inspecting state

```
bonesdeploy skill next      # what to run next
bonesdeploy status          # live state
bonesdeploy doctor          # health
bonesdeploy releases        # release history
bonesdeploy config          # dump bones.toml
```

Run them in that order when you don't know what's going on. By the fourth,
you know.
