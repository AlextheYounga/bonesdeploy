# Plan

## Current behavior

`bonesdeploy` is the developer-workstation CLI. It exports one committed Git
tree, builds it locally through Docker when required, packages an artifact, and
uses SSH to provision or invoke the Linux-only `bonesremote` release lifecycle.
`bonesinfra` embeds a pure-Python wheel and creates a workstation-local virtual
environment before running pyinfra against the production host.

The local CLI is Unix-only in several independent boundaries:

- `crates/bonesdeploy/Cargo.toml` enables `openssh` with `native-mux`.
  `openssh` and its mux dependencies intentionally fail compilation on
  non-Unix targets. `infra/ssh.rs` exposes that crate's session and child types
  throughout the transport implementation.
- `build/native.rs`, `build/artifact.rs`, `infra/assets/mod.rs`,
  `commands/secrets/`, `commands/site/{import,export}.rs`, and
  `commands/update/sync.rs` directly use Unix permission, ownership, symlink,
  or open-flag APIs.
- `build/docker.rs` resolves an operation-scoped Docker endpoint but accepts
  only `unix://` sockets and searches only for an executable named `docker`.
  Docker Desktop exposes its Linux engine through a local `npipe://` endpoint
  and installs `docker.exe`.
- `build/native.rs` records host UID/GID and recursively restores those numeric
  owners after a Linux container build. Windows has no corresponding local
  UID/GID, and Windows drive-letter paths require explicit Docker CLI coverage.
- `bonesinfra/src/lib.rs` assumes `.venv/bin/python` and invokes `python3`.
- `infra/git.rs` pipes `git archive` through an external `tar` executable even
  though the crate already depends on Rust's `tar` crate.
- `commands/secrets/mod.rs` launches `$EDITOR` through `sh`, and workstation
  storage roots in `bonesdeploy-core/src/paths.rs` assume `HOME` and XDG paths.
- the local tests create executable shell fakes and Unix symlinks directly, and
  the only GitHub Actions workflow builds the Linux `bonesremote` release
  artifact on Ubuntu.

The remote Python operations and `bonesremote` correctly assume Linux. Absolute
paths such as `/srv`, `/etc`, and `/var`, and remote Bash commands, describe the
production host rather than the local workstation and must remain Linux paths.

## Intended behavior

The `bonesdeploy` package compiles natively for `x86_64-pc-windows-msvc` and
produces a runnable `bonesdeploy.exe`. The CLI uses Windows application-data
locations, Windows virtual-environment layout, `docker.exe`, Docker Desktop's
local named pipe, Windows OpenSSH Client, and Windows-safe process invocation.

All local CLI commands remain available. Commands that require an external tool
validate that tool and report an actionable error. Remote commands still run
under Bash on the supported Linux production host, and build scripts still run
inside the pinned Linux builder container.

Local Docker policy accepts exactly the platform's local transport: Unix socket
on Unix and named pipe on Windows. One resolved endpoint remains fixed for the
entire package operation. Native builds do not apply POSIX host ownership
restoration on Windows; Docker Desktop owns host-file identity translation.
Compose packaging continues to call `docker compose` so Docker Desktop's
Compose v2 implementation remains authoritative.

Committed source export records each Git archive entry's Unix type and mode
before Windows materialization loses that metadata. Native Linux builder
containers refresh the mode inventory for the complete post-build tree. The
artifact writer uses that inventory, assigns generated Compose metadata files
mode `0644`, generated image archives mode `0600`, ordinary files without a
recorded mode `0644`, directories without a recorded mode `0755`, and symlinks
mode `0777`. Unix continues to use source metadata. Import/export and secret
handling preserve regular-file, no-clobber, and private-file guarantees through
platform-specific filesystem operations.

Windows CI builds and tests only the local components. Linux CI continues to
test the complete non-E2E workspace, including the Linux-only `bonesremote`.
Tagged releases publish a Windows local-CLI executable and checksum alongside
the existing Linux remote binary.

## Approach

Replace the Unix-only `openssh` dependency with a process-based
`SshTransport` implemented with `tokio::process::Command`. Store the connection
arguments and policy in the transport, then start one system `ssh`/`ssh.exe`
process for each operation. Preserve the current command, upload, download,
bounded-output, timeout, stderr-tail, and child-cleanup contracts. Send one
remote-command argument in the exact form
`exec bash -c '<single-quote-escaped command>'` after the `--`-delimited
`user@host` destination; OpenSSH receives the port as a separate `-p` argument.
This leaves the remote login shell only one quoting layer and gives `bash -c`
the original command bytes. Use the user's standard OpenSSH configuration,
agent, host-key policy, and credentials; do not add a second SSH configuration
system or connection multiplexer.

Add focused platform helpers for local executable lookup, private/generated
file handling, application directories, virtual-environment Python, and editor
launching. Unix implementations preserve current modes and durability calls.
Windows implementations use Windows paths and inherited user-profile ACLs,
reject symlinks and reparse points at security-sensitive inputs, and avoid
pretending Windows has POSIX ownership. Keep these helpers at the owning crate
boundary instead of spreading conditional compilation through command entry
points.

Extend the existing Docker client boundary rather than adding Bollard. Resolve
Docker contexts through the Docker CLI as today, validate `unix://` only on
Unix and local `npipe:////./pipe/...` only on Windows, and pass the accepted
endpoint to every Docker and Compose command. Find the executable using the
platform executable suffix. Isolate Docker bind-spec construction and make
Windows drive-letter behavior observable in fake-Docker tests, followed by a
real Docker Desktop smoke test. Represent mount ownership as a Unix-only build
concern and omit ownership restoration on Windows.

Use the existing Rust `tar` crate to validate and unpack the `git archive`
stream directly, recording its entry types and Unix modes in `BuildContext` and
materializing safe symlinks through the platform API. After native build
scripts finish, query the still-running Linux builder for a null-delimited
relative-path/type/mode inventory and replace the source inventory before
artifact creation. Compose-generated files are recorded at creation. This
removes the local `tar` executable requirement without discarding Linux
metadata. Continue to require Git, GnuPG, Cargo, curl, Python, Docker, and
OpenSSH as explicit platform tools. Launch the configured Windows editor
through the native command processor while retaining the existing Unix shell
behavior, with tests for paths containing spaces. Continue local updates
through `cargo install`; resume the installed binary using platform-aware
executable resolution.

Use `APPDATA` and `LOCALAPPDATA` consistently across both Rust and the embedded
Python runtime. Pass the canonical local data root into BonesInfra so patch
markers do not independently reconstruct an XDG path on Windows. Use
`python.exe` from `PATH` on Windows and `python3` on Unix, verify the interpreter
is Python 3.12 or newer, and use the matching platform venv executable path.

Split local tests into portable behavior and explicitly Unix-specific
semantics. Build fake executables as `.cmd`/`.exe` fixtures on Windows rather
than executable shell files. Move new behavioral coverage to crate-root
integration tests and retain Unix-only assertions for POSIX modes and symlinks.
Add Windows CI before adding the tagged Windows release artifact so compilation,
Clippy, tests, startup, Docker endpoint behavior, and packaging are continuously
verified.

## Responsibilities and boundaries

- `bonesdeploy-core::paths`: owns workstation application-directory selection
  while retaining canonical remote Linux path constants.
- `bonesdeploy::infra::ssh`: owns system OpenSSH executable validation, remote
  process construction, streaming, deadlines, output limits, and cleanup.
- `bonesdeploy::build::docker`: owns platform-local Docker endpoint validation,
  executable discovery, selector stability, and Docker command construction.
- `bonesdeploy::build::native`: owns Linux-builder lifecycle, bind mounts, and
  platform-specific host ownership cleanup policy.
- `bonesdeploy::build::artifact`: owns portable artifact generation and the
  Linux metadata carried by the archive.
- `bonesdeploy::infra::git` and `build::source::BuildContext`: own committed-
  source export, safe in-process archive extraction, and the initial Linux mode
  inventory.
- `bonesdeploy::infra::assets`, secrets, site import/export, and update modules:
  own platform-safe local files and process invocation for their workflows.
- `bonesinfra` Rust shim: owns platform-specific Python discovery, virtual
  environment executable layout, and projection of the canonical local data
  root. The embedded Python package consumes that root while continuing to own
  Linux production provisioning.
- crate-root `tests/`: owns portable CLI behavior, platform-specific fake-tool
  fixtures, and observable integration coverage.
- GitHub Actions: owns native Windows compilation/test evidence and release
  artifact production; the existing workflow retains Linux remote releases.

## Affected areas

- `Cargo.toml` and `Cargo.lock`: remove the Unix-only SSH dependency tree and
  add only dependencies required for the chosen cross-platform implementation.
- `crates/bonesdeploy/Cargo.toml`: remove `openssh`; keep Tokio process support.
- `crates/bonesdeploy-core/src/paths.rs`: add Windows workstation roots without
  changing remote Linux constants.
- `crates/bonesdeploy/src/infra/ssh.rs`: implement process-based system SSH.
- `crates/bonesdeploy/src/build/{docker,native,artifact,source}.rs`: support
  named pipes, Windows executable and bind paths, ownership policy, private
  temporary files, and the Linux mode inventory carried into artifacts.
- `crates/bonesdeploy/src/infra/{git,assets/mod.rs}`: remove external tar,
  capture Git archive metadata, materialize safe symlinks, and preserve
  deployment-asset semantics cross-platform.
- `crates/bonesdeploy/src/commands/secrets/`: protect local material and launch
  editors without requiring a Windows host shell compatible with `sh`.
- `crates/bonesdeploy/src/commands/site/{import,export}.rs`: implement portable
  secure file checks, persistence, and durability behavior.
- `crates/bonesdeploy/src/commands/update/{mod,release,sync}.rs`: use portable
  executable resolution and file handling.
- `crates/bonesinfra/src/lib.rs`, `crates/bonesinfra/tests/pytest.rs`, and
  `crates/bonesinfra/python/src/bonesinfra/patches/registry.py`: support Windows
  Python, venv, and local patch-marker paths.
- `crates/bonesdeploy/tests/` and `crates/bonesdeploy-core/tests/`: add portable
  and Windows-specific integration fixtures while retaining Unix security
  assertions.
- `.github/workflows/`: add Windows and Linux non-E2E validation and publish the
  Windows local-CLI artifact on version tags.
- `README.md`, `CONTEXT.md`, and architecture documentation: distinguish
  supported workstation platforms from Linux-only production hosts and list
  Windows prerequisites, build, Docker, and release behavior.

## Decisions

- Native Windows support means the MSVC `bonesdeploy` package, not the complete
  workspace. `bonesremote` remains deliberately non-compilable for Windows.
- The supported Windows build command names the package explicitly:
  `cargo build --locked --release --package bonesdeploy`.
- System OpenSSH replaces `openssh`. It is already a Windows platform feature,
  preserves users' SSH configuration and agents, and avoids introducing a new
  authentication and host-key implementation.
- SSH remote command construction has one canonical form:
  `Command` receives `-p`, `<port>`, `--`, and `<user>@<host>` as separate
  arguments, followed by one unquoted argument whose bytes are
  `exec bash -c '<escaped-command>'`. The double quotes commonly shown around
  that final argument are command-line notation and are not included in the
  argument. The existing POSIX single-quote escaping function produces
  `<escaped-command>`, and no Windows shell interprets these process arguments
  locally.
- Each SSH operation uses its own process. Connection multiplexing is not part
  of the required behavior, and preserving it would reintroduce platform-
  specific control-socket complexity.
- The Docker CLI remains the Docker API boundary. It already supports Windows
  named pipes and Compose v2; Bollard would not replace Compose and would add a
  second Docker behavior surface.
- Windows accepts only local named-pipe Docker endpoints. Broad `npipe://`
  acceptance is constrained to the local `//./pipe/` namespace; remote schemes
  remain rejected before source mounting or image mutation.
- Windows host files do not receive synthetic UID/GID values. Docker Desktop's
  Linux-container file sharing is responsible for access, and real smoke tests
  prove that build output remains readable and removable by the Windows user.
- Git remains an external prerequisite, but tar extraction moves in-process
  because the dependency already exists and removes an unnecessary Unix tool.
- `BuildContext` carries Linux archive metadata. Git supplies the initial
  inventory, native builder inspection replaces it after build scripts, and
  Compose generation records its own files; this preserves executable outputs
  rather than inferring them from Windows metadata.
- GnuPG, Cargo, curl, Python, Docker, and OpenSSH remain explicit prerequisites
  rather than being reimplemented or bundled.
- Windows private files rely on the user's profile/temp-directory ACL and are
  created without broadening inherited access. Unix continues to enforce the
  existing explicit mode bits.
- Windows release delivery is a standalone `.exe` plus SHA-256 checksum. An
  installer and code signing are separate product work.

## Risks

- Replacing a persistent SSH session with one process per operation can expose
  quoting, authentication-prompt, timeout, and child-cleanup regressions across
  uploads and downloads. Transport contract tests must cover every operation
  shape and paths/commands containing spaces and shell metacharacters.
- Docker Desktop context endpoints vary between versions. Named-pipe validation
  must accept local Desktop pipe names without accepting TCP or SSH endpoints.
- Windows drive-letter and space-containing bind paths can be parsed
  differently by Docker CLI fixtures and the real daemon. Fake argument tests
  alone are insufficient.
- Windows files do not carry POSIX executable bits. Artifact generation must
  reject a malformed or incomplete builder metadata inventory and apply the
  settled fallback modes only to known host-generated entries, or deployment
  behavior can fail only after upload.
- Reparse points are broader than ordinary symlinks. A Windows import check that
  tests only `is_symlink` could permit link-like archive inputs and weaken the
  current `O_NOFOLLOW` boundary.
- Windows rename and open-handle semantics are stricter than Unix. Atomic
  update, export persistence, and temporary-file cleanup tests must keep handles
  closed before replacement.
- Python installations differ in whether `python.exe` or the `py.exe` launcher
  is available. The supported contract requires `python.exe` on `PATH`; the
  Python 3.12 version check must reject aliases and older interpreters with an
  actionable diagnostic.
- Existing integration fixtures are shell scripts. Merely gating them out
  could yield a compiling Windows suite with little behavioral evidence, so
  critical Docker, SSH, Git, update, and process boundaries need Windows-native
  fakes.

## Validation

- On `windows-latest`, `cargo test --locked --package bonesdeploy-core` and
  `cargo test --locked --package bonesdeploy --tests` pass using Windows-native
  fake executables; tests prove Windows paths with spaces, editor invocation,
  Python venv layout, Git archive extraction, secure import rejection, export
  persistence, update continuation, Docker named-pipe acceptance, remote
  endpoint rejection, and selector stability.
- On `windows-latest`, strict Clippy passes for all local targets and
  `cargo build --locked --release --package bonesdeploy` produces
  `target\release\bonesdeploy.exe`; running `bonesdeploy.exe version` succeeds.
- SSH transport tests prove bounded output, stdin streaming, reader failure,
  download streaming, unsuccessful remote exit diagnostics, timeout cleanup,
  subsequent transport reuse, and the exact canonical remote command through
  the system OpenSSH process boundary. The Windows smoke runner connects with
  real `ssh.exe` to an ephemeral Linux sshd container and runs commands
  containing spaces, single quotes, dollar signs, and semicolons.
- A Windows Docker Desktop smoke test confirms the discovered local named pipe,
  a source path containing a drive letter and spaces, Linux/amd64 builder
  execution, generated output accessibility, cleanup, and `docker compose`
  validation/build behavior. The test is isolated from ordinary unit tests and
  enabled only on a runner with Docker Desktop.
- Artifact inspection on both Linux and Windows proves preservation of Git and
  post-build modes, `0644` Compose metadata, `0600` image archives, `0644`
  fallback files, `0755` fallback directories, `0777` symlinks, and root `.env`
  exclusions.
- Windows runs the embedded Python package's `ruff check`, `ruff format
  --check`, and `uv run pytest` commands with its local data root under
  `LOCALAPPDATA`; patch marker tests prove the canonical root is used.
- Fake-tool tests remove each required executable from the isolated `PATH` in
  turn and prove Git, GnuPG, Cargo, curl, Python, Docker, OpenSSH, and editor
  failures name the missing prerequisite and executable.
- Linux runs the affected crate tests plus the non-E2E workspace regression
  suites, `cargo clippy`, `cargo fmt`, and `shfmt -w .`; existing Unix socket,
  POSIX mode, ownership, symlink, and Linux SSH behavior remains covered.
- Release validation builds the Windows MSVC executable natively, executes its
  version command, generates a SHA-256 checksum, and publishes the named
  Windows files without changing the Linux `bonesremote` artifact contract.
- Documentation review confirms that Windows is described only as a local
  workstation platform and that all production-host requirements remain Linux.
- The final diff contains no Windows server behavior, WSL-specific branch,
  remote Docker support, Podman support, installer, or unrelated refactor.
