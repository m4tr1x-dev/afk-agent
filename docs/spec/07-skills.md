---
title: Skills
description: "What a skill is, how skills are declared and dispatched, and the built-in set."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, skills]
requirements: [FR-SKILL-001, FR-SKILL-002, FR-SKILL-003, FR-SKILL-004, FR-SKILL-005, INV-SKILL-001]
decisions: []
generated: false
---

# Skills

## Purpose

A skill is a capability the model can invoke that is not an action in the game.

The distinction is the point of this page.
An **action** manipulates the game through synthesised input, is bounded by the safety interlocks, and is the subject of [action and input](06-action-and-input.md).
A **skill** does something else: looks something up, records a note, declares a sensor, reads a region of the screen.

Keeping them apart means the safety layer has one narrow thing to guard, and skills do not have to be reasoned about as though they could press keys.

## What a skill declares

`FR-SKILL-001`.

```text
id            stable identifier
version       schema version
name          short display name
description   what the model is shown — see below
parameters    JSON Schema
returns       JSON Schema
permissions   network, filesystem, clipboard, none
cost          latency class: immediate, seconds, tens of seconds
```

**The description is a prompt.**
It is what the model reads when deciding whether this skill is the right one, and it is the only thing standing between a capable skill and a model that never invokes it.
Write it as an instruction to a competent colleague who has not seen the code: what it does, when to reach for it, and what it costs.

Every invocation is validated against the parameter schema before dispatch.
An unvalidated tool call fails somewhere inside the implementation, at a point where the error message is about a missing field rather than about a model that asked for the wrong thing.

## The built-in set

`FR-SKILL-002`.

### Knowledge

| Skill | Does | Cost |
| --- | --- | --- |
| `web_search` | Search, returning ranked results with snippets | Seconds |
| `web_fetch` | Retrieve one page and extract its readable content | Seconds |
| `deep_research` | Decompose a question, consult several sources, cross-check, synthesise | Tens of seconds |

Specified in [context and research](08-context-and-research.md).

### Memory

| Skill | Does | Cost |
| --- | --- | --- |
| `take_note` | Append a discovered fact to the note block | Immediate |
| `revise_notes` | Rewrite the note block, merging or dropping entries | Immediate |

There is no `recall`.
Notes are in the context already; a skill to retrieve them would be a skill to read what the model can already see.

### Perception

| Skill | Does | Cost |
| --- | --- | --- |
| `read_region` | Text recognition over one region at full resolution | Immediate |
| `describe_region` | A visual pass over a cropped region at a large image cost | Seconds |
| `declare_sensor` | Install a named sensor expression | Immediate |
| `list_sensors` | Report current sensor values | Immediate |

`describe_region` is the second stage of the two-stage refinement in [grounding](18-grounding-and-verification.md).

### Planning

| Skill | Does | Cost |
| --- | --- | --- |
| `set_plan` | Replace the plan tree | Immediate |
| `set_subgoal` | Move to a subgoal, with its postcondition and budget | Immediate |
| `declare_stuck` | State that the current approach is not working | Immediate |
| `ask_user` | Pause and ask a question the agent cannot resolve | Blocks until answered |

`declare_stuck` matters because the automatic detectors in [the reasoning loop](05-reasoning-loop.md) are heuristics over observable signals.
A model that can see it is going nowhere should be able to say so rather than waiting to be detected.

`ask_user` is the honest end of the road.
An agent that is confused and unattended should stop, not guess.

## Dispatch

```text
model output
  -> grammar-constrained, so the skill name is valid by construction
  -> parameter schema validation
  -> permission check
  -> registry lookup
  -> execute, with a timeout from the declared cost class
  -> result into the context
```

A skill that exceeds its timeout is cancelled and reports a timeout.
It does not get to hold a cadence open.

### Skills do not block cadences

`FR-SKILL-003`.

Anything in the seconds or tens-of-seconds class runs as a separate task.
The result arrives as an event that triggers deliberation.

The agent keeps playing while research runs, which is the only reason research is affordable inside a real-time loop at all.

### Bounds

`FR-SKILL-004`.

| Bound | Applies to |
| --- | --- |
| Sources per research call | `deep_research` |
| Tokens per result | All retrieval skills |
| Wall-clock per invocation | All skills |
| Invocations per session | Network skills |
| Concurrent invocations | All skills |

The per-session cap on network skills exists because an agent that is stuck in a way research cannot fix will keep reaching for research.

## Permissions

`INV-SKILL-001`.

Each skill declares what it may touch, and the declaration is enforced at dispatch rather than trusted.

**No skill may:**

- Alter a safety setting or raise a budget.
- Register a skill, or change one's permissions.
- Synthesise input. That is the executor's exclusive responsibility (`FR-ACT-008`).
- Read a session recording, or anything from a previous session (`INV-CTX-001`).
- Write outside the session's own directory.

The first of these is the load-bearing one.
If a skill could change a safety limit, then the safety limits would be reachable from model output, and [safety and limits](12-safety-and-limits.md) would be advisory.

## Presenting skills to the model

The catalogue costs tokens in the invariant part of the prompt, and it is read on every call.

The tactical cadence is shown a **reduced catalogue**: the skills that make sense at that rate, which excludes everything in the tens-of-seconds class.
The deliberative cadence is shown all of them.

This is not only economy.
Showing a fast loop a skill it cannot afford invites it to invoke one, and then the fast loop is waiting on research.

## Third-party skills

Out of scope for the first version.

The mechanism would be straightforward and the security position is not: a skill is code, and a model chooses when to run it.
Before this is reopened, [the threat model](14-threat-model.md) needs a section on it and the execution model needs a sandbox, which is `ADR-0020`.

## Interfaces

| Direction | Interface |
| --- | --- |
| In | Validated invocations from the reasoning loop |
| Out | Results into the context |
| Out | Sensor declarations to perception |
| Out | Plan changes to the reasoning loop |
| Out | Invocation events to the session recording |

## Open questions

1. Whether `ask_user` should have a timeout that converts to stopping. An agent blocked overnight on a question is not obviously better than one that stopped.
2. Whether `declare_sensor` should validate that the expression evaluates meaningfully before installing it. A sensor that is always false is indistinguishable from a subgoal that never succeeds.
3. How skills that the tactical cadence cannot see are surfaced when they are the right answer. Currently the deliberative pass has to notice.
4. Whether `web_search` and `web_fetch` should be exposed at all, or only `deep_research`. The lower-level pair is more flexible and is also how an agent ends up reading forums for twenty minutes.
5. What happens when two subgoals declare sensors with the same name.

## Related decisions

`ADR-0020` will record the skill format and execution model, and `ADR-0021` the research backend.
Neither is accepted.
