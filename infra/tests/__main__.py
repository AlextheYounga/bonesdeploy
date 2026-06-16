"""Test discovery runner — no external deps required."""

import importlib
import sys
import traceback
from pathlib import Path


def discover_tests():
    root = Path(__file__).parent
    for file in sorted(root.glob("test_*.py")):
        mod = file.stem
        print(f"\n=== {mod} ===")
        m = importlib.import_module(f"tests.{mod}")
        for name in sorted(dir(m)):
            if not name.startswith("test_"):
                continue
            fn = getattr(m, name)
            if not callable(fn):
                continue
            yield mod, name, fn


def main():
    passed = 0
    failed = 0

    for mod_name, test_name, fn in discover_tests():
        try:
            fn()
            print(f"  OK: {test_name}")
            passed += 1
        except Exception:
            print(f"  FAIL: {test_name}")
            for line in traceback.format_exc().splitlines()[-3:]:
                print(f"        {line}")
            failed += 1

    print(f"\n{passed} passed, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
