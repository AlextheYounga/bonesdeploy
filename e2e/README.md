# End-to-end tests

Runs bonesdeploy against real Incus system containers. Unlike Docker, Incus
containers boot a full systemd as PID 1, so `systemd-run`, `systemctl`,
AppArmor, and fail2ban behave like they do on an actual VPS.

The ignored `docker_compose` scenario runs nested rootful Docker in the Incus
guest. It covers a custom project Dockerfile, base plus override files,
web/database/worker services, multiple networks, health-gated startup, direct
ports plus loopback nginx ingress, persistent named volumes across a second deployment,
and restoration of the previous release after an unhealthy activation. It is
not part of ordinary local test runs.

## One-time host setup

```sh
sudo systemctl enable --now incus
sudo incus admin init --minimal
sudo usermod -aG incus-admin $USER   # then re-login
```

If the host firewall default-denies input, allow DHCP/DNS on the Incus
bridge or containers never get an IPv4 address:

```sh
sudo ufw allow in on incusbr0
sudo ufw route allow in on incusbr0
sudo ufw route allow out on incusbr0
```

The musl target for the container-side `bonesremote` binary is installed
automatically on first run (`rustup target add x86_64-unknown-linux-musl`).
Native scenarios also require the host Docker daemon: they build artifacts on
the test workstation, not inside the Incus guest.

## Running

```sh
cargo test-e2e
```

The alias (in `.cargo/config.toml`) expands to
`cargo test -p e2e -- --ignored --test-threads=1 --nocapture`. Tests are
`#[ignore]`d so `cargo test --workspace` stays fast and offline.
`--test-threads=1` is required: test scenarios share the Incus daemon and
stream subprocess output to the terminal.

### Running individual framework tests

The setup suite is split into one test per framework. All tests within the
same test binary share a single container (the first test run bootstraps
the server; the rest reuse it). Run a subset by passing a test-name filter:

```sh
# Single framework
cargo test -p e2e --test setup -- angular --ignored --test-threads=1 --nocapture

# Multiple native artifact frameworks
cargo test -p e2e --test setup -- angular vue --ignored --test-threads=1 --nocapture

# One native artifact framework scenario
cargo test -p e2e --test setup -- laravel --ignored --test-threads=1 --nocapture
```

Test names: `angular`, `django`, `laravel`, `next_server`, `next_static`, `nuxt_server`,
`nuxt_static`, `rails`, `sveltekit`, and `vue`. Every native artifact scenario
covers first deploy, a second release, and failed activation rollback after the
nginx service restart phase. Rails and Django scenarios also verify their exact
configured site-linked Ruby/Python versions; dynamic Next, Nuxt, and SvelteKit
scenarios verify the site-linked Node version. Run these ignored scenarios
manually; they are not routine validation.

### Managed runtime verification

Run these manually after the wheel is regenerated:

```sh
cargo test -p e2e --test setup -- rails --ignored --test-threads=1 --nocapture
cargo test -p e2e --test setup -- django --ignored --test-threads=1 --nocapture
cargo test -p e2e --test setup -- next_server --ignored --test-threads=1 --nocapture
cargo test -p e2e --test setup -- nuxt_server --ignored --test-threads=1 --nocapture
cargo test -p e2e --test setup -- sveltekit --ignored --test-threads=1 --nocapture
```

Each scenario requires a stable per-site link under
`/srv/sites/<site>/.bonesdeploy/runtimes/`, executed as the site user. Expected
version output begins with `ruby 3.4.8` for Rails, `Python 3.14.0` for Django,
and is exactly `v24.19.0` for dynamic Node services.

## How it works

- **Base image** — on first run a Debian container is prepared with sshd and
  published as the local image `bonesdeploy-e2e-base`.
- **Shared container** — all tests in a test binary share a single Incus
  container launched lazily on the first `Harness::create()`. The first test
  pays the bootstrap cost; subsequent tests skip it. The container is
  deleted at process exit via a `#[dtor]` hook (fires even when tests are
  filtered, panicked, or killed cleanly).
- **Isolated session** — each run gets a throwaway `HOME` under `target/e2e/`
  with its own SSH keypair, ssh config, and gitconfig. Your real `~/.ssh` is
  never read or written. The XDG config, data, and cache roots point under
  `target/e2e/`; the XDG state root uses a short, isolated `/tmp` path so SSH
  control sockets stay within Unix path limits. Project state, keyrings, and
  the materialized bonesinfra venv are isolated from the host.
- **Local binaries** — `bonesdeploy` is built for the host; `bonesremote` is
  built as a static musl binary and pre-seeded into the container, so
  bootstrap's `command -v bonesremote` guard skips the
  cargo-install-from-GitHub path and the container runs your working tree.
- **Artifact builds** — native scenarios build the committed revision locally
  with Docker and upload the complete artifact. Compose scenarios likewise run
  config, pulls, and image builds locally, then upload the release tree and
  image archive. The guest receives artifacts only through BonesRemote's receipt
  path; it does not run application builds, image pulls, or image builds.
- **Framework fixtures** — `fixtures/*.md` are mdpack archives of real
  framework projects. Each scenario expands its archive into a disposable local
  Git repository and runs `bonesdeploy deploy`; no production application
  repository or first push is required. Project setup creates the encrypted default
  `infra/secrets/.env.gpg`; the runtime seeds the remote shared environment file
  itself.
- **Artifact-only production** — native and Compose scenarios assert that the
  guest has no application bare repository, application build unit, or leftover
  artifact staging directory. The host performs the build and the guest only
  receives and activates the artifact.
- **Lifecycle coverage** — every native scenario covers artifact-only first
  deploy, second release, failed activation rollback, and pruning the old
  release. The Compose scenario additionally checks that release image tags are
  recorded and used by the generated override, then prunes the old release and
  its unreferenced image.
- **Cleanup** — sample project directories are dropped at the end of each
  test. The shared container and session home are dropped at process exit via
  a `#[dtor]` hook (the `dtor` crate registers a destructor that fires when
  the test binary exits, even on panics or filtered runs).

## Environment knobs

| Variable | Effect |
| --- | --- |
| `BONES_E2E_KEEP=1` | Keep containers and scratch dirs after the run for inspection |
| `BONES_E2E_REBUILD=1` | Rebuild the cached base image |
| `BONES_E2E_IMAGE=...` | Upstream image for the base (default `images:debian/13`) |

## Debugging

```sh
BONES_E2E_KEEP=1 cargo test-e2e
incus list bones-e2e            # harness containers share this prefix
incus exec <name> -- bash       # poke around the box
incus delete --force <name>     # clean up when done
```

If a run is killed hard (drop guards never fire), stray containers keep the
`bones-e2e` prefix and are safe to delete.
