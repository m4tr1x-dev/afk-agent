---
title: Explanation
description: Why the system is built the way it is — the arguments, the trade-offs, and the things that are genuinely hard.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation]
requirements: []
decisions: []
generated: false
---

# Explanation

The specification says what the system does.
These pages say why, and are free to argue, compare and admit doubt in a way a specification cannot.

They are written in the first person plural and they carry opinions.
Where an explanation page and a specification page disagree, the specification is authoritative and the explanation is out of date.

## Planned pages

None of these exist yet.
They are listed here so the shape of the argument is visible, and because each one is cheapest to write in the week its governing decision is accepted — the reasoning is fresh then, and expensive to reconstruct six months later.

### The product thesis

- *Why no per-game training* — the central claim: a general agent beats a per-game script because the goal is stable and the game is not.
- *Why the agent does not learn* — why the model's weights never change, and where quality comes from instead.
- *What the agent remembers* — context is the only memory; what survives a task, what does not, and why that isolation is free.
- *Why a local model* — privacy, cost, latency, and an honest account of the quality gap.

### The hard problems

- *Grounding clicks in pixels* — the hardest technical problem here. Coordinate spaces, resolution loss, why a model's click lands 40 px off, and what actually fixes it.
- *Sustained actions explained* — a model decides once a second; a game wants input every frame. How the two are reconciled.
- *Perception token economics* — why you cannot send sixty frames a second to a vision model, with the arithmetic.
- *Failure modes and recovery* — the catalogue of ways an agent gets stuck: looping, hallucinated interface elements, modal blindness, goal drift.

### The platform

- *The Rust and C# split* — what the process boundary buys and what it costs.
- *Overlay rendering explained* — compositing, transparency, click-through, and why the overlay sometimes does not appear.
- *Research skills and game guides* — why reading a wiki helps enormously, and the prompt-injection risk that arrives with it.

### The position

- *Terms of service and anti-cheat* — what protection systems actually detect, why we stay in user space, and why that still may violate a game's terms.
