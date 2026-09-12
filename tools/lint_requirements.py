#!/usr/bin/env python3
"""Check the requirement identifier graph across documentation, source and tests.

Requirement identifiers are the join key between the specification, the
decision records, the source and the tests. They are what turns "did we build
what we specified?" into a query rather than a judgement call — but only if the
graph is consistent, which is what this script enforces.

    python tools/lint_requirements.py docs
    python tools/lint_requirements.py docs --source crates src tests experiments
    python tools/lint_requirements.py docs --source crates --emit docs/reference/traceability.md

Markers in source, all of them plain comments so that this stays a text scan.
It has to: the documentation jobs run on a hosted Linux runner that has neither
cargo nor msbuild, and the scan must work before the code compiles.

    // REQ: FR-AREA-NNN        an implementation site
    // COVERS: INV-AREA-NNN    inside a test function
    [Covers("FR-AREA-NNN")]    a C# test attribute

A manual procedure carries its identifiers in front matter, because
several requirements name ``Manual`` as their verification method and must
have a satisfiable path:

    covers: [FR-AREA-NNN, NFR-AREA-NNN]

Hard errors:

* An identifier is referenced and never defined in a requirements table. A typo
  here silently removes a requirement from the report.
* An identifier is defined twice. Numbers are assigned once; two definitions
  mean every annotation pointing at it is ambiguous.
* An identifier is malformed, or uses an area that does not exist.
* A ``REQ:`` marker names an identifier nothing defines.
* An identifier has an implementation site and **no covering test**. This is
  the rule that keeps the scheme honest, and it is the one the testing strategy
  promised and nothing enforced.

Not an error: an identifier defined with no implementation site. That is the
backlog, and making it visible for free is one of the reasons the scheme exists.

`experiments/` is deliberately **not** scanned by the checks that run in
continuous integration. Probes are timeboxed and deleted when their question is
resolved, and a requirement whose only covering test lives in one would lose
its coverage the day the probe is archived — silently, because deleting a
directory does not look like removing a test. Markers inside a probe are
welcome as a record of intent for whoever ports the code into a real crate;
they simply do not discharge the obligation.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

# FR-ACT-001, NFR-PERC-002, INV-SAFE-001 carry an area segment.
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

# The verification column of that same row, which decides what may satisfy the
# covering-test rule. Manual and review methods are satisfied by a procedure or
# a review log rather than by an automated test.
DEFINITION_ROW = re.compile(
    r"^\|\s*`((?:(?:FR|NFR|INV)-[A-Z]{3,5}-\d{3}|CON-\d{3}))`\s*\|(?P<rest>.*)$",
    re.MULTILINE,
)

FRONT_MATTER = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)

REQ_MARKER = re.compile(r"(?://|#)\s*REQ:\s*(.+)$")
COVERS_MARKER = re.compile(r"(?://|#)\s*COVERS:\s*(.+)$")
COVERS_ATTRIBUTE = re.compile(r"\[Covers\(([^)]*)\)\]")
COVERS_FRONT_MATTER = re.compile(r"^covers:\s*\[(.*?)\]", re.MULTILINE)

SOURCE_SUFFIXES = {".rs", ".cs", ".py", ".toml"}
SKIP_DIRS = {"target", "bin", "obj", "site", "node_modules", ".git"}

# Methods satisfied by something other than an automated test.
NON_AUTOMATED = {"manual", "review"}


def annotate(message: str) -> str:
    return f"::error::{message}" if "GITHUB_ACTIONS" in os.environ else message


class Graph:
    def __init__(self) -> None:
        self.definitions: dict[str, list[Path]] = defaultdict(list)
        self.verification: dict[str, str] = {}
        self.references: dict[str, set[Path]] = defaultdict(set)
        self.implemented: dict[str, list[str]] = defaultdict(list)
        self.covered: dict[str, list[str]] = defaultdict(list)


GENERATED = re.compile(r"^generated:\s*true\s*$", re.MULTILINE)


def scan_docs(root: Path, graph: Graph) -> None:
    for page in sorted(root.rglob("*.md")):
        text = page.read_text(encoding="utf-8")

        # A generated page never defines anything. The traceability page in
        # particular restates every identifier in a table shaped exactly like a
        # definition, and counting those would report all 101 as defined twice
        # — an error caused entirely by the report of the errors.
        front = FRONT_MATTER.match(text)
        if front and GENERATED.search(front.group(1)):
            for rid in ID_PATTERN.findall(text):
                graph.references[rid].add(page)
            continue

        for match in DEFINITION_ROW.finditer(text):
            rid = match.group(1)
            graph.definitions[rid].append(page)
            # Constraints are defined in scope-and-goals with an ID, a
            # statement and a source. They carry no verification method, and
            # reading the last cell as one turns their rationale into one.
            if rid.startswith("CON-"):
                graph.verification[rid] = ""
            else:
                cells = [c.strip() for c in match.group("rest").split("|")]
                graph.verification[rid] = next((c for c in reversed(cells) if c), "")

        # Front-matter identifiers count as references. The `requirements` and
        # `affects` arrays are how a page or a record joins the graph, so a
        # typo in one has to be caught here or it is caught nowhere.
        for rid in ID_PATTERN.findall(text):
            graph.references[rid].add(page)

        # A manual procedure covers what its front matter says it covers.
        if front:
            for match in COVERS_FRONT_MATTER.finditer(front.group(1)):
                for rid in ID_PATTERN.findall(match.group(1)):
                    graph.covered[rid].append(str(page).replace("\\", "/"))


def scan_source(root: Path, graph: Graph) -> int:
    """Collect REQ and COVERS markers. Returns the number of files scanned."""
    if not root.is_dir():
        return 0
    scanned = 0
    for path in sorted(root.rglob("*")):
        if path.suffix not in SOURCE_SUFFIXES:
            continue
        if SKIP_DIRS & set(path.parts):
            continue
        scanned += 1
        rel = str(path).replace("\\", "/")
        text = path.read_text(encoding="utf-8", errors="replace")
        for number, line in enumerate(text.splitlines(), start=1):
            where = f"{rel}:{number}"
            if match := REQ_MARKER.search(line):
                for rid in ID_PATTERN.findall(match.group(1)):
                    graph.implemented[rid].append(where)
                    graph.references[rid].add(path)
            if match := COVERS_MARKER.search(line):
                for rid in ID_PATTERN.findall(match.group(1)):
                    graph.covered[rid].append(where)
                    graph.references[rid].add(path)
            if match := COVERS_ATTRIBUTE.search(line):
                for rid in ID_PATTERN.findall(match.group(1)):
                    graph.covered[rid].append(where)
                    graph.references[rid].add(path)
    return scanned


def check(graph: Graph) -> list[str]:
    errors: list[str] = []

    for rid, pages in sorted(graph.definitions.items()):
        if len(pages) > 1:
            where = ", ".join(str(p) for p in pages)
            errors.append(
                f"{rid} is defined in more than one place ({where}). "
                "Identifiers are assigned once and never reused."
            )

    for rid, referencing in sorted(graph.references.items()):
        area = rid.split("-")[1] if not rid.startswith("CON-") else None
        if area is not None and area not in VALID_AREAS:
            where = ", ".join(str(p) for p in sorted(referencing))
            errors.append(
                f"{rid} uses unknown area '{area}' ({where}). "
                f"Valid areas: {', '.join(sorted(VALID_AREAS))}"
            )
        if rid not in graph.definitions:
            where = ", ".join(str(p) for p in sorted(referencing))
            errors.append(
                f"{rid} is referenced but never defined in a requirements table ({where})"
            )

    # The rule the testing strategy promised and nothing enforced: an
    # implementation with no covering test is a requirement nobody can show
    # still holds.
    for rid, sites in sorted(graph.implemented.items()):
        if graph.covered.get(rid):
            continue
        method = graph.verification.get(rid, "").lower()
        hint = ""
        if method and all(m in NON_AUTOMATED for m in re.split(r"[,/]", method) if m.strip()):
            hint = (
                f" Its verification method is '{graph.verification[rid]}', so a "
                "procedure under tests/manual/ with it in the `covers` front "
                "matter satisfies this."
            )
        errors.append(
            f"{rid} is implemented at {sites[0]} but no test covers it."
            f" Add a `// COVERS: {rid}` marker to the test that exercises it.{hint}"
        )

    return errors


def emit(graph: Graph, path: Path, link_base: Path) -> None:
    """Write the page to ``path``, computing links as if it lived at ``link_base``.

    The two differ when the freshness check regenerates into a temporary
    directory: links computed from there would differ from the committed
    ones and every page would read as stale.
    """
    rows = []
    for rid in sorted(graph.definitions):
        defined = graph.definitions[rid][0]
        defined_rel = str(defined).replace("\\", "/")
        sites = graph.implemented.get(rid, [])
        tests = graph.covered.get(rid, [])
        if not sites:
            state = "backlog"
        elif not tests:
            state = "implemented, uncovered"
        else:
            state = "implemented"
        rows.append(
            "| `{rid}` | {method} | [{defined}]({link}) | {sites} | {tests} | {state} |".format(
                rid=rid,
                method=graph.verification.get(rid, "") or "—",
                defined=defined_rel,
                link=os.path.relpath(defined_rel, link_base.parent.as_posix()).replace("\\", "/"),
                sites=("`" + "`, `".join(sites) + "`") if sites else "—",
                tests=("`" + "`, `".join(tests) + "`") if tests else "—",
                state=state,
            )
        )

    body = """---
title: Requirements traceability
description: "Every requirement identifier, where it is defined, where it is implemented, and what covers it."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [reference, traceability]
requirements: []
decisions: []
generated: true
generated_from: docs/spec/01-requirements.md, docs/spec/00-scope-and-goals.md, crates/, src/, tests/
---

# Requirements traceability

Generated by `tools/lint_requirements.py`. Do not edit by hand; continuous
integration regenerates it and fails when the committed copy differs.

`backlog` is not a defect. It is a requirement nothing implements yet, which is
the normal state of most of this table today.

`implemented, uncovered` never reaches the default branch: an implementation
site with no covering test fails the check that produces this page.

| ID | Verification | Defined in | Implemented at | Covered by | State |
| --- | --- | --- | --- | --- | --- |
"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body + "\n".join(rows) + "\n", encoding="utf-8")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Check the requirement identifier graph.")
    parser.add_argument("docs", nargs="?", default="docs")
    parser.add_argument("--source", nargs="*", default=[], help="source trees to scan for markers")
    parser.add_argument("--emit", help="write the traceability page to this path")
    parser.add_argument(
        "--emit-as",
        help="the path the page will be published at; links are computed from it. "
        "Defaults to --emit. The freshness check writes elsewhere and passes this.",
    )
    args = parser.parse_args(argv[1:])

    root = Path(args.docs)
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 2

    graph = Graph()
    scan_docs(root, graph)

    scanned = 0
    for tree in args.source:
        scanned += scan_source(Path(tree), graph)

    errors = check(graph)
    for error in errors:
        print(annotate(error))

    if args.emit:
        emit(graph, Path(args.emit), Path(args.emit_as or args.emit))

    defined = len(graph.definitions)
    implemented = len(graph.implemented)
    covered = len(graph.covered)
    orphans = sorted(set(graph.definitions) - set(graph.implemented))
    print(
        f"{defined} requirements defined, {implemented} implemented, "
        f"{covered} covered by a test; {scanned} source file(s) scanned",
        file=sys.stderr,
    )
    if orphans:
        # Not an error. A requirement nothing implements yet is the backlog.
        print(f"  backlog: {len(orphans)} not implemented", file=sys.stderr)

    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
