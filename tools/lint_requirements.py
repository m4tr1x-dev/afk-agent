#!/usr/bin/env python3
"""Check the requirement identifier graph across the documentation.

Requirement identifiers are the join key between the specification, the
decision records, the source and the tests. They are what turns "did we build
what we specified?" into a query rather than a judgement call — but only if the
graph is consistent, which is what this script enforces.

Three failures are hard errors:

* An identifier is *referenced* (in a page's front matter, in a decision's
  ``affects`` list, or inline in prose) but never *defined* in a requirements
  table. This is almost always a typo, and a typo here silently removes a
  requirement from the traceability report.
* An identifier is defined twice. Numbers are assigned once and never reused;
  two definitions mean two different requirements now share a name, and every
  test annotation pointing at it is ambiguous.
* An identifier is malformed.

Requirements that are defined but not yet implemented are *not* an error. They
are the backlog, and making it visible for free is one of the reasons the
scheme exists.
"""

from __future__ import annotations

import os
import re
import sys
from collections import defaultdict
from pathlib import Path

# FR-ACT-001, NFR-PERF-002, INV-SAFE-001 carry an area segment.
# CON-001 does not: constraints are global rather than belonging to a subsystem.
ID_PATTERN = re.compile(r"\b(?:(?:FR|NFR|INV)-[A-Z]{3,5}-\d{3}|CON-\d{3})\b")

VALID_AREAS = {
    "PERC",
    "LOOP",
    "ACT",
    "GND",
    "SKILL",
    "CTX",
    "GUI",
    "CFG",
    "SAFE",
    "OBS",
    "MODEL",
}

# A definition is a table row whose first cell is an identifier in backticks:
#     | `FR-ACT-001` | The executor MUST ... |
DEFINITION = re.compile(
    r"^\|\s*`((?:(?:FR|NFR|INV)-[A-Z]{3,5}-\d{3}|CON-\d{3}))`\s*\|",
    re.MULTILINE,
)

FRONT_MATTER = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)


def annotate(message: str) -> str:
    return f"::error::{message}" if "GITHUB_ACTIONS" in os.environ else message


def main(argv: list[str]) -> int:
    root = Path(argv[1] if len(argv) > 1 else "docs")
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 2

    definitions: dict[str, list[Path]] = defaultdict(list)
    references: dict[str, set[Path]] = defaultdict(set)

    pages = sorted(root.rglob("*.md"))
    for page in pages:
        text = page.read_text(encoding="utf-8")

        for match in DEFINITION.finditer(text):
            definitions[match.group(1)].append(page)

        body = FRONT_MATTER.sub("", text)
        for rid in ID_PATTERN.findall(text):
            references[rid].add(page)
        # Definitions are also references to themselves; that is harmless.
        del body

    errors: list[str] = []

    for rid, pages_defining in sorted(definitions.items()):
        if len(pages_defining) > 1:
            where = ", ".join(str(p) for p in pages_defining)
            errors.append(
                f"{rid} is defined in more than one place ({where}). "
                "Identifiers are assigned once and never reused."
            )

    for rid, pages_referencing in sorted(references.items()):
        # Constraints are global and carry no area segment.
        area = rid.split("-")[1] if not rid.startswith("CON-") else None
        if area is not None and area not in VALID_AREAS:
            where = ", ".join(str(p) for p in sorted(pages_referencing))
            errors.append(
                f"{rid} uses unknown area '{area}' ({where}). "
                f"Valid areas: {', '.join(sorted(VALID_AREAS))}"
            )
        if rid not in definitions:
            where = ", ".join(str(p) for p in sorted(pages_referencing))
            errors.append(
                f"{rid} is referenced but never defined in a requirements "
                f"table ({where})"
            )

    for error in errors:
        print(annotate(error))

    defined = len(definitions)
    referenced = len(references)
    orphans = sorted(set(definitions) - set(references))
    print(
        f"{defined} requirements defined, {referenced} referenced, "
        f"{len(orphans)} defined but never referenced elsewhere",
        file=sys.stderr,
    )
    if orphans:
        # Not an error. A requirement nothing else points at yet is simply
        # waiting to be implemented, which is the normal state early on.
        print("  backlog: " + ", ".join(orphans[:20]), file=sys.stderr)

    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
