---
title: Scope and goals
description: The system boundary — what is inside afk-agent, what belongs to the user, and what is external.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, scope]
requirements: [CON-001, CON-002, CON-003, CON-004, CON-005, CON-006, CON-007]
decisions: [ADR-0002, ADR-0003, ADR-0009, ADR-0025]
generated: false
---

# Scope and goals

## Purpose

This page draws the boundary.
It says what the system is responsible for, what the user is responsible for, what is external, and what is deliberately left out of the first version.

Every other specification page assumes this boundary.
A requirement that crosses it without saying so is a defect.

## What the system is

afk-agent is a **general computer-use agent for games**.

It takes one natural-language goal, observes a single game window, and operates the mouse and keyboard until the goal is met, a budget is exhausted, or the user stops it.

It carries no knowledge of any specific game, and acquires none that outlives the task.

## Inside the boundary

The system is responsible for:

- Capturing frames from one target window.
- Deriving a structured observation from each frame — text, candidate interactive elements, sensor values.
- Deciding what to do, at three separate rates.
- Synthesising keyboard and mouse input, including input that persists across many frames.
- Verifying that each action had its intended effect.
- Retrieving information from the web when the agent lacks it.
- Assembling and compacting the model context.
- Hosting the local model and managing its memory against the game's.
- Enforcing every safety interlock.
- Presenting a control surface and an on-screen overlay.
- Recording the session for later review.

## Outside the boundary

**The game.**
The system treats it as opaque.
It has no model of the game's rules, reads none of its state except through pixels, and does not know when the game is loading, paused or broken except by looking.

**The operating system's input and composition stack.**
The system uses public APIs and does not extend, replace or intercept them.

**The model weights.**
Downloaded by the user at first run, under their publisher's licence, stored outside the repository.
The system verifies and loads them; it does not distribute them.

**The internet.**
Reached only through the research skills, only on request, and never with anything derived from the user's screen.

**The user's other windows.**
Out of scope by design, and keeping them out is a safety property rather than a limitation — see `CON-004` below.

## The user's responsibilities

Stated because the system cannot enforce them and the documentation must not imply otherwise.

- Deciding whether automating a given game is permitted, and accepting the consequences. See [terms of service and anti-cheat](../explanation/anti-cheat-and-terms-of-service.md).
- Putting the game into a window mode the capture path supports.
- Writing a goal that can be pursued from what is visible on screen.
- Being present enough to notice when the agent is doing something unintended, on first contact with a new game.

## Assumptions

The system is specified against these.
If one is false, behaviour is undefined rather than merely degraded.

1. One game, one window, one monitor at a time.
2. The game is in windowed or borderless-windowed mode. Exclusive fullscreen is detected and reported, not worked around.
3. The user's account is their own and the machine is theirs.
4. The machine has a GPU that can hold the model and the game at once, or the user accepts running the model on the processor.
5. The user is not present during a session, but can return at any moment and expects control back immediately.
6. The game responds to synthesised input at all. This is not universally true and is the first thing the project verifies.

## Constraints

| ID | Constraint | Source |
| --- | --- | --- |
| `CON-001` | The system runs on Windows 11 only. | Capture, input synthesis and composition are platform-specific to the point where portability would be a rewrite. |
| `CON-002` | The system MUST NOT load a kernel driver, inject code into any process, hook any function, read another process's memory, or modify game files. | Architectural commitment; see [terms of service and anti-cheat](../explanation/anti-cheat-and-terms-of-service.md). |
| `CON-003` | Model inference MUST run locally, on the user's machine or on a host they control. | Privacy, and the cost of streaming continuous screen capture to a hosted model. |
| `CON-004` | Input MUST be delivered only while the target window is in the foreground. | The primary blast-radius control. |
| `CON-005` | The model's weights MUST NOT change as a result of running the system. | See [why the agent does not learn](../explanation/why-the-agent-does-not-learn.md). |
| `CON-006` | No information derived from the game MUST persist beyond the session that produced it, except in session recordings, which the agent MUST NOT read. | See [what the agent remembers](../explanation/what-the-agent-remembers.md). |
| `CON-007` | The system MUST NOT require elevated privileges for normal operation. | A tool with a game's window handle does not need administrator rights, and asking for them would be both unnecessary and alarming. |

## Out of scope for the first version

Not refusals — these are things a later version might do, listed so that nobody builds them by accident now.

- More than one game at a time, or more than one monitor.
- Gamepad and controller output.
- Exclusive fullscreen.
- Audio as an input signal.
- Launching or installing games.
- Any user interface for editing the agent's plan mid-session beyond stopping it and restarting with a different goal.
- Remote control from another device.
- Sharing anything between users.

## Permanent non-goals

These are refusals, and they do not expire.
The full list with reasoning is in [non-goals](19-non-goals.md); the short version is: competitive multiplayer, evading detection, real-money activity, accounts that are not the user's, training on gameplay, and platforms other than Windows.

## Open questions

1. **Non-blocking.** Does the system support a game running on a second machine, captured over a video link? Currently out of scope, but the capture layer does not inherently prevent it, and saying so explicitly would prevent someone assuming it works.
2. **Blocking.** What is the defined behaviour when the target window closes mid-session — stop, or wait for it to reappear? The safety answer and the usability answer differ.
3. **Non-blocking.** Is a game running inside a browser one target window, or does the browser chrome need to be excluded from the capture region?

## Related decisions

The constraints above are inputs to `ADR-0002`, `ADR-0003` and `ADR-0009`, all accepted, and to `ADR-0025`, which records the posture `CON-002` states.
`ADR-0008`, the capture API, is not written and is blocked on a prototype.
