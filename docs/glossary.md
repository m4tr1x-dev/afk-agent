---
title: Glossary
description: The project's vocabulary, with the distinctions that matter and the words we avoid.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [glossary, reference]
requirements: []
decisions: []
generated: false
---

# Glossary

This domain overloads ordinary words.
"Task", "action" and "goal" each have three plausible meanings, and a specification that uses them loosely produces code that disagrees with itself.

Every term here has exactly one meaning in this project.
Where a term is easily confused with a neighbour, the entry says what it is **not**.
Vale enforces the substitutions at the bottom of this page, and cspell draws its vocabulary from these entries.

## Core loop

**Session**
: One continuous run of the agent against one game, from start to stop.
A session owns a context window, a set of sensors, and a perception cache.
Everything a session learns dies with it.
Not a "run" — see below.

**Run**
: One execution of the application process.
A run may contain several sessions.
Used almost exclusively in the context of logs and crash reports.

**Tick**
: One iteration of a cadence loop.
Always qualified: a *reflex tick*, a *tactical tick*, a *deliberative tick*.
An unqualified "tick" in a specification is a defect.

**Cadence**
: One of the three loops that run concurrently at different frequencies — reflex, tactical, deliberative.
See [the reasoning loop](spec/05-reasoning-loop.md).

**Frame**
: One captured image of the target window, with a timestamp.
Raw pixels.
Not an observation.

**Observation**
: The structured description of the screen that is handed to a model: detected elements, recognised text, sensor values, the previous action's outcome, and one annotated image.
Derived from a frame; never the frame itself.

**Scene class**
: A coarse label for what is currently on screen — `world`, `inventory_open`, `dialog`, `loading`, `death_screen`.
Assigned by the perception layer and used to index the perception cache and the grounding arbiter's statistics.

## Intent and execution

**Goal**
: What the user wrote.
A single natural-language string, supplied once at session start.
There is exactly one goal per session.

**Subgoal**
: A step towards the goal, proposed by the deliberative cadence, carrying a machine-checkable postcondition and a budget.
There are many subgoals per session, arranged in a tree.

**Plan**
: The current tree of subgoals.
Revised by the deliberative cadence; never by the tactical one.

**Action**
: A single entry from the action vocabulary, with its parameters resolved — `click_mark(7)`, `hold_key("w", 1500)`.
Emitted by a model, validated, then handed to the executor.
Not an input event.

**Input event**
: One synthesised keyboard or mouse event delivered to the operating system.
A single action becomes anywhere from zero to several hundred input events.

**Sustained action**
: An action with a duration, which the executor steps across many reflex ticks — holding a movement key, turning the camera, drawing a mouse path.
The distinction from an instantaneous action is architectural, not cosmetic: it is why the executor exists.

**Executor**
: The component that turns accepted actions into per-tick input state and delivers it.
The only place in the system that calls the input injection API.

**Input state**
: The authoritative record of which keys and buttons are currently held, by which subgoal, and since when.
Every safety guarantee in the project terminates here.

## Perception and grounding

**Grounding**
: Turning "the shop button" into a screen coordinate that actually hits the shop button.
The hardest problem in the project, and the subject of [its own specification](spec/18-grounding-and-verification.md).

**Mark**
: A numbered label the perception layer draws over a candidate element, so a model can refer to it by integer rather than by coordinate.
Marks are assigned per observation and are not stable across observations; the element identifier is.

**Set-of-Marks**
: The grounding path in which the model chooses a mark identifier from an enumerated list, rather than producing coordinates.
Abbreviated SoM in code, never in prose.

**Element**
: A candidate interactive region — a button, a slot, an icon — with a bounding box, a confidence, and the source that proposed it.

**Sensor**
: A named predicate over a screen region, evaluated every reflex tick without a model.
Declared by the deliberative cadence, used for verification and for stuck detection.
The mechanism that makes checking cheap.

**Postcondition**
: A sensor expression stating how the agent will know a subgoal succeeded.
Required when a subgoal is proposed.
Prose is not a postcondition.

**Verification**
: Checking, after an action, whether the expected state change occurred.
Three-valued: confirmed, refuted, or inconclusive.
The third value is load-bearing — collapsing it into either of the others makes the agent either credulous or needlessly repetitive.

**Perception cache**
: Session-scoped, in-memory structures that let the agent skip work it has already done — a template atlas of confirmed elements, a memoised mapping from scene and subgoal to action.
A latency optimisation, never a correctness dependency.
Clearing it mid-session must make the agent slower, not wrong.

## Model and context

**Context**
: The prompt assembled for a model call.
Also the only memory this project has: what the agent learns during a task lives here and nowhere else.

**Note block**
: The part of the context where the agent records facts it discovered — what a button does, where the shop is, what unlocks the next stage.
Written through a skill, compacted when it grows.

**Compaction**
: Reducing the context when it approaches the model's limit, preserving the goal, plan and notes while summarising history and discarding old observations.

**Visual token budget**
: How many tokens one image is worth to the model.
The project's main latency and quality dial, set per call.

**Skill**
: A callable capability exposed to the model with a schema — web research, note taking, sensor installation, region reading.
Distinct from an action: an action manipulates the game, a skill does not.

**Grammar**
: The constrained-decoding specification regenerated each tactical tick, which makes syntactically invalid output and references to non-existent marks structurally impossible rather than merely unlikely.

## Safety

**Panic key**
: The global hotkey that halts all input immediately and releases everything held.
Must work when the core is unresponsive.

**Foreground guard**
: The check, applied to every input event, that the target window is still the foreground window.
The primary blast-radius control.

**Dead-man switch**
: The requirement that the agent stop if the component owning the panic key stops responding.

**Guardian**
: A separate minimal process whose only job is to release all held input if the core dies.

**Budget**
: A hard limit — wall-clock, action count, or held duration — enforced below the model, which the model cannot raise.

**Blast radius**
: The set of things a misbehaving agent could affect.
Constraining it is a design activity, not a runtime check.

## Words we do not use

These are Vale substitutions and will fail the build.

| Do not write | Write | Why |
| --- | --- | --- |
| bot, botting | agent, automation | Technically wrong here — nothing is injected or hooked — and it is the word games' terms of service use for the thing we are not. |
| cheat, cheating | automation | Concedes a characterisation we do not accept. |
| hack | workaround | Imprecise, and carries the same problem. |
| exploit | defect | We report defects; we do not exploit them. |
| aimbot | automated aiming | Only appears when describing what we do not build. |

Also avoid, in specification pages: fast, robust, scalable, reasonable, several, sufficient.
Each hides a decision nobody has made.
Write the number.
