#!/usr/bin/env python3
"""Merge numbered Markdown spec files into one file per spec."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


SPECS_ROOT = Path("docs/specs")

# Keep this mapping explicit. It is the durable record of how historical spec
# folder names became the clearer names used by the single-file format.
SPEC_NAMES = {
    "feature/angular": "angular-static-site",
    "feature/backups": "borg-backups",
    "feature/cloudflare-quick-tunnel": "cloudflare-quick-tunnel",
    "feature/cloudflare-tunnel": "cloudflare-tunnel",
    "feature/decentralization": "project-owned-infrastructure",
    "feature/dedicated-build-command": "build-without-deploy",
    "feature/deletesite": "site-delete-command",
    "feature/deploy-preflight-ordering": "deploy-preflight-ordering",
    "feature/docker-runtime-backend": "docker-runtime-backend",
    "feature/etckeeper": "etckeeper-host-history",
    "feature/export": "site-shared-export",
    "feature/infra-binary": "packaged-bonesinfra-wheel",
    "feature/laravel-queue-worker": "laravel-queue-worker",
    "feature/local-artifact-builds": "local-artifact-builds",
    "feature/local-build-process-supervision": "local-build-process-supervision",
    "feature/local-build-serialization": "local-build-serialization",
    "feature/local-docker-endpoint-policy": "local-docker-endpoint-policy",
    "feature/manifest": "deployment-manifest-command",
    "feature/rename-deploy-user": "deploy-user-rename",
    "feature/selective-infra-wheel": "selective-infra-wheel",
    "feature/server-separate": "server-and-site-command-split",
    "feature/shared-import": "site-shared-import",
    "feature/ssh-transport-resilience": "ssh-transport-resilience",
    "fix/clarify-setup-scope": "clarify-server-and-site-setup-scope",
    "fix/compose-doctor-placeholder": "compose-doctor-placeholder",
    "fix/dockerfix": "docker-compose-runtime",
    "fix/e2e-incus-launch-config": "e2e-incus-launch-config",
    "fix/e2e-node-build-version": "e2e-node-build-version",
    "fix/gitdeploy": "git-ssh-deployment",
    "fix/per-site-nginx-preflight": "per-site-nginx-preflight",
    "fix/python-runtime-toolchain": "distribution-python-runtime",
    "fix/ruby-runtime-toolchain": "distribution-ruby-runtime",
    "fix/runaway-build-cgroup-kill": "runaway-build-cgroup-kill",
    "fix/rust-update-patches": "rust-update-patches",
    "fix/ssl-router-activation": "ssl-router-activation",
    "fix/ssl-site-setup-before-certificate": "ssl-site-setup-before-certificate",
    "refactor/config-centralization": "config-centralization",
    "refactor/external-public-tests": "external-public-tests",
    "refactor/framework-abstractions": "framework-abstractions",
    "refactor/project-infra": "project-owned-infrastructure",
    "refactor/project-local-bonesinfra-core": "project-local-bonesinfra-core",
    "refactor/python-update-patches": "python-update-patches",
}

TITLE_OVERRIDES = {
    "angular": "Angular",
    "borg": "Borg",
    "bonesinfra": "BonesInfra",
    "cloudflare": "Cloudflare",
    "compose": "Compose",
    "docker": "Docker",
    "e2e": "E2E",
    "git": "Git",
    "incus": "Incus",
    "laravel": "Laravel",
    "nginx": "Nginx",
    "php": "PHP",
    "python": "Python",
    "ruby": "Ruby",
    "rust": "Rust",
    "ssh": "SSH",
    "ssl": "SSL",
}

HEADING = re.compile(r"^( {0,3})(#{1,6})([ \t]+.*)$")


def title_for(slug: str) -> str:
    return " ".join(
        TITLE_OVERRIDES.get(word, word.capitalize())
        for word in slug.split("-")
    )


def source_files(directory: Path) -> list[Path]:
    return sorted(path for path in directory.glob("*.md") if path.is_file())


def demote_headings(text: str) -> str:
    """Move source headings below the merged document's title."""
    lines: list[str] = []
    fence: str | None = None

    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            marker = stripped[:3]
            fence = None if fence == marker else marker if fence is None else fence
            lines.append(line)
            continue

        if fence is None:
            match = HEADING.match(line)
            if match:
                level = min(len(match.group(2)) + 1, 6)
                line = f"{match.group(1)}{'#' * level}{match.group(3)}"
        lines.append(line)

    return "\n".join(lines).rstrip()


def merged_content(category: str, slug: str, files: list[Path]) -> str:
    sections = [f"# {title_for(slug)}", "", f"<!-- Merged from {category}/{files[0].parent.name} -->"]
    for path in files:
        sections.extend(["", demote_headings(path.read_text()), ""])
    return "\n".join(sections).rstrip() + "\n"


def discover_source_directories() -> list[Path]:
    return sorted(
        path
        for category in SPECS_ROOT.iterdir()
        if category.is_dir() and category.name != "archive"
        for path in category.iterdir()
        if path.is_dir() and source_files(path)
    )


def relative_spec_key(path: Path) -> str:
    return path.relative_to(SPECS_ROOT).as_posix()


def validate(source_directories: list[Path]) -> list[str]:
    errors: list[str] = []
    discovered = {relative_spec_key(path) for path in source_directories}

    for key in sorted(discovered - SPEC_NAMES.keys()):
        errors.append(f"missing name mapping for {key}")
    destinations: dict[Path, str] = {}
    for key, slug in SPEC_NAMES.items():
        destination = SPECS_ROOT / key.split("/", 1)[0] / f"{slug}.md"
        previous = destinations.get(destination)
        if previous is not None:
            errors.append(f"duplicate destination {destination}: {previous}, {key}")
        destinations[destination] = key

    return errors


def migrate(source_directories: list[Path], *, write: bool, remove_sources: bool, force: bool) -> None:
    for source in source_directories:
        key = relative_spec_key(source)
        slug = SPEC_NAMES[key]
        category = key.split("/", 1)[0]
        destination = SPECS_ROOT / category / f"{slug}.md"
        files = source_files(source)
        unexpected_files = [path for path in source.iterdir() if path not in files]
        if remove_sources and unexpected_files:
            names = ", ".join(str(path) for path in unexpected_files)
            raise RuntimeError(f"refusing to remove {source}; unexpected files: {names}")
        content = merged_content(category, slug, files)

        if destination.exists() and not force:
            raise RuntimeError(f"destination already exists (use --force): {destination}")

        action = "write" if write else "would write"
        print(f"{action} {destination} ({len(content.encode())} bytes)")
        if write:
            destination.write_text(content)

        if remove_sources:
            action = "remove" if write else "would remove"
            print(f"{action} {source}")
            if write:
                for path in files:
                    path.unlink()
                source.rmdir()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="validate the migration manifest")
    mode.add_argument("--write", action="store_true", help="write merged files")
    mode.add_argument("--dry-run", action="store_true", help="show planned writes without changing files (the default)")
    parser.add_argument("--remove-sources", action="store_true", help="remove source folders after writing")
    parser.add_argument("--force", action="store_true", help="overwrite existing destination files")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    source_directories = discover_source_directories()
    errors = validate(source_directories)
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1

    if args.check:
        print(f"validated {len(source_directories)} spec directories")
        return 0

    try:
        migrate(
            source_directories,
            write=args.write,
            remove_sources=args.remove_sources,
            force=args.force,
        )
    except (OSError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
