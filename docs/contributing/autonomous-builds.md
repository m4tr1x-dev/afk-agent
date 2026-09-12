---
title: Autonomous builds
description: "The harness that lets a coding agent work unattended, what it refuses to do, and how to stop it."
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

# Autonomous builds

Parts of this project are built by a coding agent working unattended for long
stretches. This page describes the machinery that makes that survivable, and —
more importantly — what it refuses to do and how to stop it.

## Stopping it

```bash
touch .agent/HALT
```

The session stops at its next opportunity. Delete the file to allow it to
continue. Nothing else is required, and nothing overrides it.

Three other files have the same effect and say something different about why:

| File | Meaning |
| --- | --- |
| `.agent/HALT` | The maintainer wants it stopped |
| `.agent/DONE-M8` | The build reached its last milestone |
| `.agent/BLOCKED-ON-HUMAN` | Progress needs a person; the file says what for |

## What it refuses to do

Some of the project's hard constraints are enforced by the harness rather than
by the agent remembering them. A constraint an agent has to hold in mind across
forty context compactions is a constraint that eventually lapses.

`.claude/settings.json` denies the commands a driver or service install needs —
`bcdedit`, `pnputil`, `sc create`, `certutil -addstore`. That is `CON-002`
turned from a rule into a refusal. It also denies `npm install`, which is
CLAUDE.md's "do not add Node to the build" made mechanical, while leaving
`npx --yes markdownlint-cli2` and `npx --yes cspell` allowed because those are
the documentation toolchain rather than a build dependency.

Force-pushing and pushing to `main` are denied. `main` carries no branch
protection, so this list is the only thing standing between an unattended run
and an unrecoverable history.

The permission mode is `acceptEdits` rather than `bypassPermissions`,
deliberately. Bypassing permissions would skip the deny list as well, and the
deny list is the part worth having. The cost is that the allow list has to be
genuinely exhaustive; that is work rather than a reason to change the mode.

## The build journal

`.agent/` is a worktree of an orphan branch, `agent-status`, and is ignored
from `main`:

```bash
git worktree add --orphan -b agent-status .agent
```

It is in version control because the maintainer is away and the repository is
the only channel that survives the machine. It is on a branch of its own
because a journal written on `main` would either dirty the working tree during
a pull request or add a commit to every feature branch. And it is JSON Lines
rather than Markdown because `markdownlint-cli2` lints `**/*.md` across the
whole repository, so a Markdown journal would fail a check on every write.

| File | Shape | Holds |
| --- | --- | --- |
| `state.json` | Rewritten | Milestone, exit criteria, current task, gates, blockers, invariants |
| `journal.jsonl` | Appended | One line per event; only its tail is ever read |
| `attempts.jsonl` | Appended | What was tried and failed, and whether to retry |
| `measurements.jsonl` | Appended | Every measured number, with its conditions |
| `blockers.jsonl` | Appended | What needs a person, and what they should do |
| `gates.jsonl` | Appended | Every gate run, and the passing-test count |

`attempts.jsonl` is the one that earns its place. The characteristic failure of
a long unattended run is retrying something that already failed, forty
compactions after the reason was forgotten.

!!! note "This is not the product's memory"

    `CON-006` — nothing the agent learns persists beyond the session — governs
    the **product** at run time. It does not govern a build journal written by
    a coding agent about its own work, which is engineering history and belongs
    in version control like any other.

    Session recordings remain write-only from the product's perspective, and
    that is what `INV-CTX-002` protects.

## Re-reading state after compaction

A session that runs for weeks is summarised many times. Relying on the agent to
remember to re-read a file after a summary is the same class of error as
relying on it to remember an invariant, so the harness does it instead.

`tools/agent/brief.py` runs from a `SessionStart` hook on startup, resume and
compaction. It prints `state.json` whole, the tail of the journal, every
attempt marked as not worth retrying, open blockers and the state of the
working tree — capped at twelve thousand characters so it never competes with
the work for context.

## The gate

`tools/agent/gate.py` is CLAUDE.md's "do not claim a build passed when it was
not run", made mechanical.

```bash
python tools/agent/gate.py            # the gates the working tree implies
python tools/agent/gate.py --gates G1 # a named gate
python tools/agent/gate.py --list
```

| Gate | Covers |
| --- | --- |
| `G0` | Toolchain present, tool registry valid, Vale styles synchronised |
| `G1` | The commands `docs-quality.yml` runs, command for command |
| `G2` | Rust: format, Clippy, tests, documentation, supply chain |
| `G3` | .NET: format, build, test |
| `G7` | Requirement identifiers and their covering tests |

`G1` mirrors the workflow exactly. A local check that does not predict a
continuous integration result is worse than none, because it teaches people to
skip it.

### The ratchet

Every green run records how many tests the workspace declares. A later run that
records fewer fails.

The characteristic failure of an autonomous build is not getting stuck. It is
getting unstuck by removing the obstacle — ignoring a test, deleting an
assertion, lowering a threshold, passing `--no-verify`. One rule catches all
four.

Overriding it takes `--allow-test-removal` with a reason, which is recorded in
`gates.jsonl` and belongs in the pull request body.

### Safety gates are never statistical

[Testing strategy](testing-strategy.md) is explicit that a panic key which
works 99.7% of the time has found a defect. The gate has no retry option for
safety suites, and it is not going to grow one. A single failure stops the
build.

## The path problem

The shell a coding agent is given starts with a reduced `PATH`: no `dotnet`,
`node`, `gh` or `cargo`, and `python` resolving to the Windows Store stub in
`WindowsApps`, which opens the Store rather than running an interpreter.

Three mechanisms, in order of preference:

1. `.claude/settings.local.json` sets the full path. Machine-specific, not in
   version control.
2. `tools/agent/env.sh`, sourced into a shell that is already running.
3. `tools/agent/tools.json`, an absolute-path registry consulted by
   `resolve()` when a lookup fails.

Gate `G0` validates the registry before anything depends on it. A malformed
registry does not announce itself: the parse fails, the lookup returns nothing,
and the gate reports "not installed" for a tool that is installed. A misleading
failure in the harness is worse than a loud one anywhere else.

## What the harness does not decide

It runs checks and records outcomes. It does not judge whether a design is
right, and it cannot tell a defect from a decision.

Three things stay with a person:

- Accepting a decision record. The evidence is gathered automatically; whether
  it supports the conclusion is not a mechanical question.
- Reading a game's terms of service. An agent assembles the clauses; it does
  not decide what they permit.
- Anything that would relax a constraint in
  [scope and goals](../spec/00-scope-and-goals.md). Those are not measurements
  and no experiment overturns them.
