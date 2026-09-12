---
title: ADR-0018 — Context is the only memory
description: "No knowledge base, no profiles, no retrieval over past sessions."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, context, architecture]
requirements: [CON-006, INV-CTX-001, INV-CTX-002, FR-CTX-005]
decisions: []
affects: [CON-006, INV-CTX-001, INV-CTX-002]
spec: [spec/08-context-and-research.md, spec/19-non-goals.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0018 — Context is the only memory

## Context and problem statement

The agent learns things while playing: where the shop is, what a button does, which key returns to the world.

The obvious design stores them, so that next time it does not have to find out again.

That design has a second effect that is less obvious, and this record is about which effect matters more.

## Decision drivers

- Knowledge from one game must not be applied to another where it does not hold.
- The agent must work on a game nobody has configured.
- Whatever is stored has to be identified, expired, and kept correct across game updates.
- Anything derived from the screen is the user's private data.

## Considered options

1. Context only — nothing persists beyond the session
2. A per-game knowledge store with retrieval
3. A general knowledge store, unscoped
4. Context only, with a user-initiated export and import

## Decision

**Context only.**

No knowledge base, no per-game profiles, no vector index, no retrieval over past sessions, and no store the agent can read from that outlives the task.

When the agent meets something it does not understand, it researches it and writes the result into its own context.
From that point the fact informs every decision, for as long as the task runs.
When the session ends, so does the knowledge.

### The reasoning

The benefit of a store is bounded and small: some seconds at the start of a session and a handful of model calls, saved.

The cost is unbounded, and it is not primarily implementation cost.
A store requires identifying the game reliably across updates and resolutions, expiring facts when the game patches beneath them, deciding which knowledge generalises, and handling the case where what is stored is confidently wrong.

**All of those problems disappear when knowledge is scoped to the task.**
The isolation between games is not enforced by rules that could have gaps; there is nothing left to leak.

Option 3 was never serious — knowledge from one game applied to another is not a risk to mitigate, it is the failure mode.

Option 4 is genuinely useful for a multi-session goal and is excluded for now.
It breaches the property that makes the isolation free, and once there is an import path there is a store, whatever it is called.
Recorded as an open question rather than a settled refusal.

### The corollary that needs an invariant

Session recordings are written to disk and contain frames, observations, prompts and outcomes.

`INV-CTX-002` makes them **write-only from the agent's perspective**.

That invariant exists because this is the exact point at which the decision would be undone.
There is a rich record of past sessions on disk, and feeding it back into the prompt is a small, obvious-looking change that reintroduces every problem above — silently, with no decision record, and unnoticed until the agent acts on something it learned in a different game.

## Consequences

### Good

- Cross-game leakage is impossible rather than mitigated.
- No store to version, migrate, expire or purge.
- No identity problem: the agent never has to decide whether two sessions are the same game.
- The strongest privacy position available: nothing derived from the screen outlives the session.
- A large subsystem does not exist — no embedding model, no index, no retrieval.

### Bad

- The agent re-derives things. The same game tomorrow means working out the interface again, and possibly researching the same question again.
- It cannot get better at a game the user plays constantly, which for a heavy user of one game is the feature they would most want.
- Long goals spanning several sessions restart from nothing.

### Neutral

- Within a session the agent does accumulate a perception cache and a note block. Both are scoped to the session, and clearing the cache mid-session must make the agent slower rather than wrong.

## Validation

Revisit if measurement shows that re-derivation costs a material fraction of session time — not a few seconds at the start, but minutes, repeatedly.

The honest first response even then is the export-and-import option, because it keeps the user in control of what crosses a session boundary and does not require the agent to decide what generalises.

## More information

- [What the agent remembers](../explanation/what-the-agent-remembers.md) — the argument written for readers.
- [Context and research](../spec/08-context-and-research.md) — the mechanism.
