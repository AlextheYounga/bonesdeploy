# Clarification

## Trigger

The prior wording about removing config-root copies could be read as deleting
the entire `~/.config/bonesdeploy` tree or moving GPG private keys into the
application repository. The user clarified that machine-local GPG state must
remain protected and available.

## Decision

The refactor removes only the obsolete per-project configuration repository,
its nested Git transport, and its project-local workspace representation. It
does not delete or relocate the BonesDeploy XDG data/configuration roots.

GPG private keys, keyrings, trust data, and other decryption authority remain
machine-local. Only encrypted project secret ciphertext may be placed under
committed `infra/secrets/`; plaintext secrets and private keys never enter Git
or build execution. Migration copies encrypted bytes without decrypting them
and leaves the local keyring untouched.

## Supersedes

Supersedes the broad interpretation of “remove config-root copies” and
“remove the old structure” from the preceding planning record. Those phrases
refer only to obsolete per-project configuration state, not machine-local
BonesDeploy state or GPG key material.

## Effect on the record

- `01-idea.md`: Defines local key material, narrows removal to per-project
  configuration state, and requires preservation of local BonesDeploy roots.
- `02-plan.md`: Separates encrypted project ciphertext from the machine-local
  GPG keyring and makes migration preserve local key material.
- `03-tasks.md`: Requires preserving the keyring during initialization,
  secret relocation, and migration, while forbidding private keys in Git or
  build inputs.
