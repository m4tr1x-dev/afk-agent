---
title: Perception token economics
description: "Why you cannot send sixty frames a second to a vision model, with the arithmetic."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, perception]
requirements: []
decisions: []
generated: false
---

# Perception token economics

The obvious design is to show the model the screen and ask what to do.
It is obvious, and the arithmetic rules it out immediately.

This page does the arithmetic, because the numbers are the reason the perception pipeline looks the way it does.

## The naive version

A game runs at 60 frames per second.
Suppose each frame costs the model a moderate image budget — call it 560 tokens, which is towards the top of what a local model will accept without slowing badly.

```text
560 tokens × 60 frames = 33,600 tokens per second, of image alone
```

A local model on consumer hardware processes a few hundred to a couple of thousand tokens per second for prefill, depending on the model and the hardware.

So one second of video is somewhere between seventeen and a hundred seconds of work.

**The gap is not a factor of two. It is a factor of twenty to a hundred.**
No amount of optimisation closes that, which means the naive design is not slow — it is impossible.

## Three dials, and what each is worth

### Look less often

The largest lever by far.

Dropping from 60 Hz to 2 Hz is a thirtyfold reduction and costs almost nothing, because **most frames contain no new information**.
The player is standing still, or walking through a corridor, or the screen is a loading bar.

The question then becomes: which frames are worth looking at?
Answering that is what [change classification](../spec/04-perception.md) does, and it does it without a model, from a small signature computed on the graphics device.

### Look at less

The second lever.

The model does not need 2560×1440.
It needs enough resolution to read interface text and distinguish icons, which is far less.

Each halving of linear resolution is a fourfold reduction in tokens.
The floor is set by the smallest text the agent must read, and past that point accuracy falls off a cliff rather than degrading.

### Look more cheaply when the answer is easy

The third lever, and the subtlest.

A model with a selectable image budget can be asked the same question at 140 tokens or at 1120.
For "which third of the screen contains the shop?" the cheap budget is sufficient.
For "exactly which pixel is the button?" it is not.

Which is why the two-stage refinement in [grounding](grounding-clicks-in-pixels.md) is not merely more accurate but also *cheaper than one expensive call*, when the expensive call would have missed.

## Where the tokens actually go

For a tactical tick:

```text
system prompt and tool schemas     ~1500 tokens   invariant, cached
goal and notes                      ~300 tokens   changes rarely
plan and subgoal                    ~150 tokens   changes on replan
recent action history               ~200 tokens   changes per tick
current observation, as text        ~400 tokens   changes per tick
the image                           ~140 tokens   changes per tick
                                    -----------
                                    ~2690 tokens
```

The image is the *smallest* item on that list.

That is not an accident. It is the result of every decision on this page, and it is why the first thing anyone should check is whether the prompt prefix is actually being cached — because if it is not, that 1500-token invariant block is being re-read on every single tick and the image budget is irrelevant beside it.

## The rule that saves the most, and is easiest to break

**Old images are never resent.**

It is tempting to keep the last few frames in the conversation so the model can see what changed.
Doing so is ruinous: ten retained frames at 140 tokens each is 1400 tokens of image, and it grows without bound.

Instead, historical observations are carried as their **textual digest** — what elements were present, what changed, what the last action did.
A hundred tokens of text conveys more of what the model needs than a picture of a screen from forty minutes ago, and it does not grow.

The corollary is the placement rule: **the image goes last in the prompt**, because everything to the right of a change must be recomputed, and the image is the most expensive single thing to be on the wrong side of that line.

## The rule that is easiest to break by accident

Everything before the image must be **byte-identical** between calls.

One timestamp in the system prompt, one tick counter, one JSON object whose keys serialise in a different order, and the cached prefix is discarded.
The agent still behaves correctly.
It simply costs several times more per tick, forever, and nothing in its behaviour indicates why.

This is why prefix reuse rate is a tracked metric.
It is the only thing that catches the failure, and the failure is otherwise invisible.

## What the arithmetic produces

Working backwards from the numbers rather than forwards from the ideal:

- A fast loop at 20–30 Hz **with no model in it**, doing sensors and input.
- A tactical loop at 1–4 Hz at a small image budget.
- A deliberative loop every 10–60 s at a large one.
- Change classification that skips most frames entirely.
- Everything historical carried as text.
- An expensive look only when a cheap one was not enough.

None of that is architectural taste.
It is the shape that falls out of a factor of a hundred between what the naive design needs and what the hardware provides.
