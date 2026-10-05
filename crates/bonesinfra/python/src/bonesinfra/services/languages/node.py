import re

from bonesinfra.services.languages.mise import MiseRuntime


class NodeRuntime(MiseRuntime):
    config_key = "node_version"
    default_version = "24.19.0"
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
    tool = "node"
    executable_name = "node"
    version_output_prefix = "v"
    runtime_path_key = "node_runtime"


NODE = NodeRuntime()
