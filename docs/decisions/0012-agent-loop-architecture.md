---
title: ADR-0012 — Run three concurrent cadences
description: "Why the agent has a reflex loop, a tactical loop and a deliberative loop rather than one."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, reasoning]
requirements: [FR-LOOP-001, FR-LOOP-002, NFR-LOOP-001, FR-ACT-002]
decisions: [ADR-0035]
affects: [FR-LOOP-001, FR-LOOP-002, NFR-LOOP-001]
spec: [spec/05-reasoning-loop.md]
evidence: [docs/explanation/how-the-agent-loop-works.md, docs/explanation/perception-token-economics.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0012 — Run three concurrent cadences

## Context and problem statement

A vision model answers in a few hundred milliseconds at best.
A game reads input every frame, sixty times a second.

An agent that acts only when the model answers is present for about one frame in twenty.

## Decision drivers

- Held keys and camera movement require being present every frame.
- Model calls cost two orders of magnitude more than a frame's budget allows.
- Planning benefits from time to think; acting does not have it.
- A slow call must not stop the agent from acting on its previous decision.

## Considered options

1. One loop at model speed
2. One loop at frame speed, with the model called asynchronously
3. Two loops: a fast executor and a model loop
4. Three loops: reflex, tactical, deliberative

## Decision

**Three cadences**, running concurrently at independent rates.

| Cadence | Rate | Model | Answers |
| --- | --- | --- | --- |
| Reflex | 20–30 Hz | None | What input should be delivered right now? |
| Tactical | 1–4 Hz | Small image budget, no extended reasoning | Given this subgoal and this screen, what next? |
| Deliberative | Every 10–60 s, or on trigger | Large image budget, extended reasoning | Is the plan still right? |

Option 1 cannot hold a key down.
Option 2 is closer, and collapses acting and planning into one rate, which means either planning is too frequent to afford or acting is too slow to play.

The split between tactical and deliberative is the one worth defending.
They differ in every respect — rate, image budget, reasoning depth, context length, what they produce — and running them at one rate forces a compromise that is wrong for both.

**The reflex loop never blocks on a model call** (`FR-LOOP-002`).
If it did, held input would stop being stepped, and the visible result in-game is stutter every time the agent thinks.

**The other cadences keep running during a deliberative pass** (`NFR-LOOP-001`).
This is what makes a fifteen-second deliberation invisible: for those fifteen seconds the agent is still executing its previous plan.

## Consequences

### Good

- The agent can hold keys, control a camera, and plan, without any of them blocking the others.
- Expensive reasoning is affordable because it is rare and hidden.
- The reflex loop is simple enough to reason about precisely, which is where every safety interlock lives.

### Bad

- Three loops with different lifetimes, sharing state that has to be safe to share.
- A plan revision arriving mid-action needs [selective cancellation](0035-sustained-action-execution.md) rather than a clean restart.
- Reasoning about timing means reasoning about three rates at once.

### Neutral

- The rates themselves are configuration, and their defaults depend on measurements that have not been taken.

## Validation

The architecture is settled; the **rates are not**, and they depend on the model latency measurement that blocks `NFR-MODEL-001`.

Revisit the split between tactical and deliberative if measurement shows tactical calls are cheap enough to carry planning as well — which would require the tactical budget to absorb extended reasoning without dropping below roughly 1 Hz.

Revisit the reflex loop's existence if the executor turns out not to need per-frame presence, which would also [invalidate the core language decision](0002-core-runtime-language.md) and should be taken seriously rather than resisted.
