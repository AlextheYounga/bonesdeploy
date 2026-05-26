# BonesDeploy Identity and User Policy

## Purpose

This section defines the Unix identities BonesDeploy should use.
The model is simple: one deploy user and one service user per project.

## Deploy User

- Handles SSH deployment work.
- Has a home directory.
- Has restricted sudo for narrow helper commands.
- Has no password login.

## Service User

- Runs the application.
- Has no home directory.
- Has no interactive login.
- Has no sudo.
- Should not own deploy metadata.

## Rules

- One project, one service user.
- Never share `www-data` or `deploy` across projects for app runtime.
- Service users should not read SSH keys or deploy tokens.
- Service users should not be able to mutate old releases.

## Findings

- service user in `sudo`/`wheel`
- service user has a login shell
- service user can read deploy keys
- multiple projects share one service user
- service user can write release directories
