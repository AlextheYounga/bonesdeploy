import os
import importlib.util

import yaml


def load_yaml(path):
    with open(path, "r", encoding="utf-8") as file:
        return yaml.safe_load(file) or {}


def bones_dir(deploy_file):
    return os.path.abspath(os.path.join(os.path.dirname(deploy_file), ".."))


def load_bones_config(deploy_file):
    return load_yaml(os.path.join(bones_dir(deploy_file), "bones.yaml"))


def load_runtime_config(deploy_file):
    return load_yaml(os.path.join(bones_dir(deploy_file), "runtime.yaml"))


def unflatten(data_dict):
    result = {}
    for key, value in data_dict.items():
        parts = key.split(".")
        node = result
        for part in parts[:-1]:
            if part not in node:
                node[part] = {}
            node = node[part]
        node[parts[-1]] = value
    return result


def load_optional_module(module_path, module_name):
    if os.path.exists(module_path):
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        return mod
    return None
