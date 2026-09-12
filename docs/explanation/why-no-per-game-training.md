---
title: Why no per-game training
description: "The central claim: a general agent beats a per-game script because the goal is stable and the game is not."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, product]
requirements: []
decisions: []
generated: false
---

# Why no per-game training

This is the project's central claim, and everything else follows from it.

**A general computer-use agent beats a per-game script, because the goal is stable and the game is not.**

## The three things people build instead

### A scripted macro

Record a sequence, replay it.
Cheap, fast, completely deterministic, and it encodes screen positions and timings.

It breaks when the interface moves, when a load takes longer than it did, when a pop-up appears, and when the game patches.
It also cannot respond to anything: it does not know whether the click worked, so it carries on regardless.

### A computer-vision tool with templates

Better. It finds elements rather than assuming positions, so it survives a window resize and some interface drift.

It still encodes *the game*: which templates, which sequences, what the numbers mean.
Somebody who knows the game writes it, and when the game changes they update it.
For a game nobody has written one for, there is nothing.

### A model trained on one game

Best results of the three, by a distance.
Also the most expensive: it needs trajectories from that game, a training run, and an evaluation, and at the end you have something that plays one game.

For the next game you do it again.

## What all three share

They are built by someone who already knows the game, for a game with enough players to justify the work.

Which means the thing you actually want — *this specific task, in the game I happen to be playing, which came out last week* — is the case none of them serve.
That gap is not an implementation detail of those approaches; it is what they are.

## The claim

If an agent can see a screen, operate a mouse and keyboard, and read a guide when it does not know something, it does not need to be told about the game in advance.

"Collect money and buy upgrades in this tycoon" contains everything a competent person would need.
A competent person would not require a training course on the specific tycoon before starting.
They would look at the screen, try things, read the wiki when stuck, and get on with it.

That is the whole design.

## What generality costs

Being honest about this matters, because the trade is real.

**It is worse at any specific game than a specialist.**
A trained agent for one game beats this at that game. That is not a defect to fix; it is the nature of the trade.

**It is slower.**
A macro runs at machine speed. This one looks, thinks and acts, several times a second at best.

**It re-derives things.**
Play the same game tomorrow and it works out the interface again, because [nothing persists](what-the-agent-remembers.md). Some seconds at the start of a session, wasted.

**It fails in unfamiliar ways.**
A broken script does nothing. A confused agent does something — possibly the wrong thing — which is why so much of the design is about [bounding what it can affect](../spec/12-safety-and-limits.md).

## What generality buys

**It works on the first run, on a game nobody has touched.**
This is the only property that distinguishes the project, and it is why "it gets better over time" is not an acceptable answer to a cold-start failure.

**It does not rot.**
The game patches and the interface moves, and the agent looks at the new interface. No maintenance, because there was nothing encoding the old one.

**It takes a goal rather than a procedure.**
"Get through this level" does not decompose into a sequence you could record, because the agent does not know what the level contains until it looks.

**It handles what you did not anticipate.**
A modal nobody expected, a death, an inventory that filled up. A script has no branch for these. An agent that can see is at least in a position to respond.

## Why it does not learn either

A natural next thought: keep the generality, and let the agent improve by training on what it did.

That is excluded, and the argument is long enough to have [its own page](why-the-agent-does-not-learn.md).
The short version is that a system which improves through training can be shipped mediocre on the expectation that it will get better — and the place that expectation would do its work is the cold start, which is the only thing that makes this project different from a script.

## How to tell whether the claim holds

It is falsifiable, and the [vision](../vision.md) states the threshold: a defined twenty-minute objective, on a game the agent has never seen, at least seven times in ten, with no per-game configuration of any kind, across five games in three genres.

If that fails, the claim is wrong and the honest response is to say so rather than to add a profile system.

## Where the line actually is

Adapting within a session is not per-game knowledge.

The agent notices that a game reads raw mouse input, calibrates how far a movement turns the camera, learns which button opens the inventory, and keeps a picture of it so it does not have to ask again.
All of that is **perception and caching**, it lives in the session, and it dies with it.

The excluded thing is narrower and more specific: **anything that has to exist before the first run.**
A profile, a template pack, a trained adapter, a configuration file someone wrote for this game.

The test is simple.
Delete everything the agent has, point it at a game nobody has ever pointed it at, and give it a sentence.
If it needs anything else, the claim has been quietly abandoned.
