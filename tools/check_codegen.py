#!/usr/bin/env python3
"""Check that the committed contract artefacts match the schema they came from.

The generator writes into the repository rather than into a build directory, for
three reasons given in ADR-0039. That choice has one failure mode: somebody edits
a schema, forgets to regenerate, and the committed types keep describing the old
protocol while the page describes the new one. Nothing else catches it, because
both files are individually well formed.

So this regenerates into a temporary directory and compares. Two differences are
reported differently on purpose:

* a file that differs is stale, and the fix is to run the generator;
* a file that is missing from the repository has never been generated, which
  usually means an output directory was added to the generator and not to this
  check.

The reverse direction matters too. A committed artefact the generator no longer
produces is a file describing a message that no longer exists, and a reader has
no way to tell. That is an error as well.

Exit code 0 when everything matches, 1 otherwise.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "contract" / "codegen" / "Cargo.toml"
SCHEMA = ROOT / "contract" / "schema"

# Flag on the generator, directory in the repository, glob for what lands there.
#
# The Rust and C# outputs are absent until the crates that hold them exist. They
# are listed here rather than remembered, so adding the crate is one line.
OUTPUTS: list[tuple[str, str, str]] = [
    ("--out-docs", "docs/reference/contract", "*.md"),
    # ("--out-rust", "crates/afk-contract/src/generated", "*.rs"),
    # ("--out-csharp", "src/AfkAgent.Contract/Generated", "*.g.cs"),
    # ("--out-grammar", "crates/afk-grammar/src/generated", "*.gbnf"),
]


def cargo() -> str:
    """Find cargo, which is not on PATH in every shell this runs from."""
    found = shutil.which("cargo")
    if found:
        return found
    candidate = Path.home() / ".cargo" / "bin" / "cargo.exe"
    return str(candidate) if candidate.exists() else "cargo"


def main() -> int:
    if not MANIFEST.exists():
        print(f"error: {MANIFEST} does not exist", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        fresh = Path(tmp)
        command = [
            cargo(),
            "run",
            "--quiet",
            "--locked",
            "--manifest-path",
            str(MANIFEST),
            "--",
            # Relative, and run from the repository root. The reference page
            # records where it came from, so an absolute path here would commit
            # this machine's directory layout to a public page.
            "--schema",
            str(SCHEMA.relative_to(ROOT)).replace("\\", "/"),
        ]
        for flag, _, _ in OUTPUTS:
            command += [flag, str(fresh / flag.removeprefix("--out-"))]

        result = subprocess.run(
            command,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
        if result.returncode != 0:
            sys.stdout.write(result.stdout)
            sys.stderr.write(result.stderr)
            print("error: the generator refused the schema", file=sys.stderr)
            return 1

        errors: list[str] = []
        for flag, target, pattern in OUTPUTS:
            generated = fresh / flag.removeprefix("--out-")
            committed = ROOT / target

            produced = {path.name: path for path in generated.glob(pattern)}
            present = {path.name: path for path in committed.glob(pattern)}

            for name, path in sorted(produced.items()):
                want = path.read_text(encoding="utf-8")
                if name not in present:
                    errors.append(
                        f"{target}/{name} is missing; run the generator and commit it"
                    )
                    continue
                have = present[name].read_text(encoding="utf-8")
                if have != want:
                    errors.append(
                        f"{target}/{name} is stale; regenerate it from "
                        f"contract/schema and commit the result"
                    )

            for name in sorted(set(present) - set(produced)):
                errors.append(
                    f"{target}/{name} is committed but the generator no longer "
                    f"produces it; delete it or restore its schema"
                )

    for error in errors:
        print(error)
    print(
        f"checked {sum(1 for _ in OUTPUTS)} output directory(ies), "
        f"{len(errors)} problem(s)",
        file=sys.stderr,
    )
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
