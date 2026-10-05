import re

from bonesinfra.services.languages.mise import MiseRuntime


class PythonRuntime(MiseRuntime):
    config_key = "python_version"
    default_version = "3.14.0"
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
    tool = "python"
    executable_name = "python"
    version_output_prefix = "Python "
    runtime_path_key = "python_runtime"


PYTHON = PythonRuntime()
