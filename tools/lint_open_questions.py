#!/usr/bin/env python3
"""Check that every open question carries a disposition, and that an accepted
page carries none that should have stopped it.

``CLAUDE.md`` has always said not to mark a page ``accepted`` while its Open
questions section still contains something that blocks implementation. That rule
lived in prose, which for a repository built largely by an agent is close to not
existing: the agent proposing the transition is the agent that would have to
hold itself to it.

Three rules, and the third is the one that makes the other two honest.

1. Every numbered item under ``## Open questions`` starts with one of
   ``**Blocking.**``, ``**Non-blocking.**`` or ``**Limitation.**``.
2. A page whose status is ``accepted`` or ``implemented`` carries no question
   marked **Blocking**. That is the rule from ``CLAUDE.md``, mechanised.
3. The same page carries no question marked **Limitation** either. A permanent
   limitation is not an open question; it belongs in the Known limitations
   section. Without this rule and that section, every limitation has to keep
   impersonating a question forever, and no page ever passes the gate - which
   is how a lifecycle quietly stops being used.

Exit code 0 when every page is consistent, 1 otherwise.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

import yaml

MARKERS = ("Blocking", "Non-blocking", "Limitation")

# A page that has reached one of these states is making a claim about its own
# readiness, and these rules are what that claim means.
SETTLED = {"accepted", "implemented"}

FRONT_MATTER = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)
SECTION = re.compile(r"^## Open questions\s*\n(.*?)(?=^## |\Z)", re.S | re.M)
ITEM = re.compile(r"^(\d+)\. (.*)$", re.M)
MARKED = re.compile(r"^\*\*(" + "|".join(MARKERS) + r")\.\*\* ")

# The index is navigation and the archive is history; neither is a live page.
SKIP = {"index.md"}


def annotate(message: str) -> None:
    print(f"::error::{message}" if "GITHUB_ACTIONS" in os.environ else message)


def check(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")

    front = FRONT_MATTER.match(text)
    status = ""
    if front:
        try:
            meta = yaml.safe_load(front.group(1)) or {}
            status = str(meta.get("status", ""))
        except yaml.YAMLError:
            # lint_frontmatter.py owns that error; reporting it twice helps
            # nobody and makes the second report look like a different defect.
            return []

    section = SECTION.search(text)
    if not section:
        return [f"{path}: no Open questions section"]

    errors: list[str] = []
    for number, body in ITEM.findall(section.group(1)):
        marker = MARKED.match(body)
        if not marker:
            errors.append(
                f"{path}: open question {number} carries no disposition; "
                f"start it with one of {', '.join(f'**{m}.**' for m in MARKERS)}"
            )
            continue

        if status in SETTLED and marker.group(1) == "Blocking":
            errors.append(
                f"{path}: status is '{status}' but open question {number} is "
                "marked Blocking; answer it or the page is not accepted"
            )
        if status in SETTLED and marker.group(1) == "Limitation":
            errors.append(
                f"{path}: status is '{status}' but open question {number} is "
                "marked Limitation; move it into the Known limitations section, "
                "which is where a permanent limitation lives"
            )

    return errors


def main(argv: list[str]) -> int:
    root = Path(argv[1] if len(argv) > 1 else "docs/spec")
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 2

    pages = [
        page
        for page in sorted(root.glob("*.md"))
        if page.name not in SKIP
    ]

    errors: list[str] = []
    counts = dict.fromkeys(MARKERS, 0)
    for page in pages:
        errors.extend(check(page))
        section = SECTION.search(page.read_text(encoding="utf-8"))
        if section:
            for _, body in ITEM.findall(section.group(1)):
                marker = MARKED.match(body)
                if marker:
                    counts[marker.group(1)] += 1

    for error in errors:
        annotate(error)

    total = sum(counts.values())
    summary = ", ".join(f"{counts[m]} {m.lower()}" for m in MARKERS)
    print(
        f"checked {len(pages)} pages, {total} open question(s): {summary}; "
        f"{len(errors)} problem(s)",
        file=sys.stderr,
    )
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
