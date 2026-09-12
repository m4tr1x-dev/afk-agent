#!/usr/bin/env python3
"""Enforce FR-ACT-008: input synthesis occurs in exactly one component.

    FR-ACT-008 — Input synthesis MUST occur in exactly one component, with no
    other call site anywhere in the system.

The requirement's rationale says why this is a check rather than a convention:
"A second call site is a second place the release guarantees can be violated,
and it is found by a user rather than by a test."

This check exists before the crate it guards. A rule added after the code it
governs is a rule people have already worked around.

Three rules, and the third is the one that is easy to overlook.

**Rule A — the operating system's input surface.** The synthesis functions may
appear only in `crates/afk-input/` and `crates/afk-guardian/`. The guardian is
on the list because releasing held input after the core dies is its entire
purpose (`FR-SAFE-003`), and it cannot call into a process that has exited.

**Rule B — the managed side synthesises nothing.** No `DllImport` or
`LibraryImport` of `user32` anywhere under `src/`. The shell and the overlay
render what the core publishes; neither has any business producing input, and
the overlay in particular must never take focus or intercept a click
(`FR-GUI-002`).

**Rule C — the unsafe surface is a list you can hold in your head.** Every
crate declares `#![forbid(unsafe_code)]` unless it is on a short allowlist of
crates that genuinely touch the platform. The check fails in both directions:
a missing declaration, and a declaration on a crate that has quietly acquired
one. Without it, "which code can do something unsound" is a question you answer
by searching rather than by reading six names.

Exit code 0 when the workspace obeys all three, 1 otherwise. Errors are printed
one per line in a form GitHub Actions renders as an annotation.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

# Where input may be synthesised. Nowhere else, in any language.
# COVERS: FR-ACT-008 - this check is the requirement's Lint verification.
INPUT_CRATES = ("crates/afk-input/", "crates/afk-guardian/")

# Crates permitted to contain `unsafe`. Each one binds to a platform interface
# that has no safe wrapper. Adding to this list is an architectural change and
# belongs in a decision record, not in a pull request that needed it.
UNSAFE_ALLOWED = {
    "afk-input",  # the input surface itself
    "afk-capture",  # graphics device, shared textures
    "afk-shm",  # shared memory mappings
    "afk-uia",  # accessibility tree, a COM interface
    "afk-ipc",  # named pipe, shared texture handles
    "afk-guardian",  # waits on a process handle, releases input
}

# The platform's input synthesis surface, plus the window-message route that
# ADR-0009 rules out on capability grounds rather than policy: it does not
# reach the raw-input path, so camera control does not work through it.
INJECTION = re.compile(
    r"\b("
    r"SendInput"
    r"|keybd_event"
    r"|mouse_event"
    r"|InjectSyntheticPointerInput"
    r"|InitializeTouchInjection"
    r"|SendMessage\w*\s*\(\s*\w+\s*,\s*WM_(?:KEY|CHAR|[LRM]BUTTON)"
    r")"
)

MANAGED_INPUT = re.compile(
    r"(DllImport|LibraryImport)\s*\(\s*\"user32(\.dll)?\"", re.IGNORECASE
)

FORBID_UNSAFE = re.compile(r"^\s*#!\[forbid\(unsafe_code\)\]", re.MULTILINE)
HAS_UNSAFE = re.compile(r"\bunsafe\s*(\{|fn\b|impl\b|extern\b)")

RUST_SUFFIXES = {".rs"}
MANAGED_SUFFIXES = {".cs"}
SEARCHED = {"crates", "src", "contract", "tests", "experiments", "tools"}


def annotate(message: str, *, file: str | None = None, line: int | None = None) -> None:
    if os.environ.get("GITHUB_ACTIONS"):
        location = ""
        if file:
            location = f" file={file}"
            if line:
                location += f",line={line}"
        print(f"::error{location}::{message}")
    else:
        where = f"{file}:{line}: " if file and line else (f"{file}: " if file else "")
        print(f"{where}{message}", file=sys.stderr)


def source_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for top in sorted(SEARCHED):
        base = root / top
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if path.suffix in RUST_SUFFIXES | MANAGED_SUFFIXES and "target" not in path.parts:
                files.append(path)
    return files


def relative(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def rule_a(files: list[Path], root: Path) -> int:
    """Input synthesis appears only where it is permitted."""
    errors = 0
    for path in files:
        rel = relative(path, root)
        if any(rel.startswith(prefix) for prefix in INPUT_CRATES):
            continue
        # This file names the functions in order to forbid them.
        if rel == "tools/check_call_sites.py":
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for number, line in enumerate(text.splitlines(), start=1):
            if INJECTION.search(line):
                annotate(
                    "FR-ACT-008: input synthesis must occur in exactly one component. "
                    f"Found an input call outside {' and '.join(INPUT_CRATES)}.",
                    file=rel,
                    line=number,
                )
                errors += 1
    return errors


def rule_b(files: list[Path], root: Path) -> int:
    """The managed side synthesises nothing."""
    errors = 0
    for path in files:
        if path.suffix not in MANAGED_SUFFIXES:
            continue
        rel = relative(path, root)
        text = path.read_text(encoding="utf-8", errors="replace")
        for number, line in enumerate(text.splitlines(), start=1):
            if MANAGED_INPUT.search(line):
                annotate(
                    "FR-ACT-008: the shell and the overlay render what the core "
                    "publishes and synthesise nothing. Remove this binding.",
                    file=rel,
                    line=number,
                )
                errors += 1
    return errors


def rule_c(root: Path) -> int:
    """Every crate forbids unsafe unless it is on the allowlist."""
    crates = root / "crates"
    if not crates.is_dir():
        return 0

    errors = 0
    for crate in sorted(p for p in crates.iterdir() if p.is_dir()):
        name = crate.name
        entry = None
        for candidate in ("src/lib.rs", "src/main.rs"):
            if (crate / candidate).is_file():
                entry = crate / candidate
                break
        if entry is None:
            continue

        text = entry.read_text(encoding="utf-8", errors="replace")
        declares = bool(FORBID_UNSAFE.search(text))
        rel = relative(entry, root)

        if name in UNSAFE_ALLOWED:
            if declares:
                annotate(
                    f"{name} is on the unsafe allowlist but forbids unsafe. "
                    "Remove it from the allowlist in tools/check_call_sites.py; "
                    "a shorter allowlist is the point.",
                    file=rel,
                )
                errors += 1
            continue

        if not declares:
            annotate(
                f"{name} does not declare #![forbid(unsafe_code)]. Add it, or add "
                "the crate to the allowlist in tools/check_call_sites.py and say "
                "in a decision record why it needs the platform.",
                file=rel,
            )
            errors += 1
            continue

        # Belt and braces: the attribute is the compiler's job, but a crate
        # that both forbids unsafe and contains it means the attribute was
        # added to a file that is not the crate root.
        for source in sorted(crate.rglob("*.rs")):
            body = source.read_text(encoding="utf-8", errors="replace")
            if HAS_UNSAFE.search(body) and not FORBID_UNSAFE.search(body):
                if source == entry:
                    continue
                annotate(
                    f"{name} forbids unsafe at its root but this file contains it; "
                    "the attribute is probably on the wrong module.",
                    file=relative(source, root),
                )
                errors += 1
    return errors


def main(argv: list[str]) -> int:
    root = Path(argv[1]) if len(argv) > 1 else Path(os.environ.get("CLAUDE_PROJECT_DIR", "."))
    root = root.resolve()

    files = source_files(root)
    errors = rule_a(files, root) + rule_b(files, root) + rule_c(root)

    if errors:
        print(f"{errors} call-site violation(s)", file=sys.stderr)
        return 1

    scanned = len(files)
    if scanned == 0:
        print("call sites: no source files yet; the rule is in place before the code")
    else:
        print(f"call sites: {scanned} source file(s), FR-ACT-008 holds")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
