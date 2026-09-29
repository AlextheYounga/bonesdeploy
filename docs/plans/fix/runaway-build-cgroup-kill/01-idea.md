# Idea

## Request

Replace the weak process-level attempts to stop runaway native builds with a
guaranteed build-user cgroup kill after a build-script timeout. Pin generated
Next and Nuxt projects to Node `25.9.0` by default because repeated Node 24
builds have exhausted memory during manual deployment testing.

## Problem

BonesRemote currently applies `RuntimeMaxSec` to a transient `systemd-run`
command that invokes `podman exec`. When the command times out, systemd stops
the Podman client unit, but the build payload remains alive in Podman's separate
container scope. BonesRemote then runs `podman rm --force` through the same
resource-starved build-user session. During the confirmed Next incident that
cleanup also timed out, leaving `next build` alive for more than two hours.

The dedicated build-user slice had an effective memory maximum of 773.8 MiB on
a 967 MiB server. The build stayed just below that limit while exhausting
global host memory. SSH and HTTPS stopped responding until the kernel's global
OOM killer killed `next-build`. The existing timeout therefore reports failure
without guaranteeing that the resource consumer stopped.

Generated framework configuration also assigns the same Node 24 default to
every framework. Next and Nuxt consequently reproduce a Node version that has
repeatedly exhibited unacceptable memory behavior in these deployments.

## Definitions

**Build-user cgroup:** The cgroup-v2 `user-<uid>.slice` belonging to one site's
dedicated `<site>-build` identity. It contains that identity's systemd user
manager, rootless Podman container payloads, and supporting processes. It does
not contain the site's runtime services, which use the separate `<site>` UID.

**Cgroup kill:** A root-authorized write to the build-user cgroup's
`cgroup.kill` control file followed by verification that the cgroup is absent
or reports `populated 0`. This is the kernel-enforced containment operation;
signalling an individual PID or asking Podman to remove a container is not a
cgroup kill.

**Build timeout:** A build script reaching its configured
`build.timeout_seconds` deadline and receiving systemd's `Result=timeout`.
Ordinary nonzero script exits are build failures, not timeouts.

**Framework Node default:** The exact Node version written into a newly
generated framework's runtime configuration and committed `.env.build`.
Explicit project configuration continues to override this generated default.

## Desired outcome

When a native build script times out, BonesRemote kills the complete dedicated
build-user cgroup and verifies that it no longer contains processes before
returning the timeout failure. No Node worker, Podman helper, or build-user
manager from that build can continue consuming host resources. Cancellation of
an interrupted build uses the same fail-closed boundary.

New Next and Nuxt projects consistently pin Node `25.9.0` for both build and
runtime behavior. Other framework defaults and existing explicit pins remain
unchanged.

## Scope

- Give timed build-script units stable identities long enough for BonesRemote
  to distinguish systemd timeout results from ordinary command failures.
- Kill and verify the complete dedicated build-user cgroup after a timeout.
- Prevent timeout unwinding from falling back to Podman cleanup in the killed
  session.
- Use the same cgroup termination operation for release cancellation while the
  deployment is still in a cancellable build phase.
- Remove process/container cleanup behavior made obsolete by the cgroup kill.
- Generate Next and Nuxt runtime and `.env.build` defaults with Node `25.9.0`.
- Add focused regression tests and update security/context documentation.

## Constraints

- Cgroup termination is executed by root and must not depend on Podman, the
  build user's systemd manager, or cooperation from the runaway process.
- The killed boundary is the dedicated build-user slice; runtime application
  services must remain running.
- A timeout is not considered contained until the cgroup is absent or reports
  `populated 0`.
- Provisioning must reject native build hosts that do not expose the required
  cgroup-v2 `cgroup.kill` control for the dedicated build-user slice.
- Containment verification is bounded; failure to prove an empty cgroup within
  the deadline is a hard error and must not permit release-state cleanup.
- Ordinary build failures retain normal container cleanup.
- A later deployment may restart the build-user manager and remove stale
  Podman metadata before creating a new container.
- Node versions remain exact `X.Y.Z` pins. Explicit user-selected versions are
  not rewritten.
- Do not run the repository end-to-end test suite.

## Exclusions

- Moving builds off the production server or implementing local artifacts.
- Changing the existing build-user memory, CPU, swap, task, disk, or I/O
  resource limits.
- Changing runtime-service cgroups or terminating runtime application users.
- Automatically rewriting existing projects from Node 24 to Node 25.
- Changing Node defaults for Django, Laravel, Rails, SvelteKit, Vue, or custom
  projects.
- Proving that Node 25 uses less memory for every Next or Nuxt workload.
