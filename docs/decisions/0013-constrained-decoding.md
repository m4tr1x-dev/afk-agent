---
title: ADR-0013 — Constrain model output with a grammar regenerated every tick
description: "How the agent guarantees that model output is valid and refers only to things that exist."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, model, grounding]
requirements: [FR-MODEL-003, FR-GND-003, FR-ACT-006, FR-SKILL-001]
decisions: [ADR-0014]
affects: [FR-MODEL-003, FR-GND-003]
spec: [spec/17-model-contract.md, spec/18-grounding-and-verification.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0013 — Constrain model output with a grammar regenerated every tick

## Context and problem statement

The model's output drives actions in a game.
Malformed output wastes a tick; output referring to something that is not on screen produces a click in the wrong place.

Both are common failure modes for language models asked to emit structured output, and both are recoverable — at a cost that accumulates several times a second.

## Decision drivers

- An invalid action is a wasted tick and a step towards the no-progress ladder.
- A reference to a non-existent target produces a real click somewhere real.
- Whatever is chosen runs on the tactical path, several times a second.
- Retrying is expensive and, under unchanged conditions, produces the same answer.

## Considered options

1. Free-form output, parsed and repaired
2. A schema supplied to the model, with validation afterwards
3. Constrained decoding against a fixed grammar
4. Constrained decoding against a grammar regenerated per tick

## Decision

**Option 4.** The grammar is regenerated every tactical tick and describes exactly what is valid at that moment: the available tool names, their parameter types, and **the enumerated marks and grid cells present in this observation**.

The regeneration is the whole point.
A fixed grammar guarantees well-formed output; a per-tick grammar guarantees output that refers to something real.

The model cannot name mark 7 when there is no mark 7 — not because it is unlikely to, but because that token sequence is not in the grammar.
This converts an entire class of failure from "rare, confusing, and detected only by verification" to "cannot occur".

Options 1 and 2 both leave the failure possible and handle it afterwards.
Handling it afterwards means a wasted tick each time, and at 2 Hz for eight hours that is not a rare event.

### The repair path

Some invalid combinations are semantic rather than syntactic and a grammar cannot express them — an action whose parameters conflict with the current input state, for instance.

For those: **one** structured repair request naming the problem, and on a second failure a recorded no-op and an advance of the no-progress counters.

Never retry blindly.
A model that produced invalid output once under unchanged conditions produces it again, and a blind retry loop is how an agent spends a session doing nothing.

## Consequences

### Good

- Hallucinated targets are structurally impossible, which is better than mitigated.
- Invalid tool names and malformed parameters are impossible.
- The action-validation layer has less to reject, so rejection becomes a signal rather than routine.

### Bad

- The model host must support grammar-constrained decoding. This becomes a hard requirement in the [model contract](../spec/17-model-contract.md) and narrows which runtimes are usable.
- The grammar is built every tick, which costs time on the tactical path. Small, and not zero.
- A grammar that is wrong constrains the model away from correct answers, and that failure is quiet.

### Neutral

- The grammar is generated from the same schema that defines the tools, so the two cannot drift.

## Validation

Track **grammar-valid-on-first-attempt rate** as a metric.
It should be near one. If it is not, the grammar is wrong rather than the model.

Measure grammar construction time on the tactical path.
If it consumes a meaningful share of the tick budget, cache the invariant part and regenerate only the enumerations.

Revisit if a runtime without grammar support proves substantially better in every other respect — in which case the repair path carries the load, and the [grounding decision](0014-grounding-strategy.md) needs re-examining, since its safety argument rests on this one.
