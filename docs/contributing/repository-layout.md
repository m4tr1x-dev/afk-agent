---
title: Repository layout
description: "One line per directory, and what belongs where."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing]
requirements: []
decisions: [ADR-0016]
generated: false
---

# Repository layout

## Top-level directories

Every tracked top-level directory appears here, and `tools/lint_layout.py`
fails the build when one does not. The `state` column carries whether the
directory exists today or is part of the shape `ADR-0016` sets.

| Directory | State | Contents |
| --- | --- | --- |
| `.claude/` | present | Coding-agent permissions and hooks; see [autonomous builds](autonomous-builds.md) |
| `.github/` | present | Workflows, issue and pull request templates, code owners |
| `docs/` | present | The specification and all other documentation |
| `styles/` | present | Vale prose rules and vocabulary |
| `tools/` | present | Documentation and build scripts |
| `contract/` | planned | Message and tool schemas, the single source of truth |
| `crates/` | planned | The Rust core, one crate per subsystem boundary |
| `src/` | planned | The C# shell and overlay |
| `tests/` | planned | Corpus, benchmark harness, replay fixtures |
| `experiments/` | planned | Timeboxed probes that answer a known-good-matrix question |

Everything else at the root is configuration: the site, the linters, the
editor, and the repository metadata.

Untracked directories are outside this table by design. The documentation site
output, the Rust target directory, build artefacts and the agent's own scratch
space are generated rather than authored, and a check that policed them would
fail on a clean working copy.

## Inside `docs/`

| Directory | Contents |
| --- | --- |
| `docs/spec/` | Normative pages |
| `docs/decisions/` | Architecture decision records |
| `docs/explanation/` | The reasoning behind the hard parts |
| `docs/reference/` | Factual pages, mostly generated once code exists |
| `docs/how-to/` | Task-oriented guides |
| `docs/tutorials/` | Guided lessons |
| `docs/contributing/` | How to work here |
| `docs/assets/` | Images and rendered diagrams |
| `docs/snippets/` | Shared documentation fragments |

## Notes on the planned directories

`ADR-0016` is accepted and sets `contract/`, `crates/`, `src/` and `tests/`.

`experiments/` is not in that record. It is listed here because the probes
that answer the [known-good matrix](../known-good-matrix.md) need somewhere to
live, and a record amending the layout is written when the first one lands
rather than in advance of it.

`contract/` is the one worth noting. The inter-process message catalogue and
the tool schemas are defined once and generate types for both languages and the
reference documentation. Hand-written types on two sides of a boundary drift,
and the drift is found at runtime.

`experiments/` holds the probes that answer the questions in the
[known-good matrix](../known-good-matrix.md). Three rules govern it, because a
directory of throwaway code otherwise becomes a second codebase held to no
standard and deleted by nobody:

- No crate under `crates/` depends on anything in it.
- Each subdirectory is named `NN-topic/`, where `NN` matches the matrix
  question it answers.
- Each carries a `findings.md` and a `results/` directory of raw artefacts.

It is archived when the last question it holds is resolved. Porting probe code
into a real crate is an explicit, reviewed act rather than a move.

## Keeping this honest

`tools/lint_layout.py` runs on every pull request and compares the table above
against the tracked contents of the repository. It fails in both directions: a
tracked directory missing from the table, a row marked `present` that contains
nothing, and a row marked `planned` whose directory has arrived.

The third case is the one that matters over time. Adding a directory is
memorable; moving its row from `planned` to `present` is not.
