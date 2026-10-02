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
- Resolve `requirements.txt` here as a local compatibility check, then remove the
  disposable dependency tree. Production prepare creates the release virtualenv
  and installs requirements with the host's distribution Python.
- Remove build-only caches, source-only files, and numbered build scripts when
  the selected runtime does not need them.
- BonesRemote verifies and receives the artifact, then promotes this output into
  a sealed release.

## Adding Scripts

Name them with a numbered prefix so the order is clear:

```text
01_install_deps.sh
02_build_assets.sh
```

No secrets or runtime state. `.env.build` is public build configuration only.
