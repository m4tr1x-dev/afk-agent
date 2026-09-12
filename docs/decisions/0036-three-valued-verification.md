---
title: ADR-0036 — Verification is three-valued, and failures are attributed
description: "Why an outcome that cannot be determined is its own result, and why every failure gets a blamed cause."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, grounding, observability]
requirements: [FR-GND-006, FR-GND-007, FR-GND-008, INV-GND-001, FR-LOOP-004]
decisions: [ADR-0014]
affects: [FR-GND-006, FR-GND-008, INV-GND-001]
spec: [spec/18-grounding-and-verification.md]
evidence: [docs/explanation/failure-modes-and-recovery.md, docs/reference/sensor-expressions.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0036 — Verification is three-valued, and failures are attributed

## Context and problem statement

After every action the agent has to answer one question: did that work?

The obvious answer is a boolean, and the obvious answer is wrong in a way that damages the control loop in two opposite directions depending on which way the ambiguous cases are rounded.

## Decision drivers

- Verification runs after every action, so it must be cheap.
- The loop's recovery behaviour depends entirely on its answer.
- Many situations genuinely cannot be determined: a loading screen, a mid-transition frame, an unavailable sensor.
- When things go wrong, somebody has to work out which subsystem is at fault.

## Considered options

1. Boolean: worked or did not
2. Boolean, with ambiguous cases treated as failure
3. Three-valued: confirmed, refuted, or cannot be determined
4. A confidence score

## Decision

**Three values**, plus an attributed cause on every refuted outcome.

### Why the third value is load-bearing

Collapsing it breaks the loop, differently in each direction.

Fold it into **confirmed** and the agent becomes credulous.
It proceeds believing things worked, and discovers otherwise several subgoals later, when recovery is expensive and the cause is buried under everything that happened since.

Fold it into **refuted** and the agent thrashes.
Every loading screen and every transition becomes evidence of failure, the no-progress detector fires constantly, and the escalation ladder runs on situations that were never wrong.

The honest answer to "did that work?" is frequently "cannot tell yet", and a control loop that cannot represent that answer will act on a guess several times a second.

A confidence score was considered and rejected: it pushes the decision to every consumer, each of which then picks its own threshold, and the thresholds diverge.

### The mechanism

Verification reads **sensor expressions** — named predicates over screen regions, evaluated on the graphics device every reflex tick without a model.

That is what makes checking every action affordable.
`FR-LOOP-004` requires each subgoal to carry its postcondition as such an expression rather than as prose, precisely so that checking costs a fraction of a millisecond instead of a model call.

### Attribution

Every refuted outcome is assigned a cause: grounding, action choice, perception, plan, execution, or the game itself.

The histogram over these is the project's primary diagnostic instrument.

The instinct in a system with a model in it is to blame the model.
A system whose text recognition misreads a counter presents exactly as a system whose model makes poor decisions, and only the attribution separates them.

It is also the entry condition for any future work on model quality: without it, effort goes wherever intuition points.

## Consequences

### Good

- The loop can distinguish "failed" from "unknown", which is what makes the escalation ladder behave sensibly.
- Verification is cheap enough to run after every action rather than on a sample.
- Failure attribution turns "the agent is not working" into a statement about a subsystem.
- The arbiter in [grounding](0014-grounding-strategy.md) has a signal to act on, which is what makes grounding improvable at all.

### Bad

- Every consumer of a verification outcome has three cases to handle, and forgetting the third is a defect that looks like flakiness.
- Deciding whether a scene has *settled* — which separates refuted from undetermined — is itself a heuristic, and getting it wrong produces the thrashing failure above.
- Attribution may need a model call for the harder cases, which is affordable because failures are rare, and is not obviously sound when the model that failed is asked to diagnose itself.

## Validation

Track the **proportion of outcomes that are undetermined**.

If it is very low, the settling heuristic is probably rounding ambiguous cases into one of the other two, and the value is not doing its job.
If it is very high, the sensors are not measuring what the subgoals actually need.

Track the **blame histogram** across real sessions.
A histogram dominated by one cause is either a genuine finding or evidence that attribution is defaulting, and the two are distinguishable by sampling and checking by hand.
