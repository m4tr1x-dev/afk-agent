---
title: afk-agent documentation
description: An autonomous computer-use agent that plays PC games from a natural-language prompt.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [home]
requirements: []
decisions: []
generated: false
---

# afk-agent documentation

You leave the computer.
You write what you want done — "collect money and buy upgrades in this tycoon", "get through this level", "collect carrots".
The agent watches the screen, works out what to do, and drives the mouse and keyboard until it is done or you stop it.

It has no per-game setup, no profiles, and no training.
When it meets a mechanic it does not understand, it searches the web the way an agent should, keeps what it learned in its working context for the rest of the task, and forgets it when the session ends.

!!! warning "This software does not exist yet"

    The repository contains documentation and no implementation.
    Every page in the specification describes behaviour that is being designed, not behaviour you can run.
    The banner at the top of each page says which state it is in.

## Where to go

**I want to understand what this is.**
Start with [Vision](vision.md), then [the glossary](glossary.md) — this domain overloads ordinary words like "task", "action" and "goal", and the glossary pins each of them down.

**I want to build it.**
The [specification](spec/index.md) is the instruction set.
Read [scope and goals](spec/00-scope-and-goals.md) and [requirements](spec/01-requirements.md) first; everything else assumes them.

**I want to know why it is built this way.**
The [decision records](decisions/index.md) carry the reasoning, the alternatives, and the conditions under which we would change our minds.

**I want to contribute.**
Start with the [contributor guide](contributing/index.md).
In the current phase, reviewing the specification is the most valuable contribution available — it is far cheaper to be wrong on a page than in a subsystem.

## What it does not do

Stating this early, because the category invites assumptions that do not apply here.

- It **injects no code** into any game, hooks no functions, loads no driver, and reads no game memory. It sees the screen through the same public Windows capture API a screen recorder uses, and acts through the same input API an accessibility tool uses.
- It **does not learn**. The model's weights never change. The agent does not get better at a game by playing it, and does not carry anything from one game to the next.
- It **does not hide**. Synthesised input is detectable by design, and we make no attempt to evade detection. See [terms of service and anti-cheat](explanation/anti-cheat-and-terms-of-service.md) before pointing this at an online game.
- It **runs entirely on your machine**. The model is local. Nothing about your screen leaves the computer.

## Status

Pre-implementation.
The specification is being written; no milestone has been reached.
See the [roadmap](roadmap.md) for what has to be true before code starts, and the [known-good matrix](known-good-matrix.md) for the questions that are still open.
