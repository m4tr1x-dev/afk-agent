---
title: Why a local model
description: "Privacy, cost, latency and offline operation — and an honest account of the quality gap."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, model]
requirements: []
decisions: []
generated: false
---

# Why a local model

A hosted model would be better at this.

Not marginally — substantially. The largest hosted models are better at vision, better at grounding, better at planning, and better at following a complicated instruction than anything that fits on a consumer graphics card alongside a running game.

The project uses a local model anyway.
Here is the argument, including the part where the trade hurts.

## The privacy argument, which is the decisive one

This software watches your screen continuously for hours while you are not there.

Sending that to someone else's computer is not "using an API".
It is streaming a recording of everything the game shows you — including anything else that appears in that window — to a third party, under their retention policy, for as long as the session runs.

The game window may show a private message, an account name, a payment confirmation, a friend's handle.
The agent cannot tell which parts of a screen are sensitive, which means the only defensible position is that none of it leaves the machine.

"Nothing about your screen leaves the computer" is a claim that can be verified by watching the network, and it stops being true the moment there is a hosted path — including a hosted path that is off by default, because a setting that could be switched on is not the same as a capability that does not exist.

## The cost argument

The naive design sends 60 frames a second to a model.
Even after all the [economising](perception-token-economics.md), a tactical loop at 2 Hz is 7,200 image-bearing calls in an hour.

At hosted prices, an overnight session is not a rounding error.
It is enough that a user would think about whether to run it, which defeats the point of a thing that runs while you are asleep.

Local inference has a fixed cost that was paid when the graphics card was bought.

## The latency argument

A round trip to a hosted model is tens to hundreds of milliseconds of network before any thinking happens.

The tactical loop's entire budget is a few hundred milliseconds.
Spending a meaningful fraction of it on transit, with variance that depends on someone else's network, makes the loop's timing a property of the internet.

Local inference is slower per token and more predictable, and predictability is what a control loop actually needs.

## The entanglement argument

Using a hosted model means agreeing to its terms in addition to the game's.

Most hosted model providers have usage policies, and those policies have opinions about automation, about game accounts, and about what their model may be pointed at.
This project already asks a user to think carefully about one set of terms.
Adding a second party with its own view of what you may do with your own game is not a simplification.

## What the trade costs

Being straightforward about this, because a document that only lists the benefits is advocacy.

**The model is worse at grounding.** This is the single biggest quality cost, and most of [the grounding design](grounding-clicks-in-pixels.md) exists to work around it. A larger model would need less of that scaffolding.

**The model is worse at long-horizon planning.** "Get through this level" needs the agent to hold a structure in mind across hours. Smaller models drift more, which is why the plan is explicit and external rather than something the model is expected to remember.

**It needs real hardware.** A discrete graphics card with enough memory for the model *and* the game. That excludes a lot of machines, and the fallback — running the model on the processor — is slow enough to change what the agent can do.

**It competes with the game.** The most awkward consequence. The model and the game want the same memory, and an agent that makes a game stutter will be uninstalled no matter how well it plays. An entire [degradation ladder](../spec/15-performance-budgets.md) exists for this, and a hosted model would not need it.

## The compromise that is allowed

The model endpoint is a **setting**, not an assumption.

A user can point the agent at a model running on another machine on their own network.
That is still local in the sense that matters — the user controls the hardware, the data does not leave their network, and no third party's terms apply — and it solves the memory contention completely by putting the model somewhere the game is not.

This is the best available answer for a machine whose graphics card is fully occupied, and it is a configuration change rather than a different product.

What is not allowed is a hosted endpoint, and the distinction is not about network topology.
It is about whether anyone other than the user can see the screen.

## What would change our mind

If a hosted provider offered an arrangement where screen contents were provably not retained, not logged, and not used — verifiable rather than promised — the privacy argument would weaken considerably.

The cost and entanglement arguments would remain.

Absent that, this stays local.
