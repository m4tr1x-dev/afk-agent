---
title: What the agent remembers
description: Context is the only memory. What survives a task, what does not, and why that boundary is free rather than enforced.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, context, memory]
requirements: []
decisions: []
generated: false
---

# What the agent remembers

The agent has no database.
There is no knowledge base, no per-game profile, no vector index, no retrieval over past sessions.

What it has is a context window, and everything it knows about the game it is currently playing lives there until the task ends.

## How knowledge arrives

The agent meets something it does not understand — a mechanic, an unfamiliar interface, a goal whose meaning is not obvious from the screen.
So it does what an agent should do: it searches the web, reads what it finds, and writes down what it learned.

"Writes down" means into its own context.
From that point the fact is part of every subsequent decision, for as long as the task runs.

This is the same loop a person runs when handed an unfamiliar game.
You play until something confuses you, you look it up, and now you know.
Nobody trained you; you read a wiki page.

## How knowledge leaves

The session ends and the context goes with it.

That is the whole isolation mechanism.
When you start a different game tomorrow, the agent does not carry a mechanic from yesterday's game into one where that mechanic does not exist, because there is nothing left to carry.
We did not have to build scoping rules, or key a knowledge store by game identity, or decide how to tell two games apart, or work out when a fact learned about one game stops applying.

**The isolation is free because there is nothing to isolate.**
This is the main reason the design is shaped this way.
The alternative — persistent per-game knowledge — sounds strictly better until you write down what it requires: identifying the game reliably across updates and resolutions, expiring facts when the game patches under them, deciding what generalises and what does not, and handling the case where the stored knowledge is confidently wrong.
Each of those is a genuine problem, and all of them vanish if knowledge is scoped to the task.

## Why the context window is big enough

The models this project targets carry a large context, and a long session is not the same as a large one.
Most of what happens in an hour of play is not worth remembering: the agent walked, the screen changed, the counter went up.

What is worth keeping is small.
The goal, the current plan, a block of discovered facts, a handful of research excerpts, a short window of recent actions and their outcomes.
That is thousands of tokens, not hundreds of thousands.

The engineering problem is therefore not storage but **compaction**: as the session runs, deciding what survives, what gets summarised, and what is discarded outright.
The goal, plan and note block survive. Action history is summarised. Old observations are dropped entirely, because an image of a screen from forty minutes ago has no value that its textual summary has not already captured.

## What goes to disk, and why the agent cannot read it

Session recordings are written to disk: frames, observations, actions, outcomes.
They exist so that you can watch what the agent did and so that a developer can replay a failure.

**The agent never reads them back.**

This is worth stating explicitly, in the specification and here, because it is the exact point at which this design would be quietly undone.
Somebody, at some point, will notice that there is a rich record of past sessions sitting on disk and that feeding it back into the prompt would be a small change.
It would also reintroduce every problem listed above, silently, without a decision record and without anyone noticing until the agent started applying one game's knowledge to another.

The recordings are output, not input.

## The perception cache is not memory either

There is one thing that looks like memory and is not.

When the agent successfully interacts with an interface element and the outcome is confirmed, it keeps a small image of that element so it can find it again by direct image matching instead of asking the model.
It also memoises the action it chose for a given screen and subgoal.

Both live in memory for the current session only, both are derived from what is on screen right now rather than from anything learned, and both are pure latency optimisation.

The test is precise: **clear the cache halfway through a session and the agent must keep working, only slower.**
If clearing it changes what the agent is able to do, then it has stopped being a cache and become a dependency, and that is a defect.

There is one deliberate leak in the memoisation: a small fraction of decisions ignore the cache and ask the model anyway.
Without it, an agent that found a working route through a menu would keep taking it after the menu changed, with no mechanism for noticing.

## What this costs

The honest accounting.

The agent re-derives things.
Start the same game tomorrow with the same goal and it works out the interface again, and possibly researches the same question again.
That is real waste — some seconds at the start of a session and a handful of model calls.

It also cannot get better at a game you play often, which for a heavy user of one game is the feature they would most want.

We think that is the right trade.
The waste is bounded and small; the complexity avoided is unbounded, and the failure modes avoided — stale knowledge applied confidently, one game's assumptions leaking into another, a store that must be versioned and migrated and purged — are the kind that produce confusing behaviour rather than clean errors.

A related but separate argument, about why the model's weights never change, is in [why the agent does not learn](why-the-agent-does-not-learn.md).
