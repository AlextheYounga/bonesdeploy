# Idea

## Request

Allow BonesDeploy to build and run as a native Windows Rust executable. Replace
or adapt workstation-side crates and code that do not support Windows while
preserving deployment to the existing Linux production hosts.

## Problem

The `bonesdeploy` local CLI cannot currently compile for Windows. It depends on
the Unix-only `openssh` crate and imports Unix filesystem APIs directly for
permissions, ownership, and safe file opening. Its Docker boundary accepts only
Unix sockets, its embedded Python runtime assumes a POSIX virtual environment,
and several local workflows invoke Unix-specific tools or shell behavior.

Docker Desktop can provide the required Linux containers and Docker Compose on
Windows, but the host-side CLI cannot reach those capabilities until its local
process, filesystem, path, and Docker boundaries support Windows explicitly.

## Definitions

**Native Windows support:** A `bonesdeploy.exe` compiled for
`x86_64-pc-windows-msvc` and run directly from PowerShell or Command Prompt on
64-bit Windows. Running the Linux binary inside WSL does not qualify.

**Windows workstation:** A supported native Windows environment with Git,
Windows OpenSSH Client, Python 3.12 or newer, Cargo, curl, and GnuPG available
on `PATH`, plus an `EDITOR` command for secrets editing. Local artifact builds
additionally require Docker Desktop configured to run Linux containers with
Docker Compose v2. Windows Developer Mode is enabled so committed project
symlinks can be materialized without elevation.

**Local CLI:** The `bonesdeploy` binary and the workstation-side portions of
`bonesdeploy-core` and `bonesinfra`. It includes project initialization,
configuration, local packaging, native and Compose builds, provisioning over
SSH, deploy and release commands, secrets, shared-data import/export, doctor,
and update workflows.

**Production host:** The remote x86_64 Debian or Ubuntu Linux server managed by
BonesDeploy. Windows support does not change this target.

**Local Docker endpoint:** The operating-system-native endpoint for a Docker
daemon on the workstation: an absolute Unix socket on Unix or a local Windows
named pipe on Windows. TCP, SSH, and other remote daemon endpoints remain
unsupported because local builds mount workstation paths.

## Desired outcome

On a supported Windows workstation,
`cargo build --locked --release --package bonesdeploy` produces
`target\release\bonesdeploy.exe`. The executable starts successfully and the
documented local CLI workflows operate against supported Linux production
hosts. Native and Compose packaging use Docker Desktop's Linux engine through
its local named pipe, and generated deployment artifacts retain the Linux
semantics required by the production host.

The same source continues to compile and behave as before on Linux. Automated
Windows validation prevents the local CLI from regressing to Unix-only code.

## Scope

- Support the local CLI on native 64-bit Windows using the MSVC Rust target.
- Make local configuration, data, cache, state, temporary files, and embedded
  BonesInfra environments use valid Windows paths and executables.
- Support Windows OpenSSH Client for every SSH command and streaming transfer
  performed by the local CLI.
- Support Docker Desktop named-pipe contexts, Linux-container native builds,
  and Docker Compose v2 packaging.
- Preserve artifact security checks and required Linux archive metadata when
  source and artifacts originate on Windows.
- Make local Git, GnuPG, editor, update, import, and export workflows portable.
- Add Windows-native automated tests, CI validation, release compilation, and
  user documentation.
- Preserve Linux local CLI behavior and Linux release validation.

## Constraints

- Production hosts and `bonesremote` remain Linux-only and retain their current
  systemd, AppArmor, nginx, POSIX ownership, and filesystem contracts.
- Windows builds target `x86_64-pc-windows-msvc`; other Windows architectures
  and the GNU Windows target are not supported by this change.
- Docker Desktop must run Linux containers. Windows containers are not a
  deployment or build target.
- Docker access remains local-only. Windows support must not permit remote
  Docker endpoints or weaken selector-stability validation.
- Remote commands and deployment scripts continue to use Bash on the Linux
  production host or inside the Linux builder container; Bash is not required
  as a Windows host shell.
- Security-sensitive regular-file, symlink/reparse-point, no-clobber, secret,
  and archive validation must retain equivalent protection on Windows rather
  than being removed for portability.
- External tool failures must identify the missing Windows prerequisite and
  executable clearly.
- Behavioral tests belong in each crate's root `tests/` directory. E2E tests
  remain excluded unless explicitly requested.
- Implementation begins only after human review and approval of this planning
  record.

## Exclusions

- Running `bonesremote` on Windows or using Windows as a production host.
- Windows containers, Windows service management, IIS, or Windows server
  provisioning.
- Native ARM64 Windows and `x86_64-pc-windows-gnu` support.
- WSL-specific installation or behavior as the supported Windows solution.
- Podman support or remote Docker daemon support.
- Bundling or installing Docker Desktop, Git, OpenSSH, Python, Cargo, GnuPG, or
  an editor, or enabling Windows Developer Mode.
- An MSI/MSIX installer, package-manager integration, automatic code signing,
  or graphical interface.
