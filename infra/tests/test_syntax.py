"""All .py files under infra/ must parse without syntax errors."""

from . import helpers


def test_all_source_files_parse():
    for root in (helpers.INFRA_DIR, helpers.SRC_DIR):
        for file in sorted(root.rglob("*.py")):
            helpers.compile_module(file)
