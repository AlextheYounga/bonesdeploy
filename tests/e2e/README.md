# E2E Tests

This package contains end-to-end tests for BonesDeploy.

These tests validate real workflows across:

- local repository state
- SSH transport
- remote Docker host behavior
- `bonesdeploy` CLI orchestration
- `bonesremote` server-side commands and hooks

## Running

Ignored by default:

```bash
cargo test -p bonesdeploy-e2e-tests
```

Run Docker-backed E2E tests explicitly:

```bash
./tests/e2e/run-e2e.sh
```

`run-e2e.sh` performs a one-time Docker lifecycle step before tests:

- `docker compose down --remove-orphans`
- `docker compose up -d`

The container remains running after the test suite completes.

## Bootstrap SSH User

The test harness defaults to a bootstrap SSH user of `root`.

Override with:

```bash
export BONES_E2E_BOOTSTRAP_USER=my-sudo-user
```
