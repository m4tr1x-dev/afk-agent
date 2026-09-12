---
title: Roadmap
description: "Milestones and their exit criteria, stated so that each one is checkable."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [roadmap]
requirements: []
decisions: []
generated: false
---

# Roadmap

No dates.
This is a sequence with exit criteria, and each milestone is finished when its criteria are met rather than when a date arrives.

The ordering is not arbitrary — it is a dependency chain, and the two places where it could most plausibly be reordered are exactly the two places where reordering would be expensive.

## M0 — Specification

**Where the project is now.**

Exit criteria:

- Every specification page is written and its Open questions section is deliberate rather than empty by omission.
- The requirement identifiers are defined and the traceability check passes.
- The blocking decision records are accepted, each preceded by its experiment.
- A competent implementer can pick up [action and input](spec/06-action-and-input.md) and build the executor without asking a question that is not already logged.

Most of this is done.
The decision records are not, because they are blocked on experiments, which is the next milestone.

## M1 — The experiments

Not a feature milestone.
Nine questions in the [known-good matrix](known-good-matrix.md), each with a defined experiment, several of which decide an architecture.

**The first one gates everything else:** does synthesised relative mouse movement reach a real game?
If it does not, the project as specified does not exist, and it is far better to learn that from a 200-line probe than from three months of building on it.

Exit criteria:

- The reachability probe works against five games across different engines.
- Camera calibration derives a usable ratio without a model.
- The model loads on the required backend, with its vision encoder, and its latency is measured **with a game running**.
- The grounding benchmark has run against a labelled corpus, and the numbers are in the specification instead of the placeholders.

## M2 — Perception, with no model at all

Capture, the pre-pass, change classification, text and element extraction, fusion, marks, sensors — visualised live in the overlay.

**This milestone contains no language model.**

The exit criterion is a human one: look at the overlay on five different games and see correct boxes on the things you would want to click.

If that does not work, no amount of model quality repairs it, and building the reasoning loop first would mean months spent debugging a planner whose real problem is that its eyes do not work.
This is the most important sequencing decision on the page.

**The overlay is delivered in this milestone**, which the section above states without the roadmap saying so.
"Look at the overlay on five different games" is not a criterion that can be met by a milestone with no overlay in it, and the interface work is otherwise listed in M7.
So M2 carries the first C# in the repository: the contract and protocol assemblies, and `AfkAgent.Overlay`.
The shell stays in M7; the overlay does not, because M3 needs a visible indicator and a panic key, and both live in it.

Exit criteria:

- Perception runs within its reflex budget on the reference hardware.
- The perception regression suite passes against the labelled corpus.
- The coordinate transform is correct under mixed display scaling, verified by test.
- The overlay draws marks over five games and does not appear in a captured frame: 1000 consecutive frames contain zero pixels of the overlay's test colour, with a tooltip and a flyout open.
- Capture-border suppression is tested in the **packaged** configuration as well as the unpackaged one, because `FR-GUI-001` may behave differently in each and discovering that in M7 means finding a perception feedback loop at the worst moment.

## M3 — Hands, safely

The executor, the input state machine, the guardian, and every interlock.

Deliberately before the model.
Safety is architecture, and an interlock added after the loop exists inherits that loop's failure modes.

Exit criteria:

- The panic key halts and releases within its budget, including while the core is artificially wedged.
- No held input survives a stop, a panic, a focus loss or a kill.
- Input never reaches a window other than the target, verified by test.
- Sustained and concurrent actions work: hold a key while turning the camera, cancel one selectively.
- The safety suite passes, with a single failure treated as a defect rather than a score.

At the end of M3 the software can see a game and act in it, safely, with nothing deciding what to do.

## M4 — The tactical loop

The model host, the prompt layout, grammar generation, the action vocabulary end to end, verification, and the arbiter.

Exit criteria:

- A stated short goal completes on one simple game.
- Grammar-constrained output is valid on first attempt at a high rate.
- Verification produces all three outcomes, and the inconclusive one is used rather than collapsed.
- The blame histogram is populated from real sessions.

## M5 — The deliberative loop

Planning, subgoals with postconditions, no-progress detection, the escalation ladder, context assembly and compaction.

Exit criteria:

- A long-horizon goal survives compaction and keeps its plan.
- The ladder recovers from induced stuck states rather than looping or giving up.
- A multi-hour session completes without exhausting its context.

## M6 — Research

The research skills, the note block, and the briefing phase.

Exit criteria:

- The agent recognises that it lacks knowledge and looks it up, unprompted.
- Retrieved content enters as untrusted data and is used as information.
- Research measurably improves completion on tasks that need knowledge the agent does not have. **If it does not, the skills are decoration** and this milestone has failed regardless of whether the code works.

## M7 — The application

The shell, session review, settings, packaging, and the memory degradation ladder.

Exit criteria:

- A user can start, watch, stop and review a session without a terminal.
- The frame-rate impact on the reference hardware is measured and within its stated bound.
- The agent degrades rather than starving the game.
- Install and first run work on a clean machine.

## M8 — Public alpha

Tutorials, how-to guides, the terms-of-service page reviewed by a human, the release process.

Exit criteria:

- The success criteria in the [vision](vision.md) are met and measured.
- Every tutorial is verified on a clean installation.
- Implemented specification pages are archived and their reference pages are authoritative.

## What could reorder this

Two places, stated because someone will propose both.

**Building the loop before perception (M4 before M2).**
Tempting, because the loop is the interesting part.
The result is a planner debugged against a perception layer that does not work, and every failure attributed to the wrong subsystem.

**Building the loop before safety (M4 before M3).**
Also tempting, because an agent without hands is not a demonstration.
The result is interlocks retrofitted onto a running loop, which is how a stop button ends up unable to stop anything that matters.

Neither reordering is accepted without a decision record that addresses the reasoning above.

## What is not on this roadmap

Training, per-game profiles, persistent memory, other platforms, gamepad output, and everything else in [non-goals](spec/19-non-goals.md).

Their absence is a decision, not a backlog.
