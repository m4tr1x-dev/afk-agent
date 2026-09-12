---
title: Build and run
description: "How to build the parts that exist, and how to run the agent without a game."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing]
requirements: []
decisions: []
generated: false
---

# Build and run

## Today

The documentation is the only thing that builds.
See [development environment](development-environment.md) for the toolchain and the commands.

```bash
python -m mkdocs serve
```

## When the code exists

!!! warning "Nothing below has been run"

    This section describes what the specification implies.
    Correct it once any of it has actually been executed, and remove this
    warning at that point rather than leaving it as decoration.

The repository will carry two builds — a Rust workspace and a .NET solution — driven from one task runner, so that a contributor working on the overlay does not have to learn the other toolchain to get a running system.

## Running without a game

Three modes, and they exist because the ordinary one is slow to iterate against.

**Against a recording.**
The core reads a recorded session instead of capturing a live window.
No game, no model, fully deterministic.
This is how perception and planning changes are developed, and it is the mode the regression suites use.

**Dry run.**
Full capture and perception against a live game, with the model deciding, and **nothing synthesised**.
Every intended action is logged instead of executed.

Safe on any game, which is what makes it the mode for collecting the labelled corpus and for the first minutes on a game nobody has tried.

**Headless.**
The core with no shell and no overlay, used by the benchmark harness.

Note that headless still runs in the user's own desktop session.
A process in another session can neither capture their windows nor deliver input to them, so headless means *no interface*, not *no session*.

## The first thing that has to work

Before any of the above matters, one question has to be answered: **does synthesised relative mouse movement reach a real game?**

The reachability probe is a small program that captures a window, emits a known sequence of movements, and checks whether the view turned.

It is the first item in the [known-good matrix](../known-good-matrix.md) and the first milestone on the [roadmap](../roadmap.md).
If it fails for a game, nothing else in this project works for that game, and it is far better to learn that from a short probe than from three months of building on the assumption.
