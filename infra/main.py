#!/usr/bin/env python3
"""
BonesDeploy infra CLI entrypoint.

Usage:
    python main.py runtime list --json
    python main.py runtime questions <runtime> --json
    python main.py runtime defaults <runtime> --json
"""

import argparse
import json
import sys
import os

sys.path.insert(0, os.path.dirname(__file__))

from src.runtimes import list_runtimes, get_runtime


class UnimplementedError(RuntimeError):
    pass


def _load_runtime(name):
    try:
        return get_runtime(name)
    except KeyError as err:
        print(str(err), file=sys.stderr)
        sys.exit(1)


def _output(data, as_json):
    if as_json:
        print(json.dumps(data))
    elif isinstance(data, list):
        for item in data:
            print(item)
    elif isinstance(data, dict):
        for key, value in data.items():
            print(f"{key}: {json.dumps(value)}")


def cmd_runtime_list(args):
    runtimes = list_runtimes()
    _output(runtimes, args.json)


def cmd_runtime_questions(args):
    module = _load_runtime(args.runtime)
    questions = module.questions() if hasattr(module, "questions") else []
    _output(questions, args.json)


def cmd_runtime_defaults(args):
    module = _load_runtime(args.runtime)
    defaults = module.defaults() if hasattr(module, "defaults") else {}
    _output(defaults, args.json)


def cmd_setup_apply(args):
    raise UnimplementedError("setup apply is not implemented yet")


def cmd_runtime_apply(args):
    raise UnimplementedError("runtime apply is not implemented yet")


def cmd_ssl_apply(args):
    raise UnimplementedError("ssl apply is not implemented yet")


def _add_json_flag(parser):
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    return parser


def main():
    parser = argparse.ArgumentParser(description="BonesDeploy infra CLI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    runtime_parser = subparsers.add_parser("runtime", help="Runtime operations")
    runtime_subparsers = runtime_parser.add_subparsers(dest="subcommand", required=True)

    _add_json_flag(runtime_subparsers.add_parser("list", help="List available runtimes")).set_defaults(func=cmd_runtime_list)

    questions_parser = _add_json_flag(runtime_subparsers.add_parser("questions", help="Get runtime questions"))
    questions_parser.add_argument("runtime", help="Runtime name")
    questions_parser.set_defaults(func=cmd_runtime_questions)

    defaults_parser = _add_json_flag(runtime_subparsers.add_parser("defaults", help="Get runtime defaults"))
    defaults_parser.add_argument("runtime", help="Runtime name")
    defaults_parser.set_defaults(func=cmd_runtime_defaults)

    setup_parser = subparsers.add_parser("setup", help="Setup operations")
    setup_apply = setup_parser.add_subparsers(dest="subcommand", required=True).add_parser("apply", help="Apply setup")
    setup_apply.add_argument("--config", required=True, help="Path to bones.toml")
    setup_apply.set_defaults(func=cmd_setup_apply)

    runtime_apply_parser = subparsers.add_parser("runtime-apply", help="Runtime apply operations")
    runtime_apply_cmd = runtime_apply_parser.add_subparsers(dest="subcommand", required=True).add_parser("apply", help="Apply runtime configuration")
    runtime_apply_cmd.add_argument("--config", required=True, help="Path to bones.toml")
    runtime_apply_cmd.add_argument("--runtime-config", required=True, help="Path to runtime.toml")
    runtime_apply_cmd.set_defaults(func=cmd_runtime_apply)

    ssl_parser = subparsers.add_parser("ssl", help="SSL operations")
    ssl_apply = ssl_parser.add_subparsers(dest="subcommand", required=True).add_parser("apply", help="Apply SSL configuration")
    ssl_apply.add_argument("--config", required=True, help="Path to bones.toml")
    ssl_apply.set_defaults(func=cmd_ssl_apply)

    args = parser.parse_args()
    try:
        args.func(args)
    except UnimplementedError as err:
        print(f"Error: {err}", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
