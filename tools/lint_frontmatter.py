#!/usr/bin/env python3
"""Validate the YAML front matter on every Markdown page under docs/.

The front matter is not decoration. Three things depend on it:

* ``status`` drives the banner that stops a reader mistaking a draft
  specification for behaviour that exists. In a project whose documentation
  precedes its code, that is the single most likely way for the docs to
  mislead, so a page without a status is a defect rather than an oversight.
* ``requirements`` and ``decisions`` form the edges of the traceability graph
  that ``lint_requirements.py`` walks.
* ``generated`` marks pages that CI regenerates and compares, so a hand edit to
  one is caught rather than silently overwritten later.

Exit code 0 when every page validates, 1 otherwise. Errors are printed one per
line in a form GitHub Actions renders as an annotation.
"""

from __future__ import annotations

import datetime as dt
import re
import sys
from pathlib import Path

import yaml

# Statuses valid on any page.
VALID_STATUS = {
    "draft",
    "review",
    "accepted",
    "implemented",
    "deprecated",
    "superseded",
}

# Decision records have their own lifecycle: a record can be proposed and then
# rejected, and the rejected one stays in the repository because the reasoning
# for not doing something is worth as much as the reasoning for doing it.
DECISION_STATUS = VALID_STATUS | {"proposed", "rejected"}

# Pages that live in an area but are not of that area's kind. An index is
# navigation; a template is a template. Neither should be held to the area's
# required fields.
NON_MEMBER_PAGES = {"index.md", "adr-template.md"}

# Required on every page, regardless of area.
BASE_FIELDS = {
    "title": str,
    "description": str,
    "status": str,
    "owner": str,
    "created": dt.date,
    "last_reviewed": dt.date,
    "applies_to": str,
    "tags": list,
    "requirements": list,
    "decisions": list,
    "generated": bool,
}

# Additional requirements, keyed by the directory under docs/.
AREA_FIELDS: dict[str, dict[str, type]] = {
    "decisions": {
        "date": dt.date,
        "deciders": list,
        "consulted": list,
        "informed": list,
        "affects": list,
        "spec": list,
    },
    "tutorials": {
        "last_verified": dt.date,
        "verified_against": str,
    },
}

FRONT_MATTER = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)
REQUIREMENT_ID = re.compile(r"\A(?:(?:FR|NFR|INV)-[A-Z]{3,5}-\d{3}|CON-\d{3})\Z")
DECISION_ID = re.compile(r"\AADR-\d{4}\Z")


def area_of(path: Path, root: Path) -> str:
    """Return the top-level area a page belongs to, or an empty string."""
    rel = path.relative_to(root)
    return rel.parts[0] if len(rel.parts) > 1 else ""


def check(path: Path, root: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    match = FRONT_MATTER.match(text)
    if not match:
        return [f"{path}:1 missing YAML front matter"]

    try:
        meta = yaml.safe_load(match.group(1))
    except yaml.YAMLError as exc:
        return [f"{path}:1 front matter is not valid YAML: {exc}"]

    if not isinstance(meta, dict):
        return [f"{path}:1 front matter must be a mapping"]

    errors: list[str] = []
    area = area_of(path, root)
    expected = dict(BASE_FIELDS)
    if path.name not in NON_MEMBER_PAGES:
        expected.update(AREA_FIELDS.get(area, {}))

    for field, kind in expected.items():
        if field not in meta:
            errors.append(f"{path}:1 missing required field '{field}'")
            continue
        value = meta[field]
        if value is None and field in {"tags", "requirements", "decisions"}:
            errors.append(f"{path}:1 field '{field}' must be a list, not null")
        elif not isinstance(value, kind):
            errors.append(
                f"{path}:1 field '{field}' should be {kind.__name__}, "
                f"got {type(value).__name__}"
            )

    status = meta.get("status")
    allowed = DECISION_STATUS if area == "decisions" else VALID_STATUS
    if isinstance(status, str) and status not in allowed:
        errors.append(
            f"{path}:1 status '{status}' is not one of {sorted(allowed)}"
        )

    if status == "superseded" and not meta.get("superseded_by"):
        errors.append(
            f"{path}:1 status is 'superseded' but 'superseded_by' is not set; "
            "a superseded page must say what replaced it"
        )

    if meta.get("generated") is True and not meta.get("generated_from"):
        errors.append(
            f"{path}:1 page is marked generated but 'generated_from' is missing"
        )

    # The H1 and the title must agree, because the title is what appears in
    # navigation and search while the H1 is what the reader sees on the page.
    body = text[match.end():]
    heading = next(
        (line for line in body.splitlines() if line.startswith("# ")), None
    )
    if heading is None:
        errors.append(f"{path} has no level-one heading")
    elif isinstance(meta.get("title"), str):
        if heading[2:].strip() != meta["title"].strip():
            errors.append(
                f"{path} heading '{heading[2:].strip()}' does not match "
                f"front-matter title '{meta['title']}'"
            )

    for rid in meta.get("requirements") or []:
        if not REQUIREMENT_ID.match(str(rid)):
            errors.append(f"{path}:1 malformed requirement identifier '{rid}'")

    for did in meta.get("decisions") or []:
        if not DECISION_ID.match(str(did)):
            errors.append(f"{path}:1 malformed decision identifier '{did}'")

    # A specification page with no requirements and no governing decisions is
    # prose pretending to be normative.
    #
    # Two pages are exempt. The index is navigation. The requirements page
    # *defines* the identifiers rather than citing them, and listing all of
    # them in its own front matter would be noise that has to be maintained by
    # hand for no reader's benefit.
    if area == "spec" and path.name not in {"index.md", "01-requirements.md"}:
        if status in {"accepted", "implemented"}:
            if not meta.get("requirements"):
                errors.append(
                    f"{path}:1 accepted specification page defines no requirements"
                )
            if not meta.get("decisions"):
                errors.append(
                    f"{path}:1 accepted specification page cites no decisions"
                )

    created, reviewed = meta.get("created"), meta.get("last_reviewed")
    if isinstance(created, dt.date) and isinstance(reviewed, dt.date):
        if reviewed < created:
            errors.append(f"{path}:1 last_reviewed is earlier than created")

    return errors


def main(argv: list[str]) -> int:
    root = Path(argv[1] if len(argv) > 1 else "docs")
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 2

    errors: list[str] = []
    pages = sorted(root.rglob("*.md"))
    for page in pages:
        errors.extend(check(page, root))

    for error in errors:
        print(f"::error::{error}" if "GITHUB_ACTIONS" in __import__("os").environ else error)

    print(f"checked {len(pages)} pages, {len(errors)} problems", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
