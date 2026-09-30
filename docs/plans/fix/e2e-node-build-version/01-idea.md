# Idea

## Request

Record and repair the Next and Nuxt native E2E setup error on a dedicated Git
Flow bugfix branch.

## Problem

Next and Nuxt E2E scenarios fail before provisioning because
`SampleProject::pin_node_version` requires `.env.build` to contain an empty
`NODE_VERSION=` line. Current framework initialization writes its default Node
version instead, so the helper reports an error even though the generated build
environment is valid.

## Definitions

**Generated Node version:** The valid `NODE_VERSION` value written to
`.env.build` during Next or Nuxt project initialization.

**E2E Node pin:** The explicit Node version required by the native E2E harness
to make its fixture builds reproducible. It replaces, rather than requires the
absence of, the generated Node version.

## Desired outcome

Next and Nuxt E2E setup writes the E2E Node pin into the generated `.env.build`
whether its existing `NODE_VERSION` value is empty or defaulted. The helper
still fails clearly when that key is absent.

## Scope

- Update the native E2E project helper to replace a generated `NODE_VERSION`
  assignment.
- Add regression coverage for empty, defaulted, and missing Node version keys.
- Re-run Next and Nuxt native E2E scenarios after the local Docker builder image
  prerequisite is available.

## Constraints

- Preserve the current framework-generated Node defaults outside the E2E helper.
- Do not change the E2E pin version or general `.env.build` parsing behavior.
- Use a Git Flow bugfix branch with a linked worktree.

## Exclusions

- Pulling the pinned `buildpack-deps` image or installing Docker emulation is a
  local host prerequisite, not a repository code change.
- Laravel's builder-image failure and the Compose placeholder doctor failure are
  separate issues.
