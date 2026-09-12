---
title: Failure modes and recovery
description: "The catalogue of ways an agent gets stuck, and the countermeasure for each."
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

# Failure modes and recovery

Agents fail in recognisable ways.
This is the catalogue, with what each looks like from outside and what the design does about it.

It is worth reading before watching a session go wrong, because most of these look identical from the outside — the agent is doing something, and it is not working — and the useful skill is telling them apart.

## Looping

**Looks like:** the agent does the same thing over and over. Open the menu, close the menu, open the menu.

**Why:** the observation after the action is identical to the observation before it, so the model reaches the same conclusion. It is not being stupid; it is being deterministic in a situation that gives it no new information.

**Countermeasure:** cycle detection watches for the pairing of screen state and action repeating. Plus target suppression — a refuted target is blocked for a while, which forces a different answer rather than waiting for a different input.

## Hallucinated elements

**Looks like:** the agent clicks empty space, confidently.

**Why:** the model was asked to produce a coordinate and produced a plausible one for an element that is not there.

**Countermeasure:** the strongest one in the project. The model chooses marks from a list, and the grammar is regenerated each tick from the marks actually present. A reference to something that is not on screen is not unlikely — it cannot be expressed.

When the coordinate path is used at all, verification catches the miss and the arbiter records it against that path.

## Modal blindness

**Looks like:** the agent keeps trying to act on the game while a dialog sits in front of it, apparently not seeing the dialog at all.

**Why:** two causes that need different fixes. Either the overlay is not being excluded from capture and the agent is looking at its own annotations, or the change classifier scored the dialog as a local change rather than a scene change and never triggered a full re-perception.

**Countermeasure:** capture exclusion, which is an invariant rather than a setting. And a scene-change threshold that errs towards re-perceiving: a spurious deliberative pass costs seconds, and missing a modal costs the session.

## Off-by-a-little clicks

**Looks like:** almost everything works, and occasionally a click lands just outside a button. Intermittent, resolution-dependent, worse on one monitor than another.

**Why:** the coordinate transform. Almost always the coordinate transform.

**Countermeasure:** one conversion chain, in one place, tested against several display-scaling configurations. This is the failure that most often gets misattributed to the model, and [grounding clicks in pixels](grounding-clicks-in-pixels.md) explains why it is so hard to see.

## Goal drift

**Looks like:** three hours in, the agent is doing something related to the goal but not the goal. Collecting a different resource. Exploring.

**Why:** the goal is in the context, and the context has been compacted several times. Each compaction is a lossy rewrite, and lossy rewrites accumulate.

**Countermeasure:** the goal and the plan are the two things compaction never touches. Postconditions are checkable expressions, so progress is measured against the original criterion rather than against the model's memory of it.

## False progress

**Looks like:** the counter is going up and the goal is not being achieved.

**Why:** the sensor is measuring the wrong thing. A currency counter whose region overlaps another number, a progress bar that fills for an unrelated reason.

**Countermeasure:** partially unsolved, and worth being honest about. The deliberative cadence periodically audits at a large image budget — does the screen actually show what the sensor claims? Where the objective matters, two independent sensors are better than one.

This is the failure mode that would be dangerous under training, and it is [one of the reasons there is none](why-the-agent-does-not-learn.md).

## Death-loop

**Looks like:** the agent dies, respawns, walks back to where it died, dies again.

**Why:** the plan does not model the hazard, and each respawn resets to a state where the plan looks applicable again.

**Countermeasure:** cycle detection over longer windows, and a subgoal budget that expires. The agent is not required to understand the hazard — it is required to notice that the same sequence keeps ending the same way.

## Waiting forever

**Looks like:** the agent does nothing. No error, no action.

**Why:** several causes with one appearance. The tactical loop is waiting on a model call that is not returning; a sustained action is scheduled with a duration nothing will end; the verification window never closes because the scene never settles.

**Countermeasure:** every model call has a timeout, every sustained action has a maximum duration, every key has a maximum hold time, and the heartbeat trigger fires deliberation even when nothing else has. Perceptual stasis detection catches the rest.

## Confidently wrong research

**Looks like:** the agent acts on something it read that is not true of this version of this game.

**Why:** guides go out of date, and a guide for a different version is indistinguishable from a current one.

**Countermeasure:** weak, and acknowledged. Research excerpts keep their source. Verification catches the consequence rather than the cause — the agent tries the thing, it does not work, the ladder advances. A wrong guide costs time rather than correctness.

## The overlay feedback loop

**Looks like:** marks appearing inside marks, the element count climbing every tick, behaviour degrading into nonsense.

**Why:** the overlay is not excluded from the agent's own capture, so it sees its annotations, draws annotations over them, and sees those.

**Countermeasure:** capture exclusion on every overlay window, including tooltips and flyouts, which are separate top-level windows and are the ones people forget.

It is listed separately from modal blindness because it is catastrophic rather than merely unhelpful, and because the symptom is so strange that nobody guesses the cause on the first try.

## Prompt injection from the screen

**Looks like:** the agent does something nobody asked for, shortly after text appeared in a chat region.

**Why:** another player wrote something that reads like an instruction, the text was recognised, and it entered the context.

**Countermeasure:** chat regions are excluded by default, recognised text is framed as data, safety limits sit below anything a prompt can reach, and the foreground guard confines the consequence to the game window. Residual risk is real — see [the threat model](../spec/14-threat-model.md).

## How to tell them apart

The reason failure attribution exists.

Every refuted verification is tagged with a cause: grounding, action choice, perception, plan, execution, or the game itself.
The histogram over those is what turns "the agent is not working" into "the agent's text recognition cannot read this game's font", which is a fixable statement.

The instinct in a system with a model in it is to blame the model.
Roughly half the entries above are not the model, and several of them present exactly as though they were.
