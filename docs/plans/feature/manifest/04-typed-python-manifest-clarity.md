# Clarification

## Trigger

The user chose a Python-owned typed manifest for v1 and rejected RON and JSON as internal manifest source formats.

## Decision

Manifest entries will be ordinary typed Python declarations inside BonesInfra. BonesDeploy will invoke BonesInfra through the existing subprocess CLI boundary. JSON remains available only as rendered command output for automation. This avoids a parser dependency, duplicated cross-language schemas, and drift between manifest data and BonesInfra strategy logic.

## Supersedes

This supersedes the earlier decision to use RON documents and the experimental `python-ron` dependency as the v1 manifest source.

## Effect on the record

`01-idea.md` now defines typed Python declarations and JSON output. `02-plan.md` now assigns manifest ownership to Python code and removes parser/package work. `03-tasks.md` now tracks typed declarations, subprocess delegation, inspection, output, tests, and documentation without RON or JSON source files.
