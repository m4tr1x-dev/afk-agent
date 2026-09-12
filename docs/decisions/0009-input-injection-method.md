---
title: ADR-0009 — Synthesise input through the public API, with no kernel component
description: "How input reaches the game, and what techniques are permanently excluded."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, action, safety, policy]
requirements: [FR-ACT-004, FR-ACT-005, FR-ACT-008, INV-ACT-001, INV-ACT-003, CON-002]
decisions: [ADR-0025]
affects: [FR-ACT-008, INV-ACT-001, CON-002]
spec: [spec/06-action-and-input.md, spec/12-safety-and-limits.md]
evidence: [docs/explanation/anti-cheat-and-terms-of-service.md, docs/explanation/sustained-actions-explained.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0009 — Synthesise input through the public API, with no kernel component

## Context and problem statement

The agent has to deliver keyboard and mouse input to a game.

Several techniques exist, and they differ enormously in what they require, what they can reach, and what they imply about the project.
The choice is not only technical: it determines whether this software can be described honestly as an accessibility-style tool or not.

## Decision drivers

- It must reach games that read raw input for camera control, which many do.
- It must not require a kernel component or entry into the game's process.
- It must be auditable: one place in the system where input is produced.
- It must be honest about detectability rather than pretending to be otherwise.

## Considered options

1. The platform's public input synthesis API
2. A signed kernel driver presenting a virtual input device
3. Posting messages to the target window
4. A hardware device driven over a serial link

## Decision

**The platform's public input synthesis API**, from exactly one component, with no kernel driver, no code injection, no hooking and no reading of game memory.

The exclusions are the substantive part of this decision and they are permanent.

Posting messages to a window is ruled out on capability: it does not reach the raw-input path, so camera control in most games does not work at all.

A kernel driver or a hardware device would both work, and both are excluded.
They require signing and distribution of a driver, they are the class of tooling game protection systems treat as adversarial, and they would make the project's position — that everything it does is something a screen reader or accessibility tool does — untrue.

**This is not a stealth strategy.**
Synthesised input is marked by the platform, any process can read that mark, and we do not obscure it.
The decision buys defensibility, not invisibility. See [ADR-0025](0025-terms-of-service-and-anti-cheat-posture.md).

### One call site

`FR-ACT-008`. Input is synthesised in the executor and nowhere else.

Every release guarantee in the project terminates there, and a second call site is a second place those guarantees can be violated — one that would be found by a user rather than by a test.

### Motion synthesis is functional, not camouflage

The executor emits curved pointer paths, varied hold durations and varied gaps, and `FR-ACT-005` requires it.

The reasons are that interfaces which activate on hover discard a click arriving in the same frame as the pointer, input code that debounces filters a zero-duration press, and games reading raw input clamp or reject a single implausible movement.
Without it the agent cannot operate a game.

**Tuning this to be harder to detect is out of scope and will not be accepted.**
The documentation states this explicitly so that no future contributor mistakes the mechanism for a stealth feature.

## Consequences

### Good

- No driver to sign, distribute or maintain.
- Nothing in the game's process, so a defect in this software cannot corrupt one.
- One auditable call site.
- The project's description of itself is accurate.

### Bad

- **Detectable.** Trivially, by any process that installs a low-level input hook. Accepted.
- Some games may not accept synthesised input at all. Unknown until measured, and it is the first experiment on the roadmap.
- Cannot reach an elevated window, which is correct behaviour rather than a limitation.

### Neutral

- Gamepad output would need a different mechanism and is out of scope for the first version.

## Validation

The decision is **blocked on one measurement**: does synthesised relative movement reach a real game?

The reachability probe — capture a window, emit a known movement sequence, detect whether the view turned — runs against five games on different engines.

If it fails broadly, this decision does not change, because the alternatives are excluded on other grounds.
What changes is the project's scope: it works for the games where input arrives and not for others, and that has to be stated plainly rather than discovered by users.

The exclusions in this record do not expire and are not subject to measurement.

## More information

- [Terms of service and anti-cheat](../explanation/anti-cheat-and-terms-of-service.md) — what protection systems detect, and why this posture.
- [Sustained actions explained](../explanation/sustained-actions-explained.md) — why relative movement and held keys force the executor's design.
