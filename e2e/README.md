# End-to-end tests

Runs bonesdeploy against real Incus system containers. Unlike Docker, Incus
containers boot a full systemd as PID 1, so `systemd-run`, `systemctl`,
AppArmor, and fail2ban behave like they do on an actual VPS.

## One-time host setup

```sh
sudo systemctl enable --now incus
sudo incus admin init --minimal
sudo usermod -aG incus-admin $USER   # then re-login
```

Root needs a subordinate uid/gid range wide enough for nested user
namespaces (rootless podman runs inside the test container, and its build
user's subuids sit above the first 65536 ids):

```sh
echo "root:100000:1000000000" | sudo tee /etc/subuid /etc/subgid
sudo systemctl restart incus
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

## Running

```sh
cargo test-e2e
```

The alias (in `.cargo/config.toml`) expands to
`cargo test -p e2e -- --ignored --test-threads=1 --nocapture`. Tests are
`#[ignore]`d so `cargo test --workspace` stays fast and offline.
`--test-threads=1` is required: scenarios share the Incus daemon and stream
subprocess output to the terminal.

## How it works

- **Base image** — on first run a Debian container is prepared with sshd and
  published as the local image `bonesdeploy-e2e-base`. Every test launches
  from it, so per-test container startup is seconds, not minutes.
- **Isolated session** — each run gets a throwaway `HOME` under `target/e2e/`
  with its own SSH keypair, ssh config, and gitconfig. Your real `~/.ssh` is
  never read or written. `XDG_CONFIG_HOME` points at a shared cache so the
  materialized bonesinfra venv survives across runs.
- **Local binaries** — `bonesdeploy` is built for the host; `bonesremote` is
  built as a static musl binary and pre-seeded into the container, so
  bootstrap's `command -v bonesremote` guard skips the
  cargo-install-from-GitHub path and the container runs your working tree.
- **Rootless build networking** — each disposable guest selects Podman's
  `slirp4netns` backend. Debian's default `pasta` backend crashes in nested
  Incus containers; production provisioning is not changed.
- **Framework fixtures** — `fixtures/*.md` are mdpack archives of real
  framework projects. Each scenario expands its archive into a disposable Git
  repository, pushes `main`, and runs `bonesdeploy deploy`.
- **Cleanup** — containers, session homes, and sample projects are dropped at
  the end of each test, pass or fail.

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
