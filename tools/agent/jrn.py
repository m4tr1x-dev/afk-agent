#!/usr/bin/env python3
"""Append one event to the build journal.

Writing to the journal has to be cheap or it does not happen. One process, one
line, no read of what is already there.

    python tools/agent/jrn.py gate --task T-0043 --gate G2 --result fail \
        --note "clippy needless_range_loop probe.rs:88"

    python tools/agent/jrn.py attempt --task T-0043 \
        --tried "absolute SendInput" --why "no camera delta" --dont-retry \
        --next "relative only"

    python tools/agent/jrn.py measure --q Q3 --metric tactical_latency_ms \
        --value 312 --lands-in docs/spec/15-performance-budgets.md

Events go to ``journal.jsonl``. Two kinds are also written to their own file,
because they are read on their own and reading them out of the journal would
mean parsing the whole thing: ``attempt`` to ``attempts.jsonl`` and ``measure``
to ``measurements.jsonl``.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import sys
from pathlib import Path

KINDS = (
    "task_start",
    "task_done",
    "gate",
    "measure",
    "decision",
    "blocker",
    "attempt",
    "pr",
    "spec_change",
    "escalate",
)

# Kinds that also land in a file of their own, because something reads them
# without wanting the rest of the journal.
SIDECAR = {"attempt": "attempts.jsonl", "measure": "measurements.jsonl", "blocker": "blockers.jsonl"}


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=KINDS)
    parser.add_argument("--task")
    parser.add_argument("--note")
    parser.add_argument("--gate")
    parser.add_argument("--result")
    parser.add_argument("--pr", type=int)
    # attempt
    parser.add_argument("--tried")
    parser.add_argument("--why")
    parser.add_argument("--next", dest="next_step")
    parser.add_argument("--dont-retry", action="store_true")
    # measure
    parser.add_argument("--q", help="known-good-matrix question this answers")
    parser.add_argument("--metric")
    parser.add_argument("--value")
    parser.add_argument("--conditions", help="JSON object describing how it was measured")
    parser.add_argument("--evidence")
    parser.add_argument(
        "--lands-in",
        help="the page whose placeholder this measurement replaces; the gate checks it",
    )
    # blocker
    parser.add_argument("--id")
    parser.add_argument("--what")
    parser.add_argument("--owner-action")
    parser.add_argument("--status", default="open")

    args = parser.parse_args(argv[1:])

    row: dict[str, object] = {"t": now(), "kind": args.kind}
    for key, value in (
        ("task", args.task),
        ("note", args.note),
        ("gate", args.gate),
        ("result", args.result),
        ("pr", args.pr),
        ("tried", args.tried),
        ("why", args.why),
        ("next", args.next_step),
        ("q", args.q),
        ("metric", args.metric),
        ("value", args.value),
        ("evidence", args.evidence),
        ("lands_in", args.lands_in),
        ("id", args.id),
        ("what", args.what),
        ("owner_action", args.owner_action),
    ):
        if value is not None:
            row[key] = value

    if args.dont_retry:
        row["dont_retry"] = True
    if args.conditions:
        try:
            row["conditions"] = json.loads(args.conditions)
        except json.JSONDecodeError as exc:
            print(f"--conditions is not valid JSON: {exc}", file=sys.stderr)
            return 2
    if args.kind == "blocker":
        row["status"] = args.status

    if args.kind == "measure" and not args.lands_in:
        print(
            "a measurement without --lands-in is a number nobody will act on; "
            "name the page whose placeholder it replaces",
            file=sys.stderr,
        )
        return 2

    d = Path(os.environ.get("CLAUDE_PROJECT_DIR", ".")) / ".agent"
    if not d.is_dir():
        print(f"{d} does not exist; the build journal has not been set up", file=sys.stderr)
        return 2

    line = json.dumps(row, ensure_ascii=False) + "\n"
    with (d / "journal.jsonl").open("a", encoding="utf-8") as handle:
        handle.write(line)
    if args.kind in SIDECAR:
        with (d / SIDECAR[args.kind]).open("a", encoding="utf-8") as handle:
            handle.write(line)

    print(line, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
