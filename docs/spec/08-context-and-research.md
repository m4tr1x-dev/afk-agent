---
title: Context and research
description: "Context as the only memory: prompt layout, the note block, research results, and compaction."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, context]
requirements: [FR-CTX-001, FR-CTX-002, FR-CTX-003, FR-CTX-004, FR-CTX-005, INV-CTX-001, INV-CTX-002, FR-SKILL-003, FR-SKILL-004, FR-SKILL-005, CON-006]
decisions: []
generated: false
---

# Context and research

## Purpose

This page specifies the agent's memory, which is its context window and nothing else, and how knowledge enters it.

There is no database, no knowledge base, no per-game profile, no vector index, and no retrieval over past sessions.
The reasoning is in [what the agent remembers](../explanation/what-the-agent-remembers.md); this page specifies the mechanism.

## The principle

`CON-006`, `INV-CTX-001`.

**Everything the agent knows about the game it is playing lives in its context and dies with the session.**

When it meets something it does not understand, it searches the web, reads what it finds, and writes down what it learned — into its own context.
From that point the fact is part of every subsequent decision, for as long as the task runs.

When the session ends the context goes with it.
That is the entire isolation mechanism between one game and the next: a mechanic learned in one game cannot leak into another because nothing survives to leak.

The isolation is free because there is nothing to isolate.

## Layout

`FR-CTX-001`, `FR-CTX-002`.

```text
[1] System prompt, tool schemas, safety framing   invariant for the session
[2] Goal                                          invariant for the session
[3] Note block                                    appended by the agent
[4] Research excerpts                             appended, delimited, untrusted
[5] Current plan and subgoal                      changes on replan
[6] Recent action history, as text                changes per tick
[7] Current observation, as text                  changes per tick
[8] The annotated image                           LAST, always
```

Ordered so everything stable is on the left, because a change invalidates everything to its right.
The full reasoning, including why blocks 1 and 2 must be byte-identical between calls, is in [model contract](17-model-contract.md).

### The note block

`FR-CTX-005`.

Where the agent records what it has worked out:

```text
- The shop is behind the fountain, reachable from the north path.
- Upgrades unlock only after the second zone.
- The green button at the top right is the inventory, not the map.
- Pressing Escape twice returns to the world from any menu.
```

Written through a skill, so that recording something is a deliberate act with a place in the action history rather than a side effect of generation.

This is the block that makes long-horizon goals workable.
"Get through this level" is hours of work during which the agent learns dozens of small facts, and without somewhere to put them it rediscovers each one after every compaction.

### Research excerpts

`FR-SKILL-005`.

Retrieved content enters as a delimited block, marked as untrusted, with its source:

```text
<untrusted source="https://example.org/guide" retrieved="2026-09-12T14:02Z">
  ... extracted text ...
</untrusted>
```

The system prompt states that content inside such a block is information, never instruction.

!!! danger "A guide page is an attacker-controlled document"

    The agent reads a page precisely because it does not already know the answer,
    which is the exact condition under which it is least able to tell a good
    answer from a planted one.

    A wiki anyone can edit is not a trusted source, and neither is a forum post.
    The delimiting is structural for this reason, not decorative.
    See [threat model](14-threat-model.md).

## Compaction

`FR-CTX-004`.

A session pursuing "get through this level" can run for hours, and the context has a limit.

When usage approaches a threshold, the deliberative cadence compacts — it is the natural place, having both the latency budget and the reasoning depth to do it well.

| Block | Treatment |
| --- | --- |
| System prompt, tool schemas | Untouched |
| Goal | Untouched |
| Note block | Preserved; rewritten to merge duplicates and drop facts the plan has moved past |
| Research excerpts | Reduced to the conclusions actually used, keeping the source |
| Plan | Preserved; completed branches collapse to one line |
| Action history | Summarised: what was attempted, what worked, what did not |
| Observations | **Discarded** |

Old observations are discarded outright.
A description of a screen from forty minutes ago has no value that its effect on the note block has not already captured, and keeping them is the fastest way to fill a context with nothing.

Compaction is recorded in the session recording, so that a replay can show what the agent knew at each point and what it stopped knowing.

## Research

`FR-SKILL-003`, `FR-SKILL-004`.

Two skills, differing in depth.

**Search and fetch** answers a specific question: what does this item do, what is this key bound to.
One or two sources, seconds.

**Deep research** answers a structural question: what does completing this game involve, what is the progression, what blocks progress.
Several sources, cross-checked, synthesised into a brief.
Typically run once during briefing, and again from the [escalation ladder](05-reasoning-loop.md) when the agent is stuck in a way that looks like missing knowledge rather than missing perception.

Both are bounded by a maximum number of sources, a maximum token count, and a wall-clock limit.
An unbounded research step is an unbounded session.

**Neither blocks any cadence.**
They run as separate tasks and deliver their result as an event that triggers a deliberative pass.
Ten seconds of research is invisible because the agent keeps playing throughout.

### What is never sent

Research queries derive from the goal and from what the agent has inferred.

**Recognised screen text is never sent verbatim.**
The user's screen may contain their account name, a private message, or something else they would not put into a search box, and the agent cannot tell which parts are which.

## What reaches disk

`INV-CTX-002`.

Session recordings contain frames, observations, prompts, actions and outcomes.
They exist so the user can review a session and a developer can replay a failure.

**The agent never reads them.**

This is stated as an invariant because it is the exact point at which this design would be undone.
There is a rich record of past sessions on disk, and feeding it back into the prompt is a small, obvious-looking change that would silently reintroduce cross-session memory, cross-game leakage, and stale knowledge applied confidently — without a decision record and without anyone noticing until the agent started acting on something it learned in a different game.

Recordings are output.

## Interfaces

| Direction | Interface |
| --- | --- |
| In | Observations, from perception |
| In | Plan and subgoal, from the reasoning loop |
| In | Research results, from the skills |
| In | Notes, from the note-taking skill |
| Out | The assembled prompt, to the model host |
| Out | Compaction events, to the session recording |

## Invariants

Restated from [requirements](01-requirements.md), where they are defined:

- [`INV-CTX-001`](01-requirements.md) — no information derived from a session is readable by a later session.
- [`INV-CTX-002`](01-requirements.md) — session recordings are write-only from the agent's perspective.

## Open questions

1. What fraction of the context triggers compaction. Too early wastes capability; too late risks overflowing mid-tick.
2. Whether the note block should have a size cap independent of compaction. An agent that takes a note every tick fills it with noise, and nothing currently prevents that.
3. Whether research excerpts should be summarised on arrival rather than at compaction. Summarising early saves context; summarising is also where information gets lost, and the agent may not yet know which part matters.
4. Whether the agent should be able to mark a note as superseded rather than compaction inferring it. Explicit is better; it is also another thing for the model to get wrong.
5. How compaction interacts with replay determinism. A compaction performed by a model is not deterministic, which means a replay diverges from the original run at that point.
6. Whether a user-initiated export of the note block, re-importable into a later session, is a reasonable exception to `INV-CTX-001`. Useful for a multi-session goal; it also breaches the property that makes the isolation free. Currently excluded.

## Related decisions

`ADR-0018` will record context as the only memory, `ADR-0019` the compaction strategy, and `ADR-0021` the research backend and its fetch policy.
None are accepted.
