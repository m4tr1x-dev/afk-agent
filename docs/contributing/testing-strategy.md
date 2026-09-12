---
title: Testing strategy
description: "What you assert when the component under test is stochastic."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing, testing]
requirements: []
decisions: []
generated: false
---

# Testing strategy

The hard part of testing this project is that its central component is a language model, and asking the same question twice may give two answers.

That does not make it untestable.
It means most of the interesting assertions belong somewhere other than on the model's output.

## The layers

| Layer | Asserts | Deterministic |
| --- | --- | --- |
| Unit | Pure logic: transforms, state machines, parsers, schedulers | Yes |
| Golden frame | Perception output against labelled frames | Yes |
| Replay | A recorded session produces its recorded action sequence | Yes, given a seed |
| Model-in-the-loop | Grounding accuracy and action validity over a corpus | Statistical |
| Manual | Input reaching a real game, the panic key, frame-rate impact | No |

Most of the value is in the first three, and most of the anxiety is about the fourth.

## Assert invariants, not outputs

The useful assertion is rarely that the model chose a particular action.

It is that the chosen action was **valid** — a real target, within bounds, not a denied key.
That it was **attributable** to exactly one subgoal.
That afterwards nothing was **held** that should not be.
That the verification outcome was **one of three values**, and a case that could not be determined was not recorded as either of the other two.

None of those depend on which action the model picked, and all of them catch real defects.

## Replay is the backbone

A recorded session plus a fixed seed produces the same action sequence.

That gives regression testing on a stochastic system: a change that alters behaviour appears as a diff in the action sequence, and a human decides whether the diff is an improvement.

Two places where determinism genuinely breaks, both handled rather than hidden.

**Compaction is a model call.**
Its result is stored in the recording and replayed from there rather than re-derived.

**The model host may not be bit-reproducible** across versions.
Replay is pinned to a runtime version, and a version change invalidates the expected outputs explicitly rather than producing mysterious failures.

## The suites that must never be statistical

Safety is not a score.

The release check, the foreground guard, the denied-key list and the panic path are asserted as absolutes.
**A single failure is a defect, not a degraded percentage.**

A test reporting that the panic key worked 99.7% of the time has found a bug.

## Model-in-the-loop tests

Slow, and they need a model, so they run nightly rather than on every pull request.

They report an **interval**, not a point.
At a few hundred samples the standard error on a proportion near one half is a couple of percentage points, so a two-point improvement is one standard error and not evidence.

A change that does not clear the interval is noise.
Treating it as a result is how a project walks slowly downhill while believing each step was an improvement.

## The suite that looks boring and catches the most

Coordinate transforms.

Exact expected mappings under several display-scaling and monitor configurations.

The defect it catches is invisible in ordinary testing — clicks land near the target rather than on it — and it presents as an unreliable model rather than as arithmetic.

## Manual tests

Some things cannot be automated and must still be checked:

- Input actually reaching a specific game.
- The panic key under real load.
- Frame-rate impact with the agent running.
- The overlay over a genuinely fullscreen game.

These are written as scripts a person follows, with the result recorded per release.
"Someone checked it once" is not a record.

## Requirement identifiers in tests

A test that covers a requirement carries its identifier.

The traceability report is generated from those markers, and continuous integration fails when a requirement marked implemented has no covering test.

That is the mechanism that keeps the specification honest as the code arrives.
