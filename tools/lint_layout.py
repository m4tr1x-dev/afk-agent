#!/usr/bin/env python3
"""Verify that every tracked top-level directory is listed in the layout page.

``docs/contributing/repository-layout.md`` promises this check. Until it
existed, the page was a description that drifted the moment someone added a
directory, which is the failure mode the page itself warns about.

The check runs in both directions, and the second direction is the one that
keeps the page honest over time:

* A tracked top-level directory absent from the table is an error. The page has
  become fiction.
* A row marked ``present`` whose directory does not exist is an error. The page
  describes something that was removed.
* A row marked ``planned`` whose directory does exist is an error. The
  directory arrived and nobody moved the row, so the page now understates what
  the repository contains.

Only tracked directories count. Build output, the documentation site, the Rust
target directory and the agent's own scratch space are untracked by design and
are not the page's business.

Exit code 0 when the page and the repository agree, 1 otherwise. Errors are
printed one per line in a form GitHub Actions renders as an annotation.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

PAGE = Path("docs/contributing/repository-layout.md")

# A table row whose first cell is a backticked directory path and whose second
# cell is the state. Anything else in the table is prose and is ignored.
ROW = re.compile(
    r"^\|\s*`(?P<path>[^`]+/)`\s*\|\s*(?P<state>present|planned)\s*\|",
    re.MULTILINE,
)

VALID_STATES = {"present", "planned"}


def annotate(message: str, *, file: str | None = None) -> None:
    """Print an error, as a GitHub annotation when running under Actions."""
    if os.environ.get("GITHUB_ACTIONS"):
        location = f" file={file}" if file else ""
        print(f"::error{location}::{message}")
    else:
        prefix = f"{file}: " if file else ""
        print(f"{prefix}{message}", file=sys.stderr)


def tracked_directories(root: Path) -> set[str]:
    """Top-level directories containing at least one tracked file."""
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        capture_output=True,
        check=True,
        text=True,
    )
    directories = set()
    for path in result.stdout.split("\0"):
        if "/" in path:
            directories.add(path.split("/", 1)[0] + "/")
    return directories


def listed_directories(page_text: str) -> dict[str, str]:
    """Map each directory named in the table to its declared state."""
    listed: dict[str, str] = {}
    for match in ROW.finditer(page_text):
        path = match.group("path")
        # Nested entries such as ``docs/spec/`` document structure inside a
        # top-level directory. They are welcome on the page and are not what
        # this check is about.
        if path.count("/") == 1:
            listed[path] = match.group("state")
    return listed


def main(argv: list[str]) -> int:
    root = Path(argv[1]) if len(argv) > 1 else Path(".")
    page = root / PAGE

    if not page.is_file():
        annotate(f"{PAGE} is missing; the layout check has nothing to read")
        return 2

    text = page.read_text(encoding="utf-8")
    listed = listed_directories(text)

    if not listed:
        annotate(
            "no directory rows found; the table must have rows shaped "
            "`| `name/` | present | description |`",
            file=str(PAGE),
        )
        return 1

    tracked = tracked_directories(root)
    errors = 0

    for directory in sorted(tracked - set(listed)):
        annotate(
            f"{directory} is tracked but does not appear in the layout table",
            file=str(PAGE),
        )
        errors += 1

    for directory, state in sorted(listed.items()):
        exists = (root / directory).is_dir()
        if state == "present" and directory not in tracked:
            annotate(
                f"{directory} is listed as present but contains no tracked files",
                file=str(PAGE),
            )
            errors += 1
        elif state == "planned" and exists and directory in tracked:
            annotate(
                f"{directory} is listed as planned but now exists; "
                "change its state to present",
                file=str(PAGE),
            )
            errors += 1

    if errors:
        print(f"{errors} layout error(s)", file=sys.stderr)
        return 1

    print(f"layout: {len(tracked)} tracked director(ies), all listed")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
