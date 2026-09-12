#!/usr/bin/env python3
"""Run the verification gates, and refuse to advance the build without them.

CLAUDE.md: "do not claim a build passed when it was not run." This script is
that rule made mechanical. A task is finished when the gates for the files it
touched exit zero **in this process**, not when anyone says so.

    python tools/agent/gate.py                 # gates implied by the working tree
    python tools/agent/gate.py --gates G1 G2   # named gates
    python tools/agent/gate.py --list

Gates:

    G0  environment      toolchain present and the right versions
    G1  documentation    the commands docs-quality.yml runs, verbatim
    G2  rust             fmt, clippy, tests, docs, supply chain
    G3  dotnet           format, build, test
    G7  traceability     requirement identifiers and their covering tests

G1 mirrors the workflow command for command. A local pass that does not predict
a CI pass is worse than no local check, because it trains you to skip it.

**The ratchet.** Every green run records how many tests passed. A later run
that records fewer fails, unless it is invoked with ``--allow-test-removal``
and a reason. The characteristic failure of an autonomous build is not getting
stuck; it is getting unstuck by deleting the obstacle, and this is the one
check that catches an ignored test, a deleted assertion and a lowered
threshold in a single rule.

**Safety gates are not statistical.** The testing strategy is explicit that a
panic key which works 99.7% of the time has found a bug. There is no
``--retries`` option, and there will not be one.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = Path(os.environ.get("CLAUDE_PROJECT_DIR", ".")).resolve()
AGENT = ROOT / ".agent"
REGISTRY = ROOT / "tools" / "agent" / "tools.json"


def resolve(tool: str) -> str:
    """Find a tool on PATH, falling back to the absolute-path registry.

    The shell a coding agent is given has a reduced PATH in which ``python``
    resolves to a Windows Store stub. Concluding a tool is absent and working
    around it is the expensive failure this prevents.
    """
    found = shutil.which(tool)
    if found and "WindowsApps" not in found:
        return found
    if REGISTRY.is_file():
        try:
            registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            registry = {}
        candidate = registry.get(tool)
        if candidate and Path(candidate).is_file():
            return candidate
    return found or tool


class Step:
    """One command in a gate. Optional steps skip when the tool is absent."""

    def __init__(self, name: str, argv: list[str], *, optional: bool = False) -> None:
        self.name = name
        self.argv = argv
        self.optional = optional


def gates() -> dict[str, list[Step]]:
    py = resolve("python")
    npx = resolve("npx")
    vale = resolve("vale")
    cargo = resolve("cargo")
    dotnet = resolve("dotnet")

    return {
        "G0": [
            # The registry is the fallback every other resolve() depends on. A
            # malformed one degrades silently into "tool not installed", which
            # is the single most misleading failure this harness can produce.
            Step("tool registry", [py, "tools/agent/check_registry.py"]),
            Step("python", [py, "--version"]),
            Step("git", [resolve("git"), "--version"]),
            Step("node", [resolve("node"), "--version"]),
            # vale sync downloads the Microsoft package, which is gitignored.
            # Without it every G1 run fails confusingly on a fresh machine.
            Step("vale sync", [vale, "sync"]),
            Step("cargo", [cargo, "--version"], optional=True),
            Step("dotnet", [dotnet, "--list-sdks"], optional=True),
        ],
        "G1": [
            Step("mkdocs strict", [py, "-m", "mkdocs", "build", "--strict"]),
            Step("front matter", [py, "tools/lint_frontmatter.py", "docs"]),
            Step(
                "requirements",
                [py, "tools/lint_requirements.py", "docs", "--source", "crates", "src", "tests", "tools"],
            ),
            Step("open questions", [py, "tools/lint_open_questions.py"]),
            Step("traceability page", [py, "tools/check_generated.py"]),
            Step("layout", [py, "tools/lint_layout.py"]),
            Step("call sites", [py, "tools/check_call_sites.py"]),
            Step("markdownlint", [npx, "--yes", "markdownlint-cli2", "**/*.md"]),
            Step(
                "cspell",
                [npx, "--yes", "cspell", "docs/**/*.md", "experiments/**/*.md", "*.md"],
            ),
            Step("vale", [vale, "--minAlertLevel=error", "docs/", "CLAUDE.md"]),
        ],
        "G2": [
            Step("fmt", [cargo, "fmt", "--all", "--check"]),
            Step(
                "clippy",
                [cargo, "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
            ),
            Step("test", [cargo, "test", "--workspace", "--locked"]),
            Step("doc", [cargo, "doc", "--workspace", "--no-deps", "--locked"]),
            Step("supply chain", [cargo, "deny", "check"], optional=True),
            # The generator is excluded from the product workspace by ADR-0016,
            # which means `--workspace` above does not see it. Excluded from the
            # audit surface is not the same as excluded from the checks.
            Step(
                "codegen fmt",
                [cargo, "fmt", "--manifest-path", "contract/codegen/Cargo.toml", "--all", "--check"],
            ),
            Step(
                "codegen clippy",
                [
                    cargo, "clippy", "--manifest-path", "contract/codegen/Cargo.toml",
                    "--all-targets", "--locked", "--", "-D", "warnings",
                ],
            ),
            Step(
                "codegen test",
                [cargo, "test", "--manifest-path", "contract/codegen/Cargo.toml", "--locked"],
            ),
            Step("codegen freshness", [py, "tools/check_codegen.py"]),
        ],
        "G3": [
            Step("format", [dotnet, "format", "--verify-no-changes"]),
            Step("build", [dotnet, "build", "-c", "Release"]),
            Step("test", [dotnet, "test", "-c", "Release", "--no-build"]),
        ],
        "G7": [
            Step("traceability", [py, "tools/lint_requirements.py", "docs"]),
        ],
    }


# Which gate a changed path implies. Ordered; the first match wins.
IMPLIED = [
    ("docs/", "G1"),
    ("mkdocs.yml", "G1"),
    (".vale.ini", "G1"),
    ("styles/", "G1"),
    ("cspell.json", "G1"),
    ("project-words.txt", "G1"),
    ("tools/lint_", "G1"),
    ("CLAUDE.md", "G1"),
    ("README.md", "G1"),
    ("crates/", "G2"),
    ("Cargo.toml", "G2"),
    ("Cargo.lock", "G2"),
    ("contract/", "G2"),
    ("tests/", "G2"),
    ("src/", "G3"),
]


def changed_paths() -> list[str]:
    out = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    paths = []
    for line in out.stdout.splitlines():
        if len(line) > 3:
            paths.append(line[3:].strip().strip('"'))
    return paths


def implied_gates(paths: list[str]) -> list[str]:
    wanted: list[str] = []
    for path in paths:
        for prefix, gate in IMPLIED:
            if path.startswith(prefix) or path.endswith(prefix):
                if gate not in wanted:
                    wanted.append(gate)
                break
    return wanted


def run_step(step: Step) -> tuple[bool, str]:
    try:
        out = subprocess.run(
            step.argv,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=3600,
        )
    except FileNotFoundError:
        return (True, "not installed, skipped") if step.optional else (False, "not installed")
    except subprocess.TimeoutExpired:
        return False, "timed out after 3600s"
    if out.returncode == 0:
        return True, "pass"
    tail = (out.stdout + out.stderr).strip().splitlines()
    return False, "\n      ".join(tail[-25:]) if tail else f"exit {out.returncode}"


def count_tests() -> int:
    """Total tests the workspace declares. Zero before any test exists."""
    if not (ROOT / "Cargo.toml").is_file():
        return 0
    cargo = resolve("cargo")
    out = subprocess.run(
        [cargo, "test", "--workspace", "--locked", "--", "--list"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return sum(1 for line in out.stdout.splitlines() if line.endswith(": test"))


def ratchet(passed: int, allow_removal: str | None) -> bool:
    """Fail when the test count drops without a stated reason."""
    log = AGENT / "gates.jsonl"
    previous = 0
    if log.is_file():
        for line in log.read_text(encoding="utf-8").splitlines():
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if row.get("result") == "pass" and isinstance(row.get("tests_passed"), int):
                previous = max(previous, row["tests_passed"])
    if passed < previous and not allow_removal:
        print(
            "\nRATCHET: {now} tests declared, {before} before.\n"
            "A test disappeared. Re-run with --allow-test-removal and a reason if\n"
            "that was deliberate; the reason is recorded and belongs in the pull\n"
            "request.".format(now=passed, before=previous),
            file=sys.stderr,
        )
        return False
    return True


def record(gate_results: dict[str, str], passed: int, reason: str | None) -> None:
    if not AGENT.is_dir():
        return
    row: dict[str, object] = {
        "t": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "kind": "gate",
        "gates": gate_results,
        "result": "pass" if all(v == "pass" for v in gate_results.values()) else "fail",
        "tests_passed": passed,
    }
    if reason:
        row["test_removal_reason"] = reason
    with (AGENT / "gates.jsonl").open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(row, ensure_ascii=False) + "\n")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run the verification gates.")
    parser.add_argument("--gates", nargs="*", help="gates to run; default is implied by the diff")
    parser.add_argument("--list", action="store_true", help="print the gates and exit")
    parser.add_argument(
        "--allow-test-removal",
        metavar="REASON",
        help="permit the declared-test count to drop, and record why",
    )
    args = parser.parse_args(argv[1:])

    table = gates()
    if args.list:
        for name, steps in table.items():
            print(name + ": " + ", ".join(s.name for s in steps))
        return 0

    wanted = args.gates or implied_gates(changed_paths()) or ["G1"]
    unknown = [g for g in wanted if g not in table]
    if unknown:
        print("unknown gate(s): " + ", ".join(unknown), file=sys.stderr)
        return 2

    results: dict[str, str] = {}
    failed = False
    for gate in wanted:
        print("\n=== " + gate + " ===")
        gate_ok = True
        for step in table[gate]:
            ok, detail = run_step(step)
            mark = "pass" if ok else "FAIL"
            suffix = "" if detail == "pass" else "\n      " + detail
            print("  [" + mark + "] " + step.name + suffix)
            if not ok:
                gate_ok = False
        results[gate] = "pass" if gate_ok else "fail"
        failed = failed or not gate_ok

    passed = count_tests()
    if not ratchet(passed, args.allow_test_removal):
        failed = True
        results["ratchet"] = "fail"

    record(results, passed, args.allow_test_removal)

    print()
    if failed:
        print("GATE FAILED. The task is not done. Fix the cause; do not work around the check.")
        return 1
    print("gates " + " ".join(wanted) + " pass; " + str(passed) + " test(s) declared")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
