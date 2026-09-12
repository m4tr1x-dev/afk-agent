---
title: Sustained actions explained
description: "A model decides once a second; a game wants input every frame. How the two are reconciled."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, action]
requirements: []
decisions: []
generated: false
---

# Sustained actions explained

Here is the mismatch the whole action layer exists to solve.

A vision model answers in a few hundred milliseconds at best.
A game reads input every frame, sixty times a second.

An agent that acts only when the model answers is present for about one frame in twenty, and absent for the other nineteen.

## Why this does not come up on the desktop

Computer-use agents for desktop applications get away with a vocabulary of instantaneous actions: click, type, scroll.
Each one completes in the moment it is issued, and nothing is expected of the agent between them.

A dialog does not care that you took two seconds to decide.
The button is still there.

Games are not like that.

## What playing actually requires

Consider the simplest thing a player does: walk forward for two seconds while turning to the right.

That is not one action.
It is 120 frames during which a key must be held down and a stream of small mouse movements must arrive.
Stop being present for any of those frames and the character stops walking, or the turn stutters, or both.

Now add aiming, which is a second held button, and a jump in the middle, which is an instantaneous press that must not disturb either.

There is no way to express this as a sequence of instantaneous actions issued at one per second.

## The resolution: Decide slowly, execute continuously

Split the two.

**The model issues intentions with durations.**
Not "press W" but "move forward for 1500 ms".
Not "move the mouse to here" but "turn right by this much over 400 ms".

**A fast loop turns intentions into per-frame input state.**
It runs at 20 to 30 Hz, contains no model, and does the same small job every tick: work out what should be held right now, compare it against what is held, emit the difference.

The model decides; the executor is present.

```text
model, 1 Hz      →  "hold W for 1500 ms"
                    "look right 300 units over 400 ms"

executor, 30 Hz  →  tick 1: press W. emit mouse delta (25, 0).
                    tick 2: W already held — emit nothing. delta (25, 0).
                    tick 3: W already held. delta (25, 0).
                    ...
                    tick 12: delta stream finished.
                    ...
                    tick 45: 1500 ms elapsed. release W.
```

Note tick 2.
The executor reconciles rather than replays, so a key that should stay held produces no event at all.
The event rate is proportional to change rather than to time.

## Why camera control forces this

Even if you did not care about holding keys, camera control alone would require it.

Games that turn the view with the mouse do not read where the cursor is.
They read **relative movement deltas**, and they usually capture the cursor so its position is meaningless.

So moving the cursor to a position produces no delta at all.
And one enormous delta — the movement you would need to turn ninety degrees in a single event — gets clamped or discarded by the game's own sanity checking.

The only thing that works is a stream of small plausible movements, which means being present across many frames, which is the executor.

There is a second consequence: **how far a given delta turns the camera is unknown**, because it depends on a sensitivity setting the player chose.
The agent measures it at session start by emitting a known sequence and detecting when the view comes back around.
No model involved, and without it "look at that thing" cannot be implemented at all.

## What this buys beyond keeping up

Once the executor exists, several things become natural that were not.

**Concurrency.** Walking, turning and aiming at once is three sustained actions that do not contend, and they compose.

**Selective cancellation.** Every held input records which subgoal owns it. When the plan changes, that subgoal's inputs are released and the others are not — because dropping every key mid-traversal is its own hazard.

**A place for every safety guarantee.**
The executor is the only component that synthesises input, and it checks the halt flag and the foreground guard every tick, before anything else. Every promise the project makes about stopping terminates in that one loop.

## And one obligation

**Something is holding a key down, and it must always let go.**

Stop the session, lose focus, hit the panic key, crash — in every case, everything held must be released.

This is the failure that matters most in input automation, and it is not hypothetical.
An agent that stops while holding a movement key leaves the character walking forward into whatever is ahead, and the user cannot release that key from outside the game.

It is why there is an authoritative record of what is held, why release is idempotent and lock-free, and why [a separate process](../spec/03-container-architecture.md) exists whose only job is to release everything if the core dies.

The executor is what makes the agent able to play.
The release path is what makes it safe to leave running.
