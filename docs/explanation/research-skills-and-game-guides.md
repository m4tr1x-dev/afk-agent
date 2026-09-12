---
title: Research skills and game guides
description: "Why reading a wiki helps enormously, and the prompt-injection risk that arrives with it."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, skills, security]
requirements: []
decisions: []
generated: false
---

# Research skills and game guides

Put a competent person in front of an unfamiliar game and tell them to get through the first level.
They will play for a bit, get stuck, and look it up.

That is not a failure of competence.
It is what competence looks like when the information is external.

## The problem research solves

There is a class of thing the agent cannot work out by looking.

**What the goal means.** "Get through this level" — how many stages, what ends it, what counts as through.

**What is not on screen.** The shop is behind the fountain. The upgrade requires the second zone. The boss is immune to the obvious approach.

**What the interface does not say.** The green button is the inventory, not the map. Pressing the key twice returns to the world from any menu.

**What is arbitrary.** Every game makes choices that could reasonably have gone the other way, and no amount of staring at the screen recovers them.

An agent without research does what a person without research does: it tries things until something works.
That is slow, and for some tasks it never converges.

## The measurable version

The claim is falsifiable, and the [vision](../vision.md) states it as a success criterion.

On tasks that require knowledge the agent does not have, **enabling research measurably improves the completion rate over disabling it**.

If it does not, the skills are decoration and the milestone has failed, whatever the code does.

## Why it does not slow anything down

The instinct is that a fifteen-second research call inside a real-time loop is unaffordable.

It would be, if anything waited for it.
Nothing does.

Research runs as a separate task and delivers its result as an event that triggers a deliberative pass.
Meanwhile the reflex loop keeps stepping held actions and the tactical loop keeps acting on the current plan.
The agent is playing throughout.

This is the same reason a deliberative pass taking eight seconds is invisible, and it is the payoff from [splitting the cadences](sustained-actions-explained.md).

## Two depths

**Search and fetch** answers a specific question. What does this item do. One or two sources, seconds.

**Deep research** answers a structural one. What does completing this game involve. Several sources, cross-checked, synthesised into a brief.

Deep research typically runs once during briefing, and again from [the escalation ladder](../spec/05-reasoning-loop.md) when the agent is stuck in a way that looks like missing knowledge rather than missing perception.

That second trigger is the interesting one.
Being stuck is the clearest available signal that the agent does not know something, which is exactly when looking it up is worth the cost.

## Now the problem

**A guide page is a document anyone can edit.**

The agent fetches it because it does not know the answer.
Which is precisely the condition under which it is least able to tell a good answer from a planted one.

A page could contain text addressed to an agent rather than to a reader.
Not hypothetically — this is the standard attack against any system that reads the open web and acts on what it finds.

### What the design does

**Structural delimiting.**
Retrieved content enters inside a marked block with its source attached, and the system prompt states that content inside such a block is information and never instruction.

**No skill can reach the safety layer.**
A skill cannot change a setting, raise a budget, register another skill, or synthesise input. The limits sit below anything a prompt can influence.

**The blast radius is one window.**
The foreground guard confines any resulting action to the game. The denied-key list is enforced in the executor.

**Bounded consumption.**
A maximum number of sources, a token cap, a wall-clock limit, and a per-session cap on network calls. An agent stuck in a way research cannot fix will otherwise keep reaching for research.

### What it does not do

Delimiting and instruction are mitigations, not guarantees.
No current technique makes a language model reliably immune to instructions in its input, and claiming otherwise would be dishonest.

What the design achieves is **bounding the damage**.
The worst outcome from a poisoned guide is a badly chosen action inside the game window — not a changed configuration, not input somewhere else, not a persistent effect. There is no persistence to affect, because nothing survives the session.

## The other risk: Wrong rather than hostile

More likely than an attack, and harder to detect.

Guides go out of date. A guide for a previous version reads exactly like a current one.
The agent acts on it, and the thing does not work.

There is no good defence.
What there is: sources are kept alongside excerpts, and verification catches the consequence rather than the cause — the agent tries it, it fails, the ladder advances, deliberation reconsiders.

A wrong guide costs time. It does not cost correctness, because nothing the agent reads is trusted enough to override what it sees.

## What is never sent

Queries derive from the goal and from what the agent has inferred.

**Recognised screen text is never sent verbatim.**
The window may show an account name, a private message, something the user would not type into a search box — and the agent cannot tell which parts are which.

A query still reveals what game is being played, which is inherent to looking something up, and research can be switched off entirely for anyone who would rather it did not happen.

## The honest summary

Research makes the agent substantially more capable at exactly the tasks that motivate the project — long, open-ended goals in games nobody has written a tool for.

It also opens the one attack surface that is genuinely hard to close.

The trade is accepted because the alternative is an agent that can only do what it can infer from pixels, and because the damage is bounded by a design that assumes the content is hostile rather than hoping it is not.
