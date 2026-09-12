---
title: Grounding clicks in pixels
description: "Why a model's click lands in the wrong place, and what actually fixes it."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, grounding]
requirements: []
decisions: []
generated: false
---

# Grounding clicks in pixels

This is the hardest technical problem in the project, and the one most likely to decide whether it works at all.

The task sounds trivial: the model says "click the shop button", and something has to turn that into a coordinate.
Everything difficult is inside the word "something".

## Four ways the click ends up in the wrong place

### The model points badly

Ask a general-purpose vision model where a small button is and it will answer with a plausible position that is frequently a little wrong.
A few per cent of the frame width sounds like nothing and is tens of pixels, which for a 40-pixel button is a miss.

This is not a defect being fixed by better prompting.
Producing precise coordinates from an image is a specific skill, and models trained for it are substantially better at it than general models are — which is why the published benchmark numbers differ so much between the two.

### The resolution the model sees is not the resolution of the screen

The frame is downscaled before the model sees it, because [sending a full-resolution screen costs more than it is worth](perception-token-economics.md).
A coordinate in the downscaled image maps back to a range of screen pixels, and the smaller the target, the more that range matters.

A button that is 40 pixels across on a 2560-wide screen is 16 pixels across in a 1024-wide view.
Being off by one in the small image is being off by two and a half on the screen, before the model has made any error of its own.

### The coordinate spaces do not line up

There are four of them, and they disagree in ways that are invisible.

The captured surface, the window's client area, the virtual desktop, and whatever normalised space the input API wants.
Each conversion involves an offset, a scale, or both.

Under mixed display scaling — a 100% monitor and a 150% monitor, which is an extremely ordinary configuration — several of those factors differ per monitor and change when the window moves between them.

**This failure is the nastiest one on the page**, because it does not look like itself.
Clicks land near the target, the agent behaves as though the model is unreliable, and the actual cause is arithmetic. It is also untestable without a machine configured the way the user's is, which is why the project has one transform chain, in one place, with tests for several scaling configurations.

### The screen changed between looking and clicking

Perception ran, the model thought for 300 ms, and the menu animated closed.

Not solvable by better grounding. It is solvable by [verifying afterwards](../spec/18-grounding-and-verification.md) and noticing.

## What does not work

**Prompting harder.** Telling a model to be precise does not make it precise.

**One big screenshot.** Larger images cost more and do not proportionally improve small-target accuracy, because the bottleneck is the model's spatial resolution rather than the image's.

**Trusting the model's own confidence.** It is not calibrated for this, and a confidently wrong coordinate is indistinguishable from a confidently right one.

## What works

### Do not ask for coordinates

The single most effective change.

Have the perception layer find candidate elements, draw a number on each, and ask the model which number.

The model's job becomes selection from a short list, which is something it is good at.
Precision becomes the perception layer's problem, and the perception layer is measurably better at drawing a box around a button than a general model is at pointing at one.

### Make the wrong answer impossible to express

Regenerate the output grammar every tick from the marks actually on screen.

The model cannot name mark 7 if there is no mark 7.
This converts an entire class of failure — hallucinated targets — from "rare and confusing" to "cannot happen", which is a categorically better place to be than "mitigated".

### Ask twice

A cheap look at the whole frame to find the region, then an expensive look at just that region.

In published results this is worth several points on the same model, and the reason is intuitive: the second call sees the target at four or five times the effective resolution, because it is looking at a ninth of the screen with the same token budget.

Two cheap calls that land beat one expensive call that misses, because a miss costs an action, a verification cycle, and a step up the stuck ladder.

### Keep what worked

When a click is confirmed, keep a small image of what was clicked.

Finding that element again is then image matching — about a millisecond, deterministic, no model involved.
Over a session, a game's interface stops being something the model reasons about and becomes a lookup.

This is a cache and not knowledge: it lives in memory for the session, and clearing it makes the agent slower rather than wrong.

### Measure which path is working

The system tracks, per kind of screen, how often each approach produced a confirmed outcome, and prefers the one that is winning.

This is possible only because every action is verified, which is the deeper point: **verification is what makes grounding improvable.**
Without it there is no signal, and every choice between approaches is a matter of taste.

## What remains broken

Honest list.

**Targets perception never proposes.** If the detector does not see it and no template matches, mark selection cannot reach it. The fall-back is coordinates, with all the imprecision above.

**Positions that are not elements.** "Walk over there" has no button. The grid gives a coarse answer — a cell is a large fraction of the screen — and coarse-and-correct is the right trade for movement, but it is not precision.

**Small targets in dense interfaces.** An inventory grid of 64 similar icons is the worst case for every approach here at once.

**We do not know how good the model is.** There are no published grounding results for the model family this project targets. The two paths exist because the answer is genuinely unknown, and the benchmark that settles it is the first milestone.

## The shape of the answer

The system is built so the model rarely has to point at anything.

It chooses from a list the perception layer constructed, within a grammar in which invalid choices cannot be expressed, and when it does have to point, it points twice.
Every action is checked, and what worked is remembered for the rest of the session.

None of that makes the model better at grounding.
It arranges for grounding to be mostly someone else's job.
