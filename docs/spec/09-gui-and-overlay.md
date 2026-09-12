---
title: Application shell and overlay
description: "The shell, the in-game overlay, and the rules that keep the overlay out of its own capture."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, gui]
requirements: [FR-GUI-001, FR-GUI-002, FR-GUI-003, FR-GUI-004, FR-GUI-005, NFR-GUI-001, INV-GUI-001, FR-SAFE-001, FR-SAFE-007]
decisions: []
generated: false
---

# Application shell and overlay

## Purpose

Two surfaces with different jobs.

The **shell** is where a user sets up a session and reviews one afterwards.
The **overlay** is what a user sees when they walk back to the desk and need to know, within a second, what is happening and how to stop it.

They are separate processes for the reasons in [container architecture](03-container-architecture.md), and `INV-GUI-001` requires their lifetimes to be independent.

## The overlay

The more important of the two, because it is the one present while the agent has hands.

### What it must do

`FR-GUI-002`, `FR-GUI-003`, `FR-SAFE-007`.

- Never take focus, and never intercept a click. Input passes through it to the game.
- Show, at all times while the agent can act: the goal, the current subgoal, the last action, elapsed budget, and the panic key binding.
- Show a visible indicator whenever input synthesis is possible — not only while acting. An agent that is thinking will move in half a second, and a user must never have to guess.
- Own and register the panic key (`FR-SAFE-001`).
- Optionally draw the agent's annotations: mark boxes, the point last acted on, a short trail.

### Exclusion from capture

`FR-GUI-001`.

!!! danger "The overlay must be excluded from the agent's own capture"

    Without this the agent sees its own annotations in the captured frame,
    treats them as part of the game, and reasons about them.

    The result is a perception feedback loop: marks drawn over marks, an
    element list polluted by the agent's own drawing, and behaviour that looks
    like a confused model rather than a configuration defect.

    It is catastrophic, subtle, and very hard to diagnose from the symptom.
    The exclusion applies to **every** window the overlay owns, including
    tooltips and flyouts, which are separate top-level windows and are easy to
    forget.

### Rendering

The overlay is a transparent, always-on-top, non-activating, click-through window.

Annotations are drawn on a composition layer rather than as interface elements.
At the reflex rate with dozens of boxes, a layout pass per box is not affordable; one drawing surface updated per tick is.

**Material effects are for the shell, not the overlay.**
A blur-behind effect over a game samples the game's own output, which costs graphics time the game needs and looks wrong over moving content.
The overlay uses flat semi-transparent panels with a clear edge.
Mica and Acrylic belong on the shell's navigation, settings and review surfaces, where they look right and cost nothing.

### Display scaling and monitors

The overlay is declared per-monitor scaling aware and rebuilds its transform when the scaling or the monitor topology changes.

It follows the target window: if the game moves to another monitor, the overlay follows.
Coordinate handling uses the single chain in [perception](04-perception.md) and performs no arithmetic of its own (`INV-GND-002`).

### Exclusive fullscreen

A composited overlay cannot draw over a genuinely exclusive-fullscreen surface.

In order of preference:

1. **Most games are not actually exclusive.** Modern Windows composites most nominally fullscreen games, and the overlay works normally.
2. **Where it is exclusive, detect and tell the user**, recommending borderless windowed — a one-line settings change in nearly every game.
3. **Fall back to the shell** on a second monitor, showing the annotated mirror.
4. **Never hook the game's rendering to draw inside it.** `CON-002`. This is the technique that would work, and it is the one the project does not use.

## The shell

### Screens

| Screen | For |
| --- | --- |
| Home | Choose a target window, write a goal, set budgets, start |
| Live | Annotated frame, current plan, action stream, budget and latency meters |
| Review | Replay a finished session, frames aligned with actions and outcomes |
| Settings | Model, capture, safety limits, recording policy |
| Logs | Structured event stream, filterable |

### The live view

Shows the annotated frame as the agent sees it, which is the fastest way for a user to understand a failure: an agent clicking the wrong thing and an agent not seeing the right thing look identical from outside and obvious here.

The frame arrives through the shared texture described in [container architecture](03-container-architecture.md) — no pixel copy through main memory.

### Review

`FR-GUI-005`.

Scrub a completed session with frames, the plan at that moment, the action taken, and the verification outcome aligned on one timeline.

This is the only way a user finds out why something went wrong overnight, and it is the reason recording exists at all.

### Responsiveness

`NFR-GUI-001`.

The shell stays responsive while the core is loaded.
It performs no inference, no capture and no perception; it renders what the core publishes.

Freezing the window that contains the stop button, at the moment the system is under load, is the worst possible time to freeze it.

### Independent lifetime

`FR-GUI-004`.

Closing the shell does not stop a session.
Stopping a session does not close the shell.

A user who starts a session and closes the window has not asked for the agent to stop, and the overlay is what remains visible.

## Accessibility

The shell is fully keyboard reachable, every control is labelled for assistive technology, and contrast meets the platform guidance.

The overlay is a status display and is not interactive, so its accessibility obligation is different: the information it shows is also available in the shell and in the logs, because an overlay is useless to a user who cannot see it.

## Localisation

Interface strings are externalised from the first commit.

This is the one place in the project where a language other than English is expected — the [documentation guide](../contributing/documentation-guide.md) permits it precisely here.
Retrofitting string externalisation is tedious and doing it from the start costs nothing.

## Interfaces

| Direction | Interface |
| --- | --- |
| Shell to core | Commands over the control pipe |
| Core to shell | Events over the control pipe; annotated frames over the shared texture |
| Core to overlay | Telemetry over the shared ring |
| Overlay to core | The panic flag over shared memory; a heartbeat over the pipe |

## Open questions

1. Whether the overlay should be moveable by the user. It must not take focus, and a window that cannot be focused is awkward to drag.
2. What the overlay does when the game is minimised. Hiding matches expectations; staying visible is what a user returning to the desk needs.
3. Whether the live view should be available when the shell is on the same monitor as a fullscreen game. It cannot be seen there, and rendering it anyway costs graphics time the game wants.
4. How much annotation is useful versus distracting. Every mark box is informative during debugging and noise during normal running.
5. Whether the shell should be able to attach to a core it did not start, so a user can reopen the window mid-session. Desirable; needs a discovery and reattach protocol in [the inter-process protocol](10-ipc-protocol.md).
6. Whether capture-border suppression is available for an unpackaged application on the target Windows build. If not, every captured frame carries a border the perception layer must crop.

## Related decisions

`ADR-0010` will record the overlay rendering approach and `ADR-0011` the application framework and packaging.
Neither is accepted.
