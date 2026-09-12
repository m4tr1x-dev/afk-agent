---
title: ADR-0035 — Model issues intentions with durations; an executor steps them
description: "How a decision made once a second becomes input delivered every frame."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, action, safety]
requirements: [FR-ACT-002, FR-ACT-003, FR-ACT-004, FR-ACT-007, FR-ACT-008, FR-ACT-001, INV-ACT-002, INV-LOOP-002, FR-SAFE-003]
decisions: [ADR-0012, ADR-0009]
affects: [FR-ACT-002, FR-ACT-007, FR-ACT-001]
spec: [spec/06-action-and-input.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0035 — Model issues intentions with durations; an executor steps them

## Context and problem statement

Playing a game means holding a key for two seconds while turning the camera and holding a second button to aim.

That is not expressible as a sequence of instantaneous actions issued once per second, which is the vocabulary a desktop computer-use agent gets away with.

## Decision drivers

- Games need input present across many frames, not delivered at one instant.
- Camera control needs a stream of small relative movements, because a repositioned cursor produces no usable delta.
- Concurrent held inputs are ordinary play, not an edge case.
- Every held input must be released on stop, panic, focus loss and crash.

## Considered options

1. Instantaneous actions only, with the model re-issuing each tick
2. Actions carry durations; a fast executor steps them
3. The model emits a per-frame input script

## Decision

**Option 2.**

The model emits intentions with durations — hold this key for 1500 ms, turn right by this much over 400 ms — and the executor turns them into per-tick input state at the reflex rate.

Option 1 fails on its own terms: to hold a key the model would have to re-issue it every frame, at a rate it cannot reach.

Option 3 sounds appealing and puts timing inside the model's output, where it cannot respond to anything. A script authored a second ago cannot react to the screen having changed.

### The executor's tick

Five steps, same order every time:

1. Read the halt flag. If set, release everything and stop. Nothing below runs.
2. Check the foreground guard. If the target is not in front, release everything and pause.
3. Advance every active sustained action by one tick.
4. **Reconcile** the desired input state against what is actually held, producing the minimal event set.
5. Emit, and publish the held set to shared memory.

Step 4 being a reconciliation rather than a replay is what keeps the event rate proportional to change.
A key that should stay held produces no event at all.

Step 5 is what allows [the guardian](0003-process-topology.md) to clean up from outside the process.

### Ownership

Every held input records which subgoal owns it (`INV-LOOP-002`).

That is what makes selective cancellation possible: when the deliberative cadence abandons a subgoal, its inputs are released and another subgoal's are not.

Releasing everything on every plan change was the simpler alternative and is wrong — dropping a movement key mid-traversal is not a neutral act.

### One call site

`FR-ACT-008`. The executor is the only component that synthesises input.

Stated as a requirement rather than a convention because a second call site is a second place the release guarantees can be violated, and it would be found by a user rather than by a test.

## Consequences

### Good

- The agent can play rather than operate menus.
- Concurrent actions compose naturally when they do not contend for a key or axis.
- Every safety interlock has one obvious place to live, checked before anything else each tick.
- The event rate is proportional to change rather than to time.

### Bad

- A scheduler with cancellation, ownership and conflict resolution, which is more machinery than issuing events directly.
- The executor must run at a hard rate, which is a real-time constraint in a system that otherwise has none.
- A sustained action whose cancellation is lost leaves a key held. The maximum hold duration exists to bound that, and it is a mitigation rather than a fix.

### Neutral

- Camera control needs a calibration step at session start to learn how far a movement turns the view. No model involved, and without it aiming at a target cannot be implemented.

## Validation

Revisit if games turn out to accept a repositioned cursor for camera control — which would remove the need for per-frame presence in the movement path, though not for held keys.

The executor's existence is confirmed by the [reachability probe](../known-good-matrix.md).
If synthesised relative movement does not reach games at all, this record is not wrong, but the project's scope is much smaller than assumed.

The release guarantees are not subject to revision.
