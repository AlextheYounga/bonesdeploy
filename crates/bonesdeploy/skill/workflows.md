# BonesDeploy workflows

## First-time setup, the short path

```
bonesdeploy init
bonesdeploy setup --yes
bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
git push production master
bonesdeploy deploy
```

`setup --yes` does bootstrap + runtime + initial push. After that, you push
your source to the bare repo once (so `bonesremote` has something to build),
then `deploy`. That's the whole dance.

## First-time setup, the explicit path

```
bonesdeploy init
bonesdeploy remote bootstrap
bonesdeploy remote runtime --yes
bonesdeploy push
git push production master
bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
bonesdeploy deploy
```

Use this when you want to see each step fail independently. You will, the
first time. There's no shame in being explicit while you're learning the
shape of the thing.

## The daily deploy

```
bonesdeploy deploy
```

That's it. It pushes `.bones/` and runs the remote pipeline. If you have
`deploy_on_push = true` in `bones.toml`, `git push production <branch>` does
the same thing via the `post-receive` hook. Pick one. Don't use both.

## Git-triggered deploy

Set `deploy_on_push = true` in `bones.toml`. `bonesdeploy init` installs a
`pre-push` guard locally; `bonesdeploy push` installs the `post-receive`
trigger on the bare repo. Then:

```
git push production master
```

The `pre-push` guard runs `bonesdeploy doctor --local` and aborts on
warnings or errors. Git updates refs. `post-receive` runs `bonesremote
deploy --site <project> --revision <newrev>`. Done.

The hook is optional plumbing, not the primary model. `bonesdeploy deploy`
is the primary model. Don't let anyone tell you git hooks are the
deployment. They're a convenience on top of it.

## Secrets, end to end

```
bonesdeploy secrets init
bonesdeploy secrets edit     # add NEXT_PUBLIC_API_URL=... etc.
bonesdeploy secrets push
bonesdeploy deploy
```

`[build].vars = ["NEXT_PUBLIC_API_URL"]` in `bones.toml` tells `bonesremote`
to inject that env var into the build container. The value comes from
`shared/.env` on the host, not from `bones.toml`. `bones.toml` is committed;
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
