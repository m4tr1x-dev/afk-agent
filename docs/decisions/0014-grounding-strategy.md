---
title: ADR-0014 — Build both grounding paths and let measurement choose
description: "How a described target becomes a screen coordinate. Proposed, and blocked on a measurement."
status: proposed
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, grounding]
requirements: [FR-GND-001, FR-GND-002, FR-GND-003, FR-GND-004, FR-GND-005]
decisions: [ADR-0013, ADR-0036]
affects: [FR-GND-001, FR-GND-002, FR-GND-004]
spec: [spec/18-grounding-and-verification.md]
evidence: [docs/explanation/grounding-clicks-in-pixels.md, docs/known-good-matrix.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0014 — Build both grounding paths and let measurement choose

!!! warning "Proposed, not accepted"

    This record is **blocked on a measurement that has not been taken**, and it
    is left proposed deliberately rather than being accepted on reasoning.

    Accepting it now would be exactly the failure the [decision
    process](../contributing/adr-process.md) warns about: a guess in a smart
    format.

## Context and problem statement

The model says "click the shop button".
Something has to turn that into a coordinate that hits the shop button.

Two approaches exist, they have opposite strengths, and **the evidence needed to choose between them does not exist for the model this project targets.**

Published results give the shape of the problem.
A parser-plus-general-model pipeline reaches roughly 39.5% on a demanding high-resolution grounding benchmark; agents trained end to end on interface trajectories reach around 72.9%, and about 80.3% with a two-stage refinement pass.

There are **no published grounding results for this model family**, and its model card makes no computer-use claim.
Its direct-coordinate ability is unknown in both directions.

## Decision drivers

- A miss costs an action, a verification cycle, and a step up the no-progress ladder.
- Some targets are not elements the perception layer proposed, and cannot be selected from a list.
- Some targets are positions rather than things.
- Whatever is chosen runs several times a second.

## Considered options

1. Direct coordinates only
2. Mark selection only
3. Both, with the choice fixed in advance
4. Both, with the choice made at runtime from measured success

## Proposed decision

**Option 4**, with a global starting preference fixed by the benchmark.

Three mechanisms, all built:

**Mark selection.** Perception enumerates candidates, draws numbered marks, and the model chooses an integer. The [grammar](0013-constrained-decoding.md) is regenerated each tick from the marks actually present, so a reference to something absent cannot be expressed. Precision becomes perception's problem, which is where it belongs.

**Direct coordinates.** For targets perception did not propose. Flexible, and imprecise.

**A labelled grid.** For positions that are not elements at all. Coarse by construction, and coarse-and-correct is what "walk over there" needs.

Plus **two-stage refinement** (`FR-GND-005`): a cheap look to localise, then an expensive look at the localised region.
This is the one part of the specialist agents' advantage available without training, and the published gap between one pass and two on the same model is what justifies it.

**The arbiter** tracks, per scene class, how often each path produced a confirmed outcome, and prefers the one that is winning.
Its counters are session-scoped and discarded with the session; nothing about a game persists.

## Consequences

### Good

- The system is not betting on an unmeasured property of the model.
- Mark selection makes a whole class of failure structurally impossible.
- The arbiter is a measurement rather than a belief, and it adapts within a session.

### Bad

- Both paths are built, which is more work than picking one.
- The arbiter needs enough confirmed outcomes per scene class before its preference means anything, and early in a session it has none.
- Direct coordinates remain available and remain imprecise. Enabling them where they have not earned it is the thing this record is most likely to get wrong.

## Validation

**This record is not accepted until the grounding benchmark has run.**

The measurement, specified in [the evaluation harness](../spec/20-evaluation-harness.md): 200 to 500 labelled frames across three to five games, each with a described target and a true box, measured on all three approaches, reported per scene class and per game.

It decides:

- Whether direct coordinates are viable at all for this model.
- The arbiter's global starting preference.
- Whether refinement earns its extra call.
- Where perception, rather than the model, is the limiting factor.

If direct coordinates turn out to be unusable, this record is simplified to mark selection plus the grid, and the arbiter's job shrinks to choosing between marks and refinement.

If they turn out to be strong, the arbiter still exists, because "strong on menus" and "strong in a world view" are different claims and the per-scene-class breakdown is what separates them.
