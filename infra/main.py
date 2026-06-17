#!/usr/bin/env python3
"""
BonesDeploy infra CLI entrypoint.

Usage:
    python main.py runtime list --json
    python main.py runtime questions <runtime> --json
    python main.py runtime defaults <runtime> --json
    python main.py runtime apply --config .bones/bones.toml --runtime-config .bones/runtime.toml --ssh-user root
    python main.py setup apply --config .bones/bones.toml  < data.json
    python main.py ssl apply --config .bones/bones.toml  < data.json
"""

import argparse
import json
import sys
import os

sys.path.insert(0, os.path.dirname(__file__))

from src.runtimes import list_runtimes, get_runtime


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


def _build_flat_data_from_stdin():
    data = json.load(sys.stdin)
    return data


def cmd_setup_apply(args):
    stdin_data = _build_flat_data_from_stdin()
    hostname = stdin_data.pop("host", "")
    ssh_user = stdin_data.pop("ssh_user", "root")
    ssh_port = int(stdin_data.pop("ssh_port", 22))

    if not hostname:
        print("Error: missing host in deploy data", file=sys.stderr)
        sys.exit(3)

    from src.setup import deploy
    from src.pyinfra_runner import run as run_deploy

    run_deploy(
        hostname=hostname,
        ssh_user=ssh_user,
        ssh_port=ssh_port,
        data=stdin_data,
        deploy=deploy,
    )


def cmd_runtime_apply(args):
    from src.utils import load_toml, unflatten

    bones_cfg = load_toml(args.config)
    data = bones_cfg.get("data", {})
    project_name = data.get("project_name", "")
    repo_path = data.get("repo_path", "")
    project_root = data.get("project_root", "")
    web_root = data.get("web_root", "public")
    host = data.get("host", "")
    port = int(data.get("port", 22))
    ssh_user = args.ssh_user
    runtime_cfg = {}

    if args.runtime_config:
        from pathlib import Path
        rpath = Path(args.runtime_config)
        if rpath.exists():
            runtime_cfg = load_toml(str(rpath))

    from src.paths import DeploymentPaths
    paths = DeploymentPaths.new(project_name, repo_path, project_root, web_root)

    flat_data = {}
    flat_data["project_name"] = project_name
    flat_data["project_root"] = project_root
    flat_data["web_root"] = web_root
    flat_data["repo_path"] = repo_path
    flat_data["deploy_user"] = data.get("deploy_user", "git")
    flat_data["runtime_user"] = data.get("runtime_user", "www-data")
    flat_data["runtime_group"] = data.get("runtime_group", "www-data")
    flat_data["release_group"] = data.get("release_group", "deployers")
    flat_data["project_root_parent"] = paths.project_root_parent
    flat_data["ssh_port"] = str(port)
    flat_data["paths"] = paths.__dict__

    for key, value in runtime_cfg.items():
        if key not in flat_data:
            flat_data[key] = value

    from src.runtime import deploy
    from src.pyinfra_runner import run as run_deploy

    run_deploy(
        hostname=host,
        ssh_user=ssh_user,
        ssh_port=port,
        data=flat_data,
        deploy=deploy,
    )


def cmd_ssl_apply(args):
    stdin_data = _build_flat_data_from_stdin()
    hostname = stdin_data.pop("host", "")
    ssh_user = stdin_data.pop("ssh_user", "root")
    ssh_port = int(stdin_data.pop("ssh_port", 22))

    if not hostname:
        print("Error: missing host in deploy data", file=sys.stderr)
        sys.exit(3)

    from src.ssl import deploy
    from src.pyinfra_runner import run as run_deploy

    run_deploy(
        hostname=hostname,
        ssh_user=ssh_user,
        ssh_port=ssh_port,
        data=stdin_data,
        deploy=deploy,
    )


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

    runtime_apply_cmd = runtime_subparsers.add_parser("apply", help="Apply runtime configuration")
    runtime_apply_cmd.add_argument("--config", required=True, help="Path to bones.toml")
    runtime_apply_cmd.add_argument("--runtime-config", required=True, help="Path to runtime.toml")
    runtime_apply_cmd.add_argument("--ssh-user", required=True, help="SSH user for remote connection")
    runtime_apply_cmd.set_defaults(func=cmd_runtime_apply)

    setup_parser = subparsers.add_parser("setup", help="Setup operations")
    setup_apply = setup_parser.add_subparsers(dest="subcommand", required=True).add_parser("apply", help="Apply setup")
    setup_apply.add_argument("--config", required=True, help="Path to bones.toml")
    setup_apply.set_defaults(func=cmd_setup_apply)

    ssl_parser = subparsers.add_parser("ssl", help="SSL operations")
    ssl_apply = ssl_parser.add_subparsers(dest="subcommand", required=True).add_parser("apply", help="Apply SSL configuration")
    ssl_apply.add_argument("--config", required=True, help="Path to bones.toml")
    ssl_apply.set_defaults(func=cmd_ssl_apply)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
