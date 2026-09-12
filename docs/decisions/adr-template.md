---
title: ADR-NNNN — Short title in the imperative
description: Template for architecture decision records in this project.
status: proposed
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, template]
requirements: []
decisions: []
affects: []
spec: []
evidence: []            # paths to the artefacts behind this record
supersedes: null
superseded_by: null
generated: false
---

# ADR-NNNN — Short title in the imperative

Copy this file to `docs/decisions/NNNN-title-in-kebab-case.md` and fill it in.
Take the next free number; never renumber, never reuse, and never delete a record.
Gaps in the numbering are normal and are better than renumbering.

## Context and problem statement

What situation forces a decision?
Two or three paragraphs.
State the problem in terms a reader who joins the project in a year can still understand, which means naming the constraints rather than assuming them.

End with the question in one sentence.

## Decision drivers

The things that actually decide this, in rough order of weight.
If a driver does not discriminate between the options, it is context, not a driver — move it up.

- Driver one
- Driver two

## Considered options

List every option that was seriously weighed, including the ones rejected early.
The rejected options are the part you will want in six months, when someone asks why the obvious approach was not taken.

1. Option A
2. Option B
3. Option C

## Decision

We choose **Option X**.

State it in one or two sentences, then justify it against the drivers above.
The justification belongs here, not in the option list.

## Consequences

### Good

- What this buys us.

### Bad

- What this costs us. Be specific and honest; a record with no costs listed was not a decision, it was a preference.

### Neutral

- What changes without being better or worse.

## Validation

How will we know this decision was right, and what would make us revisit it?

For a project with this much unresolved technical risk, this section is frequently worth more than the decision itself.
Name a measurement, a threshold, or an observable condition.
"We revisit this if tick latency exceeds 400 ms at the 95th percentile on the reference hardware" is useful.
"We revisit this if it turns out badly" is not.

## Pros and cons of the options

### Option A

Neutral description of the option, then:

- Good, because …
- Bad, because …

### Option B

Neutral description, then the same treatment.

## More information

Links to the specification pages this decision governs, to prior art, to benchmarks that informed it, and to any spike whose result is load-bearing.

Anything that was assumed rather than verified is listed here explicitly, marked as needing verification, so a reader can tell the measured parts from the reasoned ones.
