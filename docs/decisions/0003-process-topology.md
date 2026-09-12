---
title: ADR-0003 — Split into five processes
description: "How many processes there are, and why each boundary exists."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, architecture]
requirements: [FR-SAFE-003, FR-GUI-004, INV-GUI-001, FR-MODEL-004]
decisions: [ADR-0002]
affects: [FR-SAFE-003, FR-GUI-004, INV-GUI-001]
spec: [spec/03-container-architecture.md]
evidence: [docs/spec/03-container-architecture.md, docs/spec/12-safety-and-limits.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0003 — Split into five processes

## Context and problem statement

The system captures a window, runs a model, synthesises input, shows a control surface, and shows an on-screen overlay.

All of that could be one process.
The question is which of those responsibilities must survive the failure of the others.

## Decision drivers

- The agent must outlive the window that started it.
- The panic key and the visible indicator must be present whenever the agent can act.
- A graphics fault or memory exhaustion must not take down something holding a key down.
- If the core dies, held input must still be released.
- A machine whose graphics memory is full should be able to run the model elsewhere.

## Considered options

1. One process
2. Two: core and interface
3. Five: core, shell, overlay, guardian, model host

## Decision

**Five processes**, all in the user's own desktop session.

Each boundary exists because something on the far side must survive a failure on the near side.

| Boundary | Survives |
| --- | --- |
| Core from shell | The user closing the control window, and faults in interface code |
| Overlay from shell | The same, while keeping the panic key and indicator present |
| Model host from core | Graphics memory exhaustion and driver resets |
| Guardian from core | The core dying while holding input |

The model host being an endpoint rather than a library pays twice.
It isolates the failure, and it makes running the model on another machine a configuration change rather than a rewrite.

**The core is not a Windows service.**
Session isolation means a service can neither synthesise input into the user's session nor capture their windows.
Headless means no interface, not session zero.

## Consequences

### Good

- Crash isolation is a safety property rather than a nicety.
- Headless operation falls out of the split, which the benchmark harness and replay both need.
- The overlay carrying the panic key is never blocked by interface work.

### Bad

- Five processes to start, supervise and shut down in order.
- Four communication channels with different shapes.
- Debugging spans processes.

### Neutral

- The overlay dying stops the session. That looks harsh for a status window and is deliberate: without it the user has no visible stop control.

## Validation

Revisit the overlay boundary only if the overlay could live inside the shell without the panic key or the indicator ever becoming unavailable — which would require the shell never to be closable, contradicting `FR-GUI-004`.

Revisit the guardian if the core proves crash-free over months of real use.
Until then it is cheap insurance against the one failure a user cannot recover from without a restart.
