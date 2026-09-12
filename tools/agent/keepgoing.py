#!/usr/bin/env python3
"""Stop hook: decline to stop mid-build, with four ways out and a runaway guard.

The maintainer is away and has pre-approved the run. The failure this prevents
is a session that finishes a task, reports it neatly, and stops — leaving weeks
of work undone because nobody was there to say "carry on".

This is the most powerful piece of the harness and the easiest to get wrong, so
it has four independent escape hatches and a guard against looping:

``.agent/HALT``               the maintainer's off switch. Create it and the
                              session stops at its next opportunity.
``.agent/DONE-M8``            the build is finished.
``.agent/BLOCKED-ON-HUMAN``   something needs a person and no further progress
                              is possible without them.
``stop_hook_active``          the harness is already retrying; never recurse.

The runaway guard is the fifth. If the journal has not grown across five
consecutive blocked stops, the session is circling rather than working, and
circling for free is worse than stopping.

Exit 0 with no output allows the stop. Printing a ``block`` decision refuses it.
Any failure in here allows the stop, because a harness defect must not be able
to trap a session.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

SENTINELS = ("HALT", "DONE-M8", "BLOCKED-ON-HUMAN")
RUNAWAY_LIMIT = 5


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        payload = {}

    # The harness is already looping on this hook. Never recurse.
    if payload.get("stop_hook_active"):
        return 0

    root = Path(os.environ.get("CLAUDE_PROJECT_DIR", "."))
    agent = root / ".agent"

    if not agent.is_dir():
        # No journal, no build to continue.
        return 0

    for sentinel in SENTINELS:
        if (agent / sentinel).exists():
            return 0

    # Runaway guard: a journal that has not grown across several blocked stops
    # means the session is not making progress, and blocking again would burn
    # the budget rather than the backlog.
    journal = agent / "journal.jsonl"
    size = journal.stat().st_size if journal.is_file() else 0
    counter = agent / ".stopcount"
    try:
        previous_count, previous_size = counter.read_text(encoding="utf-8").split()
        count = int(previous_count) + 1 if int(previous_size) == size else 0
    except (OSError, ValueError):
        count = 0
    try:
        counter.write_text(f"{count} {size}", encoding="utf-8")
    except OSError:
        return 0
    if count >= RUNAWAY_LIMIT:
        print(
            "The journal has not grown across "
            f"{RUNAWAY_LIMIT} blocked stops. Allowing the stop.",
            file=sys.stderr,
        )
        return 0

    try:
        state = json.loads((agent / "state.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        # A malformed state file is itself worth stopping for: the session
        # cannot know where it is.
        return 0

    task = state.get("task") or {}
    milestone = state.get("milestone", "?")
    next_tasks = state.get("next") or []

    reason = (
        f"Milestone {milestone} is not complete. "
        f"Current task {task.get('id', '?')}: {task.get('title', 'unknown')}.\n"
        "Re-read .agent/state.json, run tools/agent/gate.py for the files you "
        "touched, and continue.\n"
    )
    if next_tasks:
        reason += "Next: " + "; ".join(str(t) for t in next_tasks[:3]) + "\n"
    reason += (
        "Do not ask whether to continue — the maintainer is away and has "
        "pre-approved the run.\n"
        "To stop for a reason a person must resolve, create "
        ".agent/BLOCKED-ON-HUMAN with that reason."
    )

    print(json.dumps({"decision": "block", "reason": reason}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
