#!/usr/bin/env python3
"""Check that the tool registry parses and that every path in it exists.

``tools/agent/tools.json`` is the fallback every tool lookup depends on when
the shell's ``PATH`` is reduced. A malformed registry does not announce itself:
``json.loads`` raises, the caller swallows it, the lookup returns nothing, and
the gate reports "not installed" for a tool that is installed. That is a
misleading failure, and a misleading failure in the harness is worse than a
loud one anywhere else.

Run from gate G0, before anything that would depend on a lookup.

Exit code 0 when the registry parses and every path resolves, 1 otherwise.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

REGISTRY = Path(os.environ.get("CLAUDE_PROJECT_DIR", ".")) / "tools" / "agent" / "tools.json"

# Tools that are not installed yet at this point in the build. Their absence is
# expected; a malformed path for them is not, so they are still parsed and
# shape-checked, just not required to exist.
NOT_YET = {"cargo", "rustup"}


def main() -> int:
    if not REGISTRY.is_file():
        print(f"{REGISTRY} is missing", file=sys.stderr)
        return 1

    try:
        registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"{REGISTRY} is not valid JSON: {exc}", file=sys.stderr)
        print(
            "Backslashes must be escaped in JSON. Use forward slashes instead; "
            "Windows accepts them everywhere.",
            file=sys.stderr,
        )
        return 1

    problems = 0
    checked = 0
    for name, value in registry.items():
        if name.startswith("_"):
            continue
        if not isinstance(value, str):
            print(f"{name}: expected a path string, found {type(value).__name__}", file=sys.stderr)
            problems += 1
            continue
        checked += 1
        if Path(value).is_file():
            continue
        if name in NOT_YET:
            print(f"{name}: not installed yet ({value})")
            continue
        print(f"{name}: {value} does not exist", file=sys.stderr)
        problems += 1

    if problems:
        print(f"{problems} registry problem(s)", file=sys.stderr)
        return 1

    print(f"tool registry: {checked} entries, all resolvable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
