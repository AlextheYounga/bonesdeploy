#!/usr/bin/env python3
"""
BonesDeploy infra CLI entrypoint.

Usage:
    python main.py runtime list --json
    python main.py runtime questions <runtime> --json
    python main.py runtime defaults <runtime> --json
    python main.py setup apply --config <path>
    python main.py runtime apply --config <path> --runtime-config <path>
    python main.py ssl apply --config <path>
"""

import argparse
import json
import sys
import os

sys.path.insert(0, os.path.dirname(__file__))

from src.runtimes import list_runtimes, get_runtime


class UnimplementedError(RuntimeError):
    pass


def cmd_runtime_list(args):
    runtimes = list_runtimes()
    if args.json:
        print(json.dumps(runtimes))
    else:
        for r in runtimes:
            print(r)


def cmd_runtime_questions(args):
    try:
        module = get_runtime(args.runtime)
    except KeyError as err:
        print(str(err), file=sys.stderr)
        sys.exit(1)

    if not hasattr(module, "questions"):
        print(json.dumps([]))
        return

    result = module.questions()
    if args.json:
        print(json.dumps(result))
    else:
        for q in result:
            print(f"{q['key']}: {q['label']} ({q.get('type', 'string')})")


def cmd_runtime_defaults(args):
    try:
        module = get_runtime(args.runtime)
    except KeyError as err:
        print(str(err), file=sys.stderr)
        sys.exit(1)

    if not hasattr(module, "defaults"):
        print(json.dumps({}))
        return

    result = module.defaults()
    if args.json:
        print(json.dumps(result))
    else:
        for key, value in result.items():
            print(f"{key}: {json.dumps(value)}")


def cmd_setup_apply(args):
    raise UnimplementedError("setup apply is not implemented yet")


def cmd_runtime_apply(args):
    raise UnimplementedError("runtime apply is not implemented yet")


def cmd_ssl_apply(args):
    raise UnimplementedError("ssl apply is not implemented yet")


def main():
    parser = argparse.ArgumentParser(description="BonesDeploy infra CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    runtime_parser = subparsers.add_parser("runtime", help="Runtime operations")
    runtime_subparsers = runtime_parser.add_subparsers(dest="subcommand", required=True)

    list_parser = runtime_subparsers.add_parser("list", help="List available runtimes")
    list_parser.add_argument("--json", action="store_true", help="Output as JSON")
    list_parser.set_defaults(func=cmd_runtime_list)

    questions_parser = runtime_subparsers.add_parser("questions", help="Get runtime questions")
    questions_parser.add_argument("runtime", help="Runtime name")
    questions_parser.add_argument("--json", action="store_true", help="Output as JSON")
    questions_parser.set_defaults(func=cmd_runtime_questions)

    defaults_parser = runtime_subparsers.add_parser("defaults", help="Get runtime defaults")
    defaults_parser.add_argument("runtime", help="Runtime name")
    defaults_parser.add_argument("--json", action="store_true", help="Output as JSON")
    defaults_parser.set_defaults(func=cmd_runtime_defaults)

    setup_parser = subparsers.add_parser("setup", help="Setup operations")
    setup_subparsers = setup_parser.add_subparsers(dest="subcommand", required=True)
    setup_apply = setup_subparsers.add_parser("apply", help="Apply setup")
    setup_apply.add_argument("--config", required=True, help="Path to bones.yaml")
    setup_apply.set_defaults(func=cmd_setup_apply)

    runtime_apply_parser = subparsers.add_parser("runtime-apply", help="Runtime apply operations")
    runtime_apply_sub = runtime_apply_parser.add_subparsers(dest="subcommand", required=True)
    runtime_apply_cmd = runtime_apply_sub.add_parser("apply", help="Apply runtime configuration")
    runtime_apply_cmd.add_argument("--config", required=True, help="Path to bones.yaml")
    runtime_apply_cmd.add_argument("--runtime-config", required=True, help="Path to runtime.yaml")
    runtime_apply_cmd.set_defaults(func=cmd_runtime_apply)

    ssl_parser = subparsers.add_parser("ssl", help="SSL operations")
    ssl_subparsers = ssl_parser.add_subparsers(dest="subcommand", required=True)
    ssl_apply = ssl_subparsers.add_parser("apply", help="Apply SSL configuration")
    ssl_apply.add_argument("--config", required=True, help="Path to bones.yaml")
    ssl_apply.set_defaults(func=cmd_ssl_apply)

    args = parser.parse_args()
    try:
        args.func(args)
    except UnimplementedError as err:
        print(f"Error: {err}", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
