import re

from bonesinfra.services.languages.mise import MiseRuntime


class RubyRuntime(MiseRuntime):
    config_key = "ruby_version"
    default_version = "3.3.8"
    version_pattern = re.compile(r"^3\.[234]\.[0-9]+$")
    tool = "ruby"
    executable_name = "ruby"
    version_output_prefix = "ruby "
    runtime_path_key = "ruby_runtime"


RUBY = RubyRuntime()
