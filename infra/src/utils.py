import os
import tomllib


def load_toml(path):
    with open(path, "rb") as file:
        return tomllib.load(file)


def bones_dir(deploy_file):
    return os.path.abspath(os.path.join(os.path.dirname(deploy_file), ".."))


def load_bones_config(deploy_file):
    return load_toml(os.path.join(bones_dir(deploy_file), "bones.toml"))


def load_runtime_config(deploy_file):
    return load_toml(os.path.join(bones_dir(deploy_file), "runtime.toml"))


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
