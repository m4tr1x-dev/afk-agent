---
title: Non-goals
description: What this project deliberately does not build, and why each exclusion carries weight.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, scope]
requirements: [CON-002, CON-003, CON-005, CON-006]
decisions: []
generated: false
---

# Non-goals

## Purpose

Specifications say what to build.
This one says what not to, because a project defined only by its goals accumulates the obvious adjacent features until it is a different project.

Every exclusion here is load-bearing: something else in the design depends on it being absent.
Where that is the case, the entry says what depends on it.

A non-goal is not a backlog item.
Changing one requires a decision record that addresses the reasoning below, not an issue saying it would be useful.

## The agent does not learn

**Excluded:** fine-tuning, adapters, training data collection, reinforcement from gameplay, and any mechanism by which the model's weights differ after a session from before it.
`CON-005`.

**What depends on it.**
The product claim is that the first run on an unfamiliar game works.
A system that improves through training can be shipped mediocre on the expectation that it will get better, and that expectation does its work in precisely the place the project cannot afford it — the cold start, which is the only thing distinguishing this from a per-game script.

There is a second dependency, less obvious.
Because the reward signal would come from the agent's own verification loop, training would optimise against a mechanism that was designed to steer a control loop a human is watching.
A sensor reading that can be satisfied more cheaply than the goal becomes, under training, the thing the agent learns to satisfy.

The full argument is in [why the agent does not learn](../explanation/why-the-agent-does-not-learn.md), including the measurement that would justify reconsidering.

## Nothing persists between sessions

**Excluded:** a knowledge base, per-game profiles, a vector index, retrieval over past sessions, and any store the agent can read from that outlives the task.
`CON-006`.

**What depends on it.**
The isolation between games.
Knowledge acquired while playing one game must not leak into another where the mechanic does not exist, and the reason that guarantee holds is that there is nothing left to leak.

The alternative is not merely more work, it is a different set of problems: identifying a game reliably across updates and resolutions, expiring facts when the game patches under them, deciding what generalises, and handling stored knowledge that is confidently wrong.
All of them disappear when knowledge is scoped to the task.

Session recordings are the one thing written to disk, and `INV-CTX-002` makes them write-only from the agent's perspective.
That invariant exists because this is the exact point at which the design would be quietly undone — a rich record of past sessions sitting on disk, and feeding it back into the prompt being a small change.

See [what the agent remembers](../explanation/what-the-agent-remembers.md).

## No per-game anything

**Excluded:** per-game profiles as a precondition, per-game code paths, per-game configuration required before a first run, and a curated compatibility list that gates what the agent will attempt.

**What depends on it.**
`FR-CFG-003` — configuration is not required for a first run — and the generality claim in the [vision](../vision.md).

A feature that works only because someone configured this particular game measures the person who configured it, not the agent.

This does not exclude *detecting* things about the game at runtime and adapting within the session.
That is perception, and it is encouraged.

## No code in the game's process

**Excluded:** kernel drivers, code injection, function hooking, reading or writing another process's memory, modifying game files, drawing inside a game's rendering pipeline, and presenting as a physical input device.
`CON-002`.

**What depends on it.**
The project's defensibility.
Every operation the agent performs is one that ordinary uncontroversial software performs: a screen recorder captures windows, an accessibility tool synthesises input.
A project that injected code could not make that argument whatever its intentions, and would additionally be indistinguishable, to a protection system, from the tools it is designed to stop.

It also bounds the damage a defect can do.
Software that cannot write to another process's memory cannot corrupt it.

## No evasion

**Excluded:** obscuring the injected-input marker, timing camouflage presented as realism, process name obfuscation, and any feature whose purpose is to make automated play harder to notice.

**What depends on it.**
The credibility of everything in [terms of service and anti-cheat](../explanation/anti-cheat-and-terms-of-service.md).

This one needs care, because the design contains something that looks like evasion and is not.
The agent moves the mouse along a curved path with a realistic velocity profile and varies its key hold durations, and `FR-ACT-005` requires it.
The reasons are functional: games reading raw input discard a repositioned cursor, interfaces that activate on hover drop an instantaneous click, and input code that debounces filters a zero-duration press.
Without it the agent cannot play at all.

The documentation states this explicitly so that nobody mistakes it for stealth, and so that no future contributor tries to tune it into stealth.

## No cloud inference

**Excluded:** sending frames, observations or prompts to a hosted model.
`CON-003`.

**What depends on it.**
The privacy claim, which is absolute rather than best-effort: nothing about the user's screen leaves the machine.

A model endpoint on a host the user controls, including another machine on their own network, is supported and is in fact the best answer to a machine whose graphics memory is fully occupied by the game.

## Not a general desktop automation tool

**Excluded:** operating windows other than the target, acting on the desktop, and any capability whose purpose is to control applications generally.

**What depends on it.**
`INV-ACT-001` and the whole blast-radius argument.
Constraining the agent to one window is a safety property.
Removing the constraint would remove the property, and the property is most of what makes running this unattended reasonable.

## Not for competitive multiplayer, real money, or other people's accounts

**Excluded as use cases**, and as a direction this project accepts contributions for.

Competitive multiplayer because automation there takes something directly from another person.
Anything converting to real money because the incentive structure around it is incompatible with everything else on this page.
Accounts that are not the user's because that is not automation, it is access.

These are the one group here that are ethical rather than technical.
They are in a specification because the engineering follows from them: the confirmation gates in [safety and limits](12-safety-and-limits.md) exist because of the second, and chat being off by default because of the first.

## Not portable

**Excluded:** macOS, Linux, mobile, and anything that is not Windows 11.
`CON-001`.

**What depends on it.**
Nothing, which is why this is the softest entry here.
It is a resourcing judgement rather than a principle: capture, input synthesis and window composition are platform-specific to the point where a port would be a rewrite of every layer below the reasoning loop.

If someone wants to do that work, the argument against it is weak.

## Deferred rather than excluded

Listed separately because these might arrive, and conflating them with the entries above would weaken those:

- More than one game or monitor at a time
- Gamepad output
- Exclusive fullscreen
- Audio as an input signal
- Launching or installing games
- Editing a plan mid-session
- Remote control from another device

## Open questions

1. Whether an explicit, user-initiated export and reimport of a note block between sessions is a reasonable exception to the persistence exclusion. It would be genuinely useful for a long multi-session goal, and it breaches the property that makes the isolation free. Currently excluded.
2. Whether a compatibility list that *informs* rather than gates — telling the user what to expect on a given game without changing behaviour — violates the per-game exclusion. Probably not, but it is the shape of thing that becomes a gate.

## Related decisions

`ADR-0025` will record the terms-of-service and anti-cheat posture.
The exclusions here are inputs to it and to `ADR-0002`, `ADR-0009` and `ADR-0018`.
