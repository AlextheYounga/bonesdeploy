# Tasks

## Implementation

- [x] Replace `openssh` and its Unix mux dependency tree with a
  `tokio::process`-based `SshTransport` that invokes system `ssh`/`ssh.exe` once
  per operation using separate `-p`, port, `--`, and destination arguments plus
  one unquoted remote-command argument containing
  `exec bash -c '<escaped-command>'`, while preserving stdin/upload/download
  streaming, bounded output, deadlines, diagnostic tails, and child cleanup.
- [x] Update every SSH caller to the process transport lifecycle, remove session
  close handling that no longer represents persistent state, and add clear
  missing-OpenSSH diagnostics without changing remote command contents.
- [x] Replace the existing `openssh`-coupled transport fixtures with crate-root
  tests for fake `ssh`/`ssh.exe` invocation and Linux local-sshd contract tests
  covering success, remote failure, output limits, all streaming forms,
  timeouts, cleanup, shell metacharacters, and reuse after failure.
- [x] Add Windows workstation-root selection in
  `bonesdeploy-core::paths`: use `APPDATA` for configuration and
  `LOCALAPPDATA` for data, cache, and state while preserving all existing XDG
  behavior and remote Linux path constants.
- [x] Project the canonical local data root into BonesInfra and update its local
  patch registry to use that value for markers instead of independently
  reconstructing XDG or home-relative paths.
- [x] Add focused local-file platform helpers and apply them to embedded assets,
  generated artifacts, infrastructure refresh, GPG homes, encrypted and
  temporary secrets, and shared-data exports; retain Unix modes and use Windows
  profile ACL inheritance without broadening access.
- [x] Implement Windows reparse-point detection with a target-specific
  `windows-sys` dependency and use it in shared-data import so only regular,
  non-link archive files are opened; retain `O_NOFOLLOW` on Unix and preserve
  no-clobber behavior on every platform.
- [x] Make export and atomic local writes close replaceable handles before
  rename, retain file flushes, use directory syncing only where the platform
  supports it, and prove Windows persistence and cleanup behavior.
- [x] Extend `build::DockerClient` to discover `docker.exe`, accept only the
  local Windows `npipe:////./pipe/` namespace on Windows, preserve Unix-socket
  validation on Unix, and continue rejecting remote or unstable selectors
  before Docker mutation.
- [x] Isolate native-build bind specifications and mount ownership policy so
  Windows drive-letter and space-containing paths reach Docker CLI unchanged,
  Unix UID/GID restoration remains intact, and Windows performs no synthetic
  POSIX ownership cleanup.
- [x] Add Windows-native fake-Docker/Compose fixtures that prove endpoint
  propagation, selector stability, bind arguments, environment isolation,
  image tagging/saving, and cleanup across native and Compose package flows.
- [x] Extend `BuildContext` with a validated relative-path/type/Unix-mode
  inventory; populate it from Git archive headers, replace it from a
  null-delimited Linux builder inventory after native builds, and register
  Compose-generated entries when they are created.
- [x] Make artifact generation consume that inventory and assign `0644` to
  generated Compose metadata, `0600` to generated image archives, `0644` to
  unrecorded ordinary files, `0755` to unrecorded directories, and `0777` to
  symlinks while retaining path, link, and root environment exclusions.
- [x] Replace the external `tar` process in committed-source export with safe
  streaming extraction through the existing Rust `tar` crate, including Git
  failure propagation, path traversal protection, Git mode capture, and safe
  Windows symlink materialization under Developer Mode.
- [x] Make the BonesInfra shim use `.venv\Scripts\python.exe` and
  `python.exe` on Windows, retain `.venv/bin/python` and `python3` on Unix,
  validate Python 3.12 or newer, and adapt its Rust/Python test launcher to the
  same platform contract.
- [x] Make secrets editor launch native to each host: retain current Unix shell
  behavior, invoke the configured editor through Windows command processing,
  and cover editor executable paths and edited file paths containing spaces.
- [x] Make local update continuation resolve the installed platform executable,
  preserve `cargo install` updates, and adapt copied deployment-asset
  permissions without changing the Linux remote-update artifact or commands.
- [x] Adapt local CLI integration fixtures to create `.cmd` or executable
  Windows fakes where behavior is portable; retain POSIX mode, symlink, Unix
  socket, and local-sshd assertions under explicit Unix configuration rather
  than silently dropping their coverage.
- [x] Add isolated missing-tool tests for Git, GnuPG, Cargo, curl, Python,
  Docker, OpenSSH, and the configured editor; each failure must name the
  prerequisite and executable required on Windows.
- [x] Add a normal GitHub Actions validation workflow with a `windows-latest`
  local-CLI job and a Linux non-E2E regression job. Install declared tool
  prerequisites explicitly and keep `bonesremote` out of Windows Cargo
  invocations.
- [x] Add a manually dispatched Windows Docker Desktop smoke workflow for a
  labeled self-hosted runner, covering named-pipe discovery, a drive-letter
  project path containing spaces, Linux/amd64 native build output and cleanup,
  Compose v2 validation/build, and real Windows `ssh.exe` commands against an
  ephemeral Linux sshd container with shell metacharacters in command input.
- [x] Add a tagged Windows release job that builds
  `bonesdeploy-x86_64-pc-windows-msvc.exe`, executes its version command,
  writes its SHA-256 checksum, and publishes both files without changing the
  existing Linux `bonesremote` release artifact.
- [x] Update `README.md`, `CONTEXT.md`, and the architecture platform boundary
  to document native Windows prerequisites, the package-specific Cargo build,
  Docker Desktop Linux-container requirements, Windows workstation paths,
  release files, and the unchanged Linux production-host contract.

## Validation

- [ ] On native Windows, run
  `cargo test --locked --package bonesdeploy-core` and verify workstation path
  tests use Windows application-data roots while remote paths remain Linux.
- [ ] On native Windows, run
  `cargo test --locked --package bonesinfra` with Python 3.12 and verify wheel
  materialization and invocation use `.venv\Scripts\python.exe`.
- [ ] On native Windows, run `ruff check .`, `ruff format --check .`, and
  `uv run pytest` from `crates/bonesinfra/python`; verify local patch markers
  resolve below the projected `LOCALAPPDATA` BonesDeploy data root.
- [ ] On native Windows, run
  `cargo test --locked --package bonesdeploy --tests` and verify the portable
  CLI, fake-tool, Docker endpoint, SSH process, Git export, artifact, secrets,
  import/export, and update contracts pass.
- [ ] On native Windows, run
  `cargo clippy --locked --package bonesdeploy --all-targets --all-features -- -D warnings`
  and the corresponding `bonesdeploy-core` and `bonesinfra` Clippy checks with
  no warning or Unix-only compilation failure.
- [ ] On native Windows, run
  `cargo build --locked --release --package bonesdeploy`, verify
  `target\release\bonesdeploy.exe` exists, and verify
  `target\release\bonesdeploy.exe version` succeeds.
- [ ] On a Windows Docker Desktop smoke runner, verify native and Compose
  package operations succeed from a path containing spaces, all containers and
  temporary tags are cleaned up, and resulting files remain readable,
  writable, and removable by the Windows user.
- [ ] Inspect an artifact produced on Windows and verify safe paths, expected
  Git and post-build modes, settled generated/fallback modes, environment
  exclusions, manifest digest, and successful validation by the Linux
  `bonesremote` artifact boundary.
- [x] On Linux, run the affected `bonesdeploy`, `bonesdeploy-core`,
  `bonesinfra`, `bonesremote`, and clean-code test suites and verify existing
  Unix SSH, socket, ownership, mode, symlink, import/export, and deployment
  behavior remains unchanged; do not run e2e tests.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` from the repository root
  and address every warning or error without formatting unrelated files.
- [ ] Exercise the tagged-release workflow and verify the GitHub release
  contains the Windows `.exe` and checksum plus the existing Linux
  `bonesremote` binary and checksum, with each version command matching the tag.

## Completion

- [x] Remove obsolete `openssh`, mux, and Unix-only local transport code and
  confirm `Cargo.lock` contains no unused replacement dependency.
- [x] Review all `cfg` usage and confirm it separates genuine platform behavior
  rather than hiding unsupported local CLI commands or untested code.
- [x] Review security-sensitive filesystem changes for reparse-point rejection,
  no-clobber writes, private local material, archive traversal, and remote
  Docker rejection on both platforms.
- [x] Review the final diff against `01-idea.md` and `02-plan.md`; confirm it
  contains no Windows `bonesremote`, Windows server provisioning, Windows
  containers, WSL branch, remote Docker, Podman, installer, signing, or
  unrelated refactor.
- [x] Record final Windows and Linux validation evidence, workflow results,
  material plan deviations, and deliberately unfinished work in the completion
  notes.

## Completion notes

The planning record was approved on 2026-10-05. All implementation tasks are
complete.

Linux validation passed for the complete non-E2E Rust workspace, including the
clean-code suite and real local OpenSSH transport tests. Strict workspace
Clippy, `cargo fmt`, `shfmt`, Ruff, all 532 embedded Python tests, actionlint,
and `git diff --check` passed. The regenerated embedded BonesInfra wheel matches
the Python source.

The `x86_64-pc-windows-msvc` target passes `cargo check` and strict Clippy for
the complete `bonesdeploy` package and all test targets. Native Windows tests,
the Docker Desktop/OpenSSH smoke workflow, inspection of an artifact produced
on Windows, and the tagged-release workflow remain deliberately unchecked
until they run on their required Windows/GitHub environments.

Implementation review found and resolved Windows archive separator/mode
handling, artifact reparse-point traversal, nested Git symlink chains, smoke
workflow trusted-ref execution, and cancellation cleanup. Final implementation
and workflow re-reviews reported no remaining concrete findings.
