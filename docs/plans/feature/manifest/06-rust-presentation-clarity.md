# Rust Presentation Clarity

## Trigger

The Python-rendered text tree made terminal color propagation unreliable and split public CLI presentation across the Rust and Python process boundary.

## Decision

BonesInfra returns only the inspected JSON report. The public Rust command deserializes a narrow report DTO and owns tree construction, status-only text, terminal color, and JSON passthrough. Artifact kinds, owners, and detailed service state remain available in JSON but are omitted from the human tree.

## Supersedes

This supersedes the decision that Rust delegates format selection and does not interpret manifest entries.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are updated to reflect the Rust presentation boundary.
