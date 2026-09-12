---
title: ADR-0025 — Stay in user space, and never attempt to evade detection
description: "The project's permanent position on game protection systems and terms of service."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, policy, safety]
requirements: [CON-002, FR-ACT-005, FR-SAFE-005]
decisions: [ADR-0009]
affects: [CON-002, FR-ACT-005]
spec: [spec/19-non-goals.md, spec/12-safety-and-limits.md]
evidence: [docs/explanation/anti-cheat-and-terms-of-service.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0025 — Stay in user space, and never attempt to evade detection

## Context and problem statement

This software automates play.
Some games permit that, many prohibit it, and some deploy protection systems that actively look for it.

A project in this space has to decide what it will and will not do about that, and the decision shapes the architecture rather than following from it.

Deciding it late means deciding it under pressure from a user whose account was suspended, which is the worst moment to be working out a position.

## Decision drivers

- The project should be describable honestly.
- The consequence of detection falls on the user's account, not on the project.
- Techniques that evade detection are the same techniques that protection systems treat as attacks.
- Whatever is decided must be enforceable in review, not merely stated.

## Considered options

1. Stay entirely in user space, and do not attempt to hide
2. Stay in user space, but make synthesised input harder to distinguish
3. Use a kernel component or hardware device for undetectable input
4. Refuse to support any game with a protection system

## Decision

**Option 1**, and the exclusions are permanent.

### What the project does

Captures through the platform's public window capture API.
Synthesises input through the platform's public input API, from one call site.

That is the entire surface.

### What the project never does

- Load a kernel driver, or any driver.
- Inject code into any process.
- Hook any function, anywhere.
- Read or write another process's memory.
- Modify game files.
- Draw inside a game's rendering pipeline.
- Present itself as a physical input device.
- Obscure the platform's own marking of synthesised input.
- Obfuscate its process name or presence.
- Add timing camouflage presented as realism.

### The distinction that needs stating

The executor emits curved pointer paths and varied key timings, and `FR-ACT-005` requires it.

That is **functional**: interfaces which activate on hover discard a click arriving in the same frame as the pointer, input code that debounces filters a zero-duration press, and games reading raw input clamp or reject a single implausible movement.
Without it the agent cannot operate a game.

It is documented as functional in the specification, in the non-goals and in the public explanation, so that nobody — including a future contributor — mistakes it for a stealth mechanism or tries to tune it into one.

### The terms-of-service position

Automating an online game is frequently prohibited regardless of implementation and regardless of whether anyone is harmed.
Detection and permission are different questions, and a game may prohibit this without ever noticing it.

The project does not block online use, because that line is not ours to draw for someone else's account and a block would be trivially circumvented.
What it does instead is refuse to pretend the risk is absent: the interface says so at the point of use, not only in a document nobody reads.

Four categories are refused outright as use cases and as directions for contribution: competitive multiplayer, anything converting to real money, accounts that are not the user's, and generating goods for sale.

## Consequences

### Good

- Every operation is one that ordinary uncontroversial software performs, which makes the project defensible rather than merely undetected.
- No driver to sign or maintain.
- A defect in this software cannot corrupt a game process.
- The project's description of itself is accurate, which matters more than it sounds when the alternative is a position that cannot be stated plainly.

### Bad

- Synthesised input is **detectable**, trivially, by any process that installs a low-level input hook.
- Behavioural detection — timing distributions, session length — is not addressed at all. The mitigation is not running the agent where that matters.
- A composited overlay cannot draw over a genuinely exclusive-fullscreen surface, and the technique that would fix it is excluded here.
- Some users will want the excluded features, and some contributions offering them will be declined.

### Neutral

- If a game's protection notices the agent and acts, that is the system working as intended. It is not a defect and no fix is accepted for it.

## Validation

This decision does not expire and is not subject to measurement.

It would be revisited only if the exclusions turned out to prevent the software from functioning at all — and even then the correct response is to narrow the project's scope rather than to adopt the excluded techniques.

A contribution that violates any exclusion is rejected on that basis alone, without further discussion of its merits.

## More information

- [Terms of service and anti-cheat](../explanation/anti-cheat-and-terms-of-service.md) — the long argument, written for users.
- [ADR-0009](0009-input-injection-method.md) — the injection method this constrains.
- [Non-goals](../spec/19-non-goals.md) — where the exclusions are recorded normatively.
