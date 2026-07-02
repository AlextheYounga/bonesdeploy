# Build Scripts

Scripts in this directory run inside a disposable Podman container.

## Environment

- Working directory: `/workspace/source`
- No access to `.env`, `shared/`, `releases/`, the database, or host services.
- No access to `/root`, `.git`, or the bare repo.

## Contract

- Scripts run in lexical order by filename.
- Non-zero exit code fails the deploy.
- Your job: produce the deployable app layout inside `/workspace/source`.
- bonesremote will promote (hardened copy) this output into a sealed release.

## Adding Scripts

Name them with a numbered prefix so the order is clear:

```text
01_install_deps.sh
02_build_assets.sh
```

No secrets, no runtime state, no .env. Build only.
