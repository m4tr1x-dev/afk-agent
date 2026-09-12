---
title: How the agent loop works
description: "A narrative walkthrough of a session, from a sentence to a completed task."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, reasoning]
requirements: []
decisions: []
generated: false
---

# How the agent loop works

The [specification](../spec/05-reasoning-loop.md) states what the loop must do.
This page walks through a session so the shape is visible.

The example: *"Collect money and buy upgrades in this tycoon."*
A game the agent has never seen.

## Second zero — briefing

The agent has a sentence and a window handle. It knows nothing else.

It captures a frame and looks at it properly, at a large image budget, with reasoning enabled.
This is the most expensive call of the session and it happens once.

What it gets is orientation: a third-person view, a counter in the top right that looks like currency, a button that might be a shop, a character in the middle of the screen.

It does not know what "upgrades" means here, so it asks.
A research call goes out — a search, a page, a summary — and the agent does not wait for it. It starts on the part it can already see: the counter changes when you do something, so find out what.

Fifteen seconds later the research lands: the shop opens with a key press, upgrades cost currency, and currency comes from a repeatable activity in a particular area.
That goes into the note block and triggers a replan.

## The plan

```text
Goal: collect money and buy upgrades
├── Find the currency activity
│     post: currency_count > @currency_count    (the counter moves)
│     budget: 90 s
├── Repeat it until affordable
│     post: currency_count >= 500
│     budget: 20 min
└── Open the shop and buy
      post: shop_open, then currency_count decreased
      budget: 2 min
```

Each subgoal carries a **postcondition as an expression**, not as prose.
"The counter went up" is checkable thirty times a second by reading a screen region.
"I have found the activity" would need a model call to evaluate, and then the agent could only afford to check occasionally.

This is the most consequential thing the deliberative model produces.

## Second three — the tactical loop takes over

The rhythm changes.

Two to four times a second: look at the observation, choose one action, hand it to the executor.
Small image budget, no extended reasoning, a few hundred milliseconds.

```text
tick 41:  observation — world view, 6 marks, currency 0
          action      — move forward, 1500 ms
          verify      — the scene changed as expected: confirmed

tick 44:  observation — world view, a new mark appeared
          action      — look at mark 3
          verify      — the view turned: confirmed

tick 47:  observation — mark 3 is centred, its text reads "Collect"
          action      — press E
          verify      — currency went 0 to 5: confirmed
```

That last verification is the moment the session becomes tractable.
The postcondition fired, the subgoal is met, and the agent now knows what the activity is — without anyone having told it.

## Underneath — the reflex loop

Throughout all of this, thirty times a second, a loop with no model in it:

- Reads the panic flag.
- Checks the target window is still in front.
- Steps whatever sustained actions are running: the forward key is held, the camera deltas are emitted one small step at a time.
- Evaluates the sensors.
- Publishes what the overlay draws.

When tick 41 says "move forward for 1500 ms", the tactical loop has already moved on.
The reflex loop is what keeps the key down for the next 45 ticks and lets go at the right moment.

## Minute four — something goes wrong

The agent walks into a corner and stops making progress.

Four independent signals watch for this, and the one that fires here is **progress stall**: the currency sensor has not moved in ninety seconds while actions are still being issued.

The ladder starts:

1. **Retry** — the same thing with slightly different timing. Some failures are timing. Not this one.
2. **Alternative** — the subgoal listed a fallback. Try it. No.
3. **Replan** — a deliberative pass at the largest image budget. It looks properly and finds the problem: the character is against a wall, and the activity is behind it.
4. Plan revised, a new subgoal to get clear, and the session continues.

Rungs four through six — research, return to a safe state, stop and notify — are not reached.
They exist for the case where looking harder does not help, and the ladder's point is that each rung is tried before the next.

## Minute twenty — compaction

The context is getting long: twenty minutes of action history, several research excerpts, a growing note block.

The deliberative cadence compacts.
The goal and the plan stay. The note block is rewritten to merge duplicates. The action history becomes a summary — attempted the activity 140 times, succeeded 131, currency now 655. Old observations are discarded entirely.

Nothing important is lost, because the important things were written down as notes rather than left implicit in the history.

## Minute twenty-two — the shop

The second subgoal's postcondition fires: currency is above 500.

Deliberation moves to the third subgoal.
The note block already says the shop opens with a key press, from the research at second fifteen, so there is no rediscovery.

The agent presses it. A panel appears — a **scene change**, which forces full re-perception and a deliberative pass, because the screen is now materially different from anything the plan assumed.

New marks, new elements. The model picks an upgrade. Currency decreases. Confirmed.

## What actually did the work

The interesting parts are not the obvious ones.

**The postconditions.** Everything else is built on being able to check whether something worked, cheaply and continuously.

**The cadence split.** The deliberative pass at minute four took eight seconds. During those eight seconds the agent kept walking, because the reflex loop was still executing the previous plan. A single-rate loop would have stood still.

**The note block.** The shop key press was learned at second fifteen and used at minute twenty-two, across a compaction that discarded everything else from that period.

**Research, once.** Fifteen seconds at the start saved the agent from discovering the shop by trial and error, and it blocked nothing.

## And when the session ends

The context is discarded.

Tomorrow, the same game, the same goal: the agent works it out again, and researches it again.
That is real waste — some seconds and a handful of calls — and it is the price of [never carrying one game's assumptions into another](what-the-agent-remembers.md).
