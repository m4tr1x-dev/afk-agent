---
title: ADR-0000 — Use MADR for architecture decision records
description: How this project records architecture decisions, and in what format.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, process]
requirements: []
decisions: []
affects: []
spec: []
evidence: [docs/contributing/adr-process.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0000 — Use MADR for architecture decision records

## Context and problem statement

This project starts with no code and a large number of architecture decisions that are expensive to reverse: which language the core is written in, how frames reach the model, how input is injected, how the agent grounds a click in a pixel.
Most of these are being made before there is any running software to validate them against, and several rest on measurements that have not been taken yet.

A decision made under those conditions is worth recording not for its conclusion but for its reasoning.
In a year the conclusion is visible in the code; the alternatives that were weighed, and the constraint that ruled them out, are visible nowhere unless we write them down.

How do we record architecture decisions, in what format, and with what lifecycle?

## Decision drivers

- The format must render on GitHub without a build step, because a decision that is hard to read is a decision nobody reads.
- It must force the alternatives to be written down, not merely permit it.
- It must have a stable public definition, so we are not maintaining a bespoke format.
- It must accommodate decisions that rest on unverified assumptions, since most of ours currently do.

## Considered options

1. MADR — Markdown Architecture Decision Records
2. Nygard's original lightweight ADR format
3. Y-statements
4. A bespoke template
5. No formal records; rely on commit messages and pull request discussion

## Decision

We use **MADR**, version 4.x, with two project-specific additions.

MADR is Markdown-native, so it renders in the repository, in pull request diffs, and in the documentation site from one source.
Its structure makes *Decision drivers* and *Considered options* mandatory sections rather than optional prose, which is precisely the part we will want later and precisely the part that gets skipped when a format does not demand it.

The two additions:

- **Front-matter fields `affects` and `spec`.** These list the requirement identifiers and specification pages the decision governs, which is what lets CI verify that every accepted decision is reachable from the specification and vice versa.
- **A mandatory Validation section.** For this project it frequently carries more value than the decision. It names the measurement or condition that would make us revisit, which converts "we think Rust is right" into something falsifiable.

Records live in `docs/decisions/`, named `NNNN-title-in-kebab-case.md`, four digits, zero-padded.
Numbers are never reused and never renumbered.
A superseded record keeps its number, gains a `superseded_by` field, and stays in place.

Status values are `proposed`, `rejected`, `accepted`, `deprecated`, and `superseded`.

## Consequences

### Good

- The alternatives are written down at the moment they are still remembered, which is the only moment they are cheap to write down.
- A reviewer sees the full reasoning in the pull request that proposes the decision, before it hardens into code.
- The `affects` and `spec` fields make the decision graph machine-checkable, so an accepted decision that no specification page references is a CI failure rather than a quiet inconsistency.

### Bad

- MADR's long form is verbose, and there is a real temptation to skip it for decisions that feel obvious. Decisions that feel obvious at the time are exactly the ones whose reasoning is forgotten fastest.
- Records accumulate. After thirty of them, finding the relevant one requires the generated index rather than browsing.

### Neutral

- We are committed to a numbering scheme for the life of the project. This is fine, but it does mean the first few numbers are spent on process decisions rather than architecture.

## Validation

This decision is working if, six months in, a contributor can answer "why is the core not written in C#?" by reading one file and without asking anyone.

It is failing if records are being written after the code rather than before it, or if the *Considered options* sections contain only the option that was chosen.
Either symptom means the format has become paperwork, and we should either enforce it in review or replace it with something lighter.

## Pros and cons of the options

### MADR

A Markdown template with defined sections for context, drivers, options, decision, consequences and validation, maintained publicly with a stable schema.

- Good, because it renders everywhere with no tooling.
- Good, because mandatory *Considered options* captures the reasoning that is otherwise lost.
- Good, because it has both a short and a long form, so trivial decisions do not need the full ceremony.
- Bad, because the long form is wordy enough that people skip it under time pressure.

### Nygard's original format

The original five-section ADR: title, status, context, decision, consequences.

- Good, because it is very short and very widely understood.
- Bad, because it has no dedicated place for the alternatives, and in practice they end up buried in the context section or omitted. For decisions with genuine trade-off matrices — inference backend, capture API, injection method — this loses the most valuable content.

### Y-statements

A single structured sentence: in the context of X, facing Y, we decided Z to achieve W, accepting that V.

- Good, because it is extremely compact and forces clarity.
- Bad, because it cannot carry a comparison table, and several of our decisions turn on measured numbers across three or four options.

### A bespoke template

- Good, because it would fit exactly what we want.
- Bad, because it is a format nobody else knows, it will drift, and the effort is better spent on the specification.

### No formal records

- Good, because it costs nothing today.
- Bad, because it costs everything later. This project's decisions are being made ahead of the evidence; without records, the first person to question one has to reconstruct the reasoning from scratch, and will probably reach a different answer for reasons that were already considered and rejected.

## More information

- [MADR](https://adr.github.io/madr/) — the template and its documentation.
- [Documentation guide](../contributing/documentation-guide.md) — front matter, voice, and the CI checks that apply to decision records.
- The template to copy is [`adr-template.md`](adr-template.md).
