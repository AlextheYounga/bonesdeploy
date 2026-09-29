# Build Scripts

Scripts in this directory run inside a disposable local Docker container for
the committed revision.

## Environment

- Working directory: `/workspace/source`
- No access to the root `.env`, runtime secrets, `shared/`, `releases/`, the
  database, host services, Docker socket, host home, or credentials.
- Committed public `.env.build` values are explicit build inputs.

## Contract

- Scripts run in lexical order by filename.
- Non-zero exit code fails the deploy.
- Your job: produce the deployable app layout inside `/workspace/source`.
- BonesRemote verifies and receives the artifact, then promotes this output into
  a sealed release.

## Adding Scripts

Name them with a numbered prefix so the order is clear:

```text
01_install_deps.sh
02_build_assets.sh
```

No secrets or runtime state. `.env.build` is public build configuration only.

## Next.js output

For a server-rendered runtime, set `output: "standalone"`. The build script
checks for `.next/standalone/server.js` and copies `public/` and `.next/static/`
into that standalone output for the runtime service.

For a static runtime (`is_static = true`, `web_root = "out"`), set
`output: "export"`. The build script requires `out/index.html`, which the
static nginx runtime serves.
