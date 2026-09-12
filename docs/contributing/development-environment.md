---
title: Development environment
description: "Exact toolchain versions, how to install them, and how to verify the result."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing, tooling]
requirements: []
decisions: []
generated: false
---

# Development environment

There is no application code yet, so this page has two halves: what you need to work on the documentation today, and what you will need when the code exists.

Only the first half is verified.

## Documentation, today

Verified on Windows 11 build 26200 on 2026-09-12.

**Python 3.12**, for the site and the lint scripts.

```bash
winget install --id Python.Python.3.12 --accept-package-agreements --accept-source-agreements
```

```bash
python -m pip install -r docs/requirements.txt
```

**Node.js LTS**, for the Markdown and spelling checks.

```bash
winget install --id OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
```

**Vale**, for prose.

```bash
winget install --id errata-ai.Vale --accept-package-agreements --accept-source-agreements
```

Vale downloads its style package on first use:

```bash
vale sync
```

### Verify

Each of these should report no problems.

```bash
python -m mkdocs build --strict
```

```bash
python tools/lint_frontmatter.py docs
```

```bash
python tools/lint_requirements.py docs
```

```bash
npx markdownlint-cli2 "**/*.md"
```

```bash
npx cspell "docs/**/*.md" "*.md"
```

```bash
vale --minAlertLevel=error docs/ *.md
```

The last one prints warnings and suggestions when run without the flag.
Those are advisory; only errors fail continuous integration.

### Serve the site locally

```bash
python -m mkdocs serve
```

## Code, when it exists

**Not verified.** Nothing here has been installed or tested, because there is nothing to build.

**Rust**, pinned by a toolchain file at the repository root once one exists.

```bash
winget install --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
```

**.NET SDK**, pinned by a version file at the repository root once one exists.
The project targets the current release; `ADR-0011` is accepted and the exact version is pinned by that file rather than stated here.

**Visual Studio 2026**, with the desktop development and Windows application development workloads.
The workload names are unverified; check them in the installer rather than trusting this page.

### Why the versions are not listed here

Because they belong in the files that enforce them, not in a page that drifts.

Once the code exists, the toolchain versions live in the pinning files and this page points at them.
A version written in prose is a version that disagrees with the build within a month.

## A note about paths

If you install Python or Node while a shell is already open, the new entries are not on that shell's path.
Open a new shell, or call the executables by full path.

This catches people once per machine and costs twenty minutes of confusion.

### A coding agent gets a reduced path

An agent working in this repository is given a shell whose `PATH` carries the
system directories and Git, and nothing else. `dotnet`, `node`, `gh` and
`cargo` are absent, and `python` resolves to the Windows Store stub in
`WindowsApps`, which opens the Store rather than running an interpreter.

Two mechanisms exist, in this order:

1. `.claude/settings.local.json` sets a full `PATH` with the real Python ahead
   of `WindowsApps`. It is machine-specific and is not in version control.
2. `tools/agent/tools.json` maps each tool to its absolute path, and
   `tools/agent/env.sh` repairs the path of a shell that is already running.

The second is a fallback rather than the normal route. A tool that has to be
found through it is a sign the environment is wrong, and the fix belongs in the
settings file.

The failure this prevents is the expensive one: an agent that concludes a tool
is not installed, records that as a finding, and works around a problem that
does not exist.

## Continuous integration

Continuous integration runs the documentation checks on hosted runners, which is why the toolchain is pinned in `docs/requirements.txt`.

The code checks, when they exist, may need a self-hosted runner: the hosted images are unlikely to carry a preview Visual Studio, and the latency benchmark has to run on known hardware to mean anything.
That is an unresolved question in the [known-good matrix](../known-good-matrix.md).

## What you do not need

- A graphics card, to work on documentation.
- A game, to work on documentation.
- Model weights, until there is something that loads them.

The specification is the deliverable at the moment, and it is editable on any machine.
