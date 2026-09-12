#!/usr/bin/env python3
"""Print the build state, for injection into a fresh or compacted context.

A multi-week build is summarised many times. Anything the agent does not write
down is gone, and relying on it to remember to re-read a file is the same class
of error as relying on it to remember an invariant. So this runs from a
``SessionStart`` hook instead, on startup, resume and compaction.

What it prints is bounded on purpose. The journal is append-only and grows
without limit; only its tail is ever read. The one file printed whole is
``state.json``, which is rewritten rather than appended and stays small.

Output goes to stdout and the exit code is 0, which is what causes the harness
to place it in the new context. A failure here must not stop the session, so
every read is defensive: a missing or malformed file degrades to a line saying
so rather than to a traceback.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

# Roughly 3000 tokens. Large enough for the state plus a useful tail, small
# enough that it never competes with the work for context.
BUDGET = 12_000

JOURNAL_TAIL = 25
ATTEMPTS_TAIL = 15


# The Windows console defaults to a code page that cannot render the dashes
# used below, and a UnicodeEncodeError here would lose the whole briefing.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def state_dir() -> Path:
    root = Path(os.environ.get("CLAUDE_PROJECT_DIR", "."))
    return root / ".agent"


def read_lines(path: Path, limit: int) -> list[dict]:
    """Last ``limit`` well-formed JSON objects from a JSONL file."""
    if not path.is_file():
        return []
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows[-limit:] if limit else rows


def git_summary(root: Path) -> list[str]:
    def run(*args: str) -> str:
        try:
            out = subprocess.run(
                ["git", *args],
                cwd=root,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=10,
            )
            return out.stdout.strip() if out.returncode == 0 else ""
        except (OSError, subprocess.SubprocessError):
            return ""

    lines = []
    branch = run("rev-parse", "--abbrev-ref", "HEAD")
    if branch:
        dirty = run("status", "--porcelain")
        state = f"{len(dirty.splitlines())} uncommitted file(s)" if dirty else "clean"
        lines.append(f"branch: {branch}  ({state})")
    log = run("log", "--oneline", "-5")
    if log:
        lines.extend(f"  {line}" for line in log.splitlines())
    return lines


def main() -> int:
    root = Path(os.environ.get("CLAUDE_PROJECT_DIR", "."))
    d = state_dir()
    out: list[str] = []

    out.append("=== AFK-AGENT BUILD STATE — authoritative; re-read after every compaction ===")

    state_path = d / "state.json"
    if state_path.is_file():
        try:
            out.append(json.dumps(json.loads(state_path.read_text(encoding="utf-8")), indent=1))
        except (OSError, json.JSONDecodeError) as exc:
            out.append(f"!! {state_path} is unreadable: {exc}")
            out.append("!! Repair it before continuing. It is the only durable record of where the build is.")
    else:
        out.append("!! .agent/state.json is missing. The build journal has not been set up.")
        out.append("!! See docs/contributing/autonomous-builds.md")

    attempts = [r for r in read_lines(d / "attempts.jsonl", 0) if r.get("dont_retry")]
    if attempts:
        out.append("")
        out.append("--- ALREADY TRIED AND FAILED — do not repeat ---")
        for row in attempts[-ATTEMPTS_TAIL:]:
            out.append(
                f"  [{row.get('task', '?')}] {row.get('tried', '?')}"
                f"\n      why: {row.get('why', '?')}"
                + (f"\n      next: {row['next']}" if row.get("next") else "")
            )

    blockers = [r for r in read_lines(d / "blockers.jsonl", 0) if r.get("status") != "closed"]
    if blockers:
        out.append("")
        out.append("--- OPEN BLOCKERS ---")
        for row in blockers:
            out.append(f"  {row.get('id', '?')}: {row.get('what', '?')}")
            if row.get("owner_action"):
                out.append(f"      owner: {row['owner_action']}")

    journal = read_lines(d / "journal.jsonl", JOURNAL_TAIL)
    if journal:
        out.append("")
        out.append(f"--- LAST {len(journal)} JOURNAL EVENTS ---")
        for row in journal:
            out.append(
                f"  {row.get('t', '?')} {row.get('kind', '?')}"
                f" {row.get('task', '')} {row.get('note', row.get('result', ''))}".rstrip()
            )

    git = git_summary(root)
    if git:
        out.append("")
        out.append("--- GIT ---")
        out.extend(git)

    text = "\n".join(out)
    if len(text) > BUDGET:
        text = text[:BUDGET] + "\n… truncated to the context budget; read .agent/ directly for more."
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
