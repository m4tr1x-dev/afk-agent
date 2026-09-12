---
title: Why the agent does not learn
description: The model's weights never change. This is a design decision with reasons, not a feature we ran out of time for.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, model, scope]
requirements: []
decisions: []
generated: false
---

# Why the agent does not learn

The obvious feature request writes itself.
The agent plays a game for six hours, succeeds at some things and fails at others, and every one of those outcomes is labelled — the verification loop already knows which actions worked.
That is a training set arriving for free.
Why not use it?

We decided not to.
The model's weights never change, there is no fine-tuning, no adapter, and no trajectory collection.
This page is the argument, because a decision like this erodes unless the reasoning is written down somewhere a future contributor will find it.

## What "learning" would actually mean here

Two things get called learning and only one of them is excluded.

The agent absolutely does acquire knowledge during a task.
It reads a guide when it does not know how a mechanic works, records what it discovered, and acts on it for the rest of the session.
That is [context, not training](what-the-agent-remembers.md), and it is central to how the thing works.

What is excluded is **weight-level learning**: taking the session's outcomes and using them to change the model, so that next week's agent is different from this week's.
That is the thing we do not do.

## The product reason

The promise is that you point this at a game nobody has ever pointed it at, write a sentence, and it works.
Not after a warm-up period.
Not after it has collected enough trajectories.
The first run on an unfamiliar game is the product.

A system that improves through training has a tempting failure mode: it can be shipped mediocre, on the expectation that it will get better.
That expectation is doing a lot of work, and it is doing it in the one place we cannot afford it — the cold start, which is the only case that distinguishes this project from a per-game script.

Excluding training removes the escape hatch.
If the agent cannot handle a new game on the first run, that is a perception problem or a prompting problem or a grounding problem, and it has to be fixed as one.

## The generality reason

Train on the sessions you have and you get a model that is better at the games you have.
There is a hard technical reason for this: the data is whatever the user played, which means it is unbalanced by construction, and a low-rank adapter fitted to an unbalanced set moves the model towards the majority of it.

You can fight this — cap the proportion of any one game, hold out a game the adapter never sees, measure transfer rather than fit.
Those mitigations work, and they are exactly what the literature on GUI agents does.
They also mean that the metric you actually care about is performance on games the adapter has never seen, which is to say: how good the model is in general.
At which point the honest question is whether a training pipeline, a data filter, an evaluation harness and an adapter lifecycle are the best available way to buy general capability, or whether that is better bought by picking a stronger base model and asking it better questions.

For a single-machine project, we think it is the second.

## The self-training reason

The label comes from our own verification loop, which means the training signal is the agent's own judgement of its own actions.
Every generation is fitted to a distribution the previous generation produced.

This narrows the policy.
The failure is not dramatic — the model does not break — it becomes more confident about doing what it already does, and stops exploring the things it was bad at, which are precisely the things worth improving.
The symptom is falling entropy in the action distribution with a flat success rate, and by the time it is visible you have shipped several generations built on it.

There is a second version of the same problem with sharper teeth.
Our reward is a sensor reading over a screen region.
If a counter can be made to go up more cheaply by something other than achieving the goal, training teaches that shortcut, faithfully and permanently.
The verification loop was designed to steer a control loop that a human is watching, not to survive being optimised against.

## The cost asymmetry

A bad adapter does not merely play worse.
It plays worse **while taking real actions in a live game**: spending currency, using consumables, walking into hazards, losing progress that cannot be recovered.

Defending against that properly needs a held-out benchmark split by episode rather than at random, statistical significance rather than a raw delta, evaluation in the deployment path rather than in the training framework, a shadow-play comparison against the incumbent, live monitoring after promotion, and a rollback.
Every one of those is necessary, and together they are a larger system than the agent.

## What we do instead

Quality comes from four places, all of which are cheaper to improve and none of which can silently degrade:

- **Perception.** Most failures are not the model choosing badly. They are the model being shown something it cannot act on. The [failure blame histogram](../spec/18-grounding-and-verification.md) exists to keep this honest, because the instinct is always to blame the model.
- **Grounding.** A model that cannot reliably point at things does not need retraining if it can be given a list to choose from instead.
- **Research.** When the agent lacks knowledge, the fix is to go and read, not to have been trained on it.
- **Verification.** An agent that reliably notices its own failures recovers from them, and recovering is worth more than being right more often.

## What would change our mind

We would revisit this if the blame histogram, from real sessions across several games, showed that grounding and action choice dominate the failures *and* that the perception layer had already been pushed as far as it goes.
That is the condition under which the model is genuinely the bottleneck, and it is a measurement rather than an intuition.

Even then the first question would be whether a stronger base model is available, not whether we should build a training pipeline.

Until that measurement exists, this stays out.
Build the measurement first.
