---
title: Requirements
description: The identified functional requirements, non-functional bounds and invariants that every other page refers to.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, requirements]
requirements: []
decisions: []
generated: false
---

# Requirements

## Purpose

This page defines every requirement identifier in the project.
Other specification pages cite them; tests annotate them; decision records list the ones they affect.
Nothing is normative unless it appears here or on a page that cites an identifier defined here.

Identifiers are assigned once and never reused.
A withdrawn requirement is marked withdrawn and left in place, because tests and issues refer to it by number.

Constraints are defined in [scope and goals](00-scope-and-goals.md) as `CON-001` through `CON-007` and are not repeated here.

**Verification methods** used in the tables below:

| Method | Meaning |
| --- | --- |
| Unit | A unit test in the owning crate or project |
| Replay | A test against a recorded session, with a deterministic expected outcome |
| Corpus | A measurement against the labelled frame corpus |
| Integration | An automated test that exercises several components together |
| Manual | A scripted manual procedure, because no automated form exists |
| Bench | A timed measurement against a stated budget |
| Lint | A check in continuous integration that makes the violation impossible to merge |

**`Lint` is not a weaker `Unit`.** A unit test shows that the code behaved
correctly once. A lint shows that the incorrect form cannot be merged at all,
which is the stronger claim and the right one for a requirement whose failure
mode is "somebody added a second one of these". It is used only where the
violation is structurally detectable rather than behavioural.

## Perception

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-PERC-001` | The system MUST capture frames from a single window identified by its window handle, not from the whole desktop. | Window-scoped capture keeps the user's other windows out of the observation entirely, which is a privacy property and not merely a convenience. | Integration |
| `FR-PERC-002` | The system MUST detect that the target window has moved, been resized, changed device scaling, or changed monitor, and rebuild its coordinate transform before the next action. | Every grounded action becomes wrong the moment this is missed, and it is silent. | Integration |
| `FR-PERC-003` | The system MUST classify each frame as unchanged, locally changed, in continuous motion, or a scene change, without invoking a model. | Invoking a vision model per frame is impossible within the latency and memory budget; classification is what makes the rest affordable. | Unit, Bench |
| `FR-PERC-004` | The system MUST extract text from the frame, with a per-region confidence. | Counters, labels and dialogue are the densest source of game state available without reading memory. | Corpus |
| `FR-PERC-005` | The system MUST propose candidate interactive elements, each with a bounding box, a confidence and the source that proposed it. | The list a model chooses from. | Corpus |
| `FR-PERC-006` | The system MUST assign each element an identifier that is stable across consecutive observations of the same scene. | Stable identifiers are what let a plan refer to something across ticks. | Unit |
| `FR-PERC-007` | The system MUST render numbered marks over candidate elements on the image given to the model. | The Set-of-Marks grounding path depends on it. | Corpus |
| `FR-PERC-008` | The system MUST evaluate declared sensors on every reflex tick. | Verification and stuck detection both read sensors, and both must be cheap enough to run continuously. | Unit, Bench |
| `FR-PERC-009` | The system MUST report when capture yields no usable frame, distinguishing an unsupported window mode from a transient failure. | A black frame is the most common first-run failure and the two causes need different fixes. | Integration |
| `FR-PERC-010` | The system SHOULD consult the platform accessibility tree when the target exposes one, and prefer its geometry over visually inferred geometry. | Where it exists it is exact, and it costs nothing to ask. Most games expose nothing; some launchers and browser-hosted games expose everything. | Integration |
| `NFR-PERC-001` | Frame classification MUST complete within its share of the reflex tick budget defined in [performance budgets](15-performance-budgets.md). | It runs on every frame; anything slower collapses the reflex cadence. | Bench |
| `NFR-PERC-002` | The system MUST NOT read back a full-resolution frame to main memory on every frame. | At high resolution and frame rate this alone would exceed the memory bandwidth budget. | Bench |
| `INV-PERC-001` | Every element in an observation has a bounding box wholly inside the captured region. | An element partly outside the region produces a click outside the window, which the foreground guard would reject, wasting a tick and corrupting the verification signal. | Unit |
| `INV-PERC-002` | Observation timestamps are monotonically non-decreasing within a session. | Replay determinism depends on it. | Unit |

## Reasoning loop

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-LOOP-001` | The system MUST run three concurrent cadences — reflex, tactical and deliberative — at independent rates. | A model decides once a second; a game needs input every frame. Separating decision from execution is the central design choice. | Integration |
| `FR-LOOP-002` | The reflex cadence MUST NOT block on a model call. | If it does, held input stops being stepped and the agent stutters in-game. | Unit |
| `FR-LOOP-003` | The system MUST maintain a plan of subgoals, revised only by the deliberative cadence. | Letting the fast loop rewrite the plan produces thrashing. | Unit |
| `FR-LOOP-004` | Each subgoal MUST carry a machine-checkable postcondition and a budget. | A subgoal whose success cannot be checked cannot be verified, retried or abandoned on evidence. | Unit |
| `FR-LOOP-005` | The system MUST detect that it is making no progress, using at least: perceptual stasis while acting, a stalled progress sensor, a repeating cycle of screen and action, and an elevated rate of unverified actions. | Each signal misses cases the others catch; any one alone produces both false positives and long silent failures. | Replay |
| `FR-LOOP-006` | On detecting no progress, the system MUST escalate through retry, alternative action, replan, research, return to a safe state, and finally stop and notify — in that order, only advancing when the previous step failed. | Without an ordered ladder the agent either gives up instantly or loops forever. | Replay |
| `FR-LOOP-007` | The system MUST trigger a deliberative pass on a scene change, on subgoal completion, on stuck detection, and on a timer. | These are the four moments where the plan is most likely to be wrong. | Unit |
| `FR-LOOP-008` | The system MUST stop when the goal's postcondition is met, when a budget is exhausted, when the target window closes, or when the user stops it. | An agent with no stop conditions never finishes. | Integration |
| `NFR-LOOP-001` | A deliberative pass MUST NOT prevent the tactical and reflex cadences from continuing to act on the previous plan. | Deliberation takes seconds; the game does not pause. | Bench |
| `INV-LOOP-001` | At most one plan is active per session. | Two plans mean two owners for the same input, which the executor cannot resolve. | Unit |
| `INV-LOOP-002` | Every action executed is attributable to exactly one subgoal. | Cancellation on replan is selective, and selectivity requires ownership. | Unit |

## Action and input

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-ACT-001` | The executor MUST release every held key and button when the session stops, the panic key fires, the target loses foreground, or the core exits. | An agent holding a movement key at shutdown leaves the character walking into a hazard, and the user cannot undo it from outside the game. | Unit, Integration, Manual |
| `FR-ACT-002` | The action vocabulary MUST distinguish instantaneous actions from actions with a duration, and the executor MUST step the latter across reflex ticks. | Games are played by holding things, not by clicking once. | Unit |
| `FR-ACT-003` | The executor MUST support concurrent sustained actions that do not contend for the same key or axis, and MUST reject those that do. | Walking forward while turning the camera while aiming is ordinary play, not an edge case. | Unit |
| `FR-ACT-004` | Camera control MUST be expressed as a sequence of small relative movements, never as a single large movement or a cursor reposition. | Games reading raw input either discard a repositioned cursor or clamp one enormous delta. Without this, camera control does not function. | Manual, Integration |
| `FR-ACT-005` | The system MUST synthesise mouse motion along a path with a non-uniform velocity profile, and MUST vary key hold durations and inter-event gaps. | Functional, not cosmetic: interfaces that activate on hover drop an instantaneous click, and input code that debounces discards a zero-duration press. | Manual |
| `FR-ACT-006` | Every action MUST be validated against the current observation and the safety rules before any input is synthesised, and a rejected action MUST be logged and reported back to the model. | A rejected action is information the model needs; silently dropping it produces a loop. | Unit |
| `FR-ACT-007` | The system MUST maintain an authoritative record of which keys and buttons are held, by which subgoal, and since when. | Every release guarantee in the project reads from it. | Unit |
| `FR-ACT-008` | Input synthesis MUST occur in exactly one component, with no other call site anywhere in the system. | A second call site is a second place the release guarantees can be violated, and it is found by a user rather than by a test. | Lint, Manual |
| `NFR-ACT-001` | The executor MUST step held input at a rate no lower than the reflex cadence. | Below it, held keys visibly stutter in-game. | Bench |
| `INV-ACT-001` | No input event is delivered while the target window is not the foreground window. | The primary blast-radius control: it is what stops the agent typing into another application. | Unit, Integration |
| `INV-ACT-002` | The set of held inputs recorded by the executor matches the set actually held by the operating system. | If they diverge, the release path releases the wrong things. | Integration |
| `INV-ACT-003` | No key on the denied list is ever synthesised, regardless of what any model produces. | Window-closing and session-switching keys are not recoverable by the user if the agent is unattended. | Unit |

## Grounding and verification

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-GND-001` | The system MUST support choosing a target by mark identifier from an enumerated candidate list. | Choosing from a list is structurally safer than producing coordinates. | Corpus |
| `FR-GND-002` | The system MUST support a target expressed as a coordinate, for cases where no candidate element exists. | Some targets are places, not elements. | Corpus |
| `FR-GND-003` | The system MUST constrain model output so that a reference to a mark that is not present is impossible rather than merely unlikely. | Turns a class of runtime failure into a class of impossible output. | Unit |
| `FR-GND-004` | The system MUST track, per scene class and per grounding path, the rate at which grounded actions are subsequently confirmed, and select the path accordingly. | Which path is better is a measurement, not a belief, and it varies by what is on screen. | Replay |
| `FR-GND-005` | The system MUST support a two-stage refinement: a low-cost pass over the whole frame to localise, then a high-cost pass over the localised region. | Two cheap calls that land beat one expensive call that misses. | Corpus, Bench |
| `FR-GND-006` | After every action, the system MUST determine whether the expected state change occurred, yielding confirmed, refuted or inconclusive. | The loop's only feedback. | Unit, Replay |
| `FR-GND-007` | On a refuted grounding, the system MUST suppress that target for a bounded number of ticks and increment the no-progress counters. | Without suppression the agent retries the same wrong target indefinitely. | Unit |
| `FR-GND-008` | The system MUST record, for every failure, an attributed cause among grounding, action choice, perception, plan, execution, and the game itself. | This histogram is the project's primary diagnostic instrument, and without it optimisation proceeds on intuition. | Replay |
| `INV-GND-001` | A verification outcome is exactly one of confirmed, refuted or inconclusive; an outcome that cannot be determined is never recorded as either of the other two. | Collapsing inconclusive into confirmed makes the agent credulous; collapsing it into refuted makes it thrash. | Unit |
| `INV-GND-002` | Every coordinate that reaches input synthesis has passed through the single defined transform chain. | Coordinate arithmetic performed anywhere else is how mixed-scaling bugs enter, and they are invisible until a user has two monitors. | Unit, Lint |

## Skills

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-SKILL-001` | The system MUST expose skills to the model with a declared schema, and MUST validate every invocation against it. | Unvalidated tool calls fail deep in the implementation rather than at the boundary. | Unit |
| `FR-SKILL-002` | The system MUST provide skills for web search, page retrieval, multi-step research, note taking, sensor declaration, and region reading. | This is the minimum set a goal like "get through this level" requires. | Integration |
| `FR-SKILL-003` | Research skills MUST run without blocking any cadence, and MUST deliver their result as an event that triggers deliberation. | Research takes seconds to tens of seconds; nothing may wait on it. | Integration |
| `FR-SKILL-004` | Research skills MUST be bounded by a maximum number of sources, a maximum token count and a wall-clock limit. | An unbounded research step is an unbounded session. | Unit |
| `FR-SKILL-005` | Retrieved web content MUST enter the context inside a delimited block marked as untrusted data, with its source. | A guide page is an attacker-controlled document. | Unit, Manual |
| `INV-SKILL-001` | No skill invocation can alter a safety setting, register a new skill, or raise a budget. | Otherwise the safety layer is reachable from model output. | Unit |

## Context

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-CTX-001` | The system MUST assemble the model prompt with its invariant portion first and the image last. | Anything to the right of a change is recomputed; putting the image last minimises that, and placing invariant content first is what allows reuse at all. | Unit, Bench |
| `FR-CTX-002` | The invariant portion of the prompt MUST be byte-identical between consecutive calls of the same cadence. | A single changed byte discards the whole cached prefix. Timestamps and counters in the system prompt are the usual cause. | Unit |
| `FR-CTX-003` | Historical observations MUST be represented in the context as text, never as retained images. | Retaining images makes a multi-turn visual loop unaffordable within the context budget. | Unit |
| `FR-CTX-004` | The system MUST compact the context as it approaches the model's limit, preserving the goal, the plan and the note block, summarising action history, and discarding old observations. | A long session otherwise terminates on context exhaustion rather than on the goal. | Replay |
| `FR-CTX-005` | The system MUST allow the model to record discovered facts into a note block that persists for the session. | This is how knowledge acquired mid-task survives compaction. | Integration |
| `INV-CTX-001` | No information derived from a session is readable by a later session. | See `CON-006`. The isolation between games depends entirely on this. | Unit, review |
| `INV-CTX-002` | Session recordings are write-only from the agent's perspective. | The single most likely way this design gets quietly undone is someone feeding recordings back into the prompt. | Review |

## Model

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-MODEL-001` | The system MUST run inference locally, against a model host it starts or a host endpoint the user configures. | `CON-003`. | Integration |
| `FR-MODEL-002` | The system MUST set the per-image visual token budget per call, and use a lower budget for the tactical cadence than for the deliberative one. | The main latency and quality dial. | Bench |
| `FR-MODEL-003` | The system MUST constrain model output to a grammar derived from the current tool schemas and the currently valid targets, regenerated each tactical tick. | See `FR-GND-003`. | Unit |
| `FR-MODEL-004` | The system MUST monitor available graphics memory and degrade — reducing context, then visual budget, then moving the deliberative model off the graphics processor — before the game is starved. | An agent that makes the game stutter is uninstalled regardless of how well it plays. | Bench, Manual |
| `FR-MODEL-005` | The system MUST verify the integrity of model weights before loading them. | Weights are downloaded; a corrupted download should fail loudly rather than produce a subtly broken agent. | Unit |
| `NFR-MODEL-001` | Tactical inference MUST complete within the budget in [performance budgets](15-performance-budgets.md) at the stated visual token budget on the reference hardware. | The tactical cadence's rate follows directly from it. | Bench |
| `INV-MODEL-001` | Model weights are never modified by the system. | `CON-005`. | Review |

## Safety

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-SAFE-001` | A global panic key MUST halt all input and release everything held, within the latency budget, and MUST function when the core is unresponsive. | The user's only guaranteed recourse. It cannot depend on the component most likely to be wedged. | Manual, Integration |
| `FR-SAFE-002` | The system MUST detect input the user generates and pause immediately, distinguishing it from its own synthesised input. | The user returning to the keyboard must get control back without finding a control. | Integration, Manual |
| `FR-SAFE-003` | A separate supervising process MUST release all held input if the core terminates unexpectedly. | Without it, a crash leaves the machine with a key held and no way to release it short of a restart. | Integration |
| `FR-SAFE-004` | The system MUST enforce a wall-clock budget, an action-rate budget, and a maximum hold duration per key. | Unattended operation means nobody notices a runaway for hours. | Unit |
| `FR-SAFE-005` | The system MUST require explicit confirmation before any action matching a destructive pattern — spending currency, trading, deleting, or changing account settings. | These are irreversible and a model mistake here costs the user something real. | Unit |
| `FR-SAFE-006` | The system MUST offer a mode that logs intended actions without executing them. | A dry run on first contact with an unfamiliar game costs nothing and reveals a great deal. | Integration |
| `FR-SAFE-007` | The system MUST display a visible indicator whenever it is capable of delivering input. | A user must never be uncertain whether the agent has hands. | Manual |
| `NFR-SAFE-001` | The panic key MUST take effect within the latency stated in [safety and limits](12-safety-and-limits.md), measured from key press to the last release event. | An interlock with an unstated bound is not an interlock. | Bench, Manual |
| `INV-SAFE-001` | No component whose behaviour is determined by a model can relax, disable or raise any safety limit. | A model may change what the agent tries to do; never what it is permitted to do. | Review, Unit |
| `INV-SAFE-002` | After any stop, panic, focus loss or crash, no input remains held. | The one failure the user cannot recover from without external help. | Integration, Manual |

## Application shell and overlay

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-GUI-001` | The overlay MUST be excluded from the system's own capture. | Otherwise the agent observes its own annotations and reasons about them — a perception feedback loop that is catastrophic and very hard to diagnose. | Integration |
| `FR-GUI-002` | The overlay MUST NOT accept input focus or intercept clicks. | Stealing focus from the game breaks both the game and the foreground guard. | Manual |
| `FR-GUI-003` | The overlay MUST display the current goal, the current subgoal, the last action, the elapsed budget, and the panic key. | The information a returning user needs in the first two seconds. | Manual |
| `FR-GUI-004` | The shell MUST be closable without stopping a running session, and the session MUST be closable without the shell. | Separate lifetimes, because the agent outlives its window. | Integration |
| `FR-GUI-005` | The shell MUST let the user review a completed session, with frames, actions and outcomes aligned. | The only way a user can find out why something went wrong. | Manual |
| `NFR-GUI-001` | The shell MUST remain responsive while the core is under load. | Freezing the only stop button under load is the worst possible time to freeze it. | Manual |
| `INV-GUI-001` | The overlay's lifetime is independent of the shell's. | The overlay owns the panic key and the visible indicator; it may not disappear with a window. | Integration |

## Configuration

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-CFG-001` | Configuration MUST be layered: built-in defaults, then user file, then session override. | Users need to change one thing without restating everything. | Unit |
| `FR-CFG-002` | Every setting MUST have a documented type, default and permitted range, and invalid values MUST be rejected at load with the file and line named. | A silently clamped setting is worse than a rejected one. | Unit |
| `FR-CFG-003` | Configuration MUST NOT be required for a first run. | Zero setup is the product claim. | Integration |
| `INV-CFG-001` | No configuration setting can disable a safety interlock. | See `INV-SAFE-001`; a configuration file is model-reachable the moment a skill can write one. | Unit |

## Observability

| ID | Statement | Rationale | Verification |
| --- | --- | --- | --- |
| `FR-OBS-001` | The system MUST emit structured events for every tick, action, verification outcome and skill invocation. | Replay, diagnosis and the blame histogram all read them. | Unit |
| `FR-OBS-002` | The system MUST be able to record a session in a form that replays deterministically against a fixed model seed. | The only way to test a stochastic agent repeatably. | Replay |
| `FR-OBS-003` | Session recordings MUST be local by default, subject to a disk quota and a retention period, and MUST be deletable in one action. | They contain images of the user's screen. | Integration |
| `FR-OBS-004` | The system MUST NOT transmit any recording, log or metric off the machine without explicit opt-in. | `CON-003` in spirit; the user's screen is the most sensitive thing here. | Review |
| `NFR-OBS-001` | Emitting observability data MUST NOT consume more than its stated share of the reflex tick budget. | Instrumentation that breaks the loop it measures is worse than none. | Bench |
| `INV-OBS-001` | A recording that was written is readable by the replay harness of the same version. | A recording nobody can read is disk usage, not observability. | Replay |

## Open questions

1. `FR-SAFE-005` needs a concrete definition of "destructive pattern" that does not depend on understanding the game. Candidate approach: detect the confirmation dialogue rather than the action, since games reliably put one in front of irreversible things. Unresolved.
2. `FR-PERC-010` assumes the accessibility probe can be bounded in time. If a busy target can make it hang past its timeout, it must move off the critical path entirely.
3. `NFR-MODEL-001` cannot be written as a number until the model spike has run on the reference hardware with a game in the background.
4. `FR-LOOP-005` lists four no-progress signals but not their thresholds, which are empirical and need the recorded corpus.
5. Whether `INV-CTX-001` should permit an explicit, user-initiated export and reimport of a note block between sessions. It would be useful and it would breach the isolation property; currently excluded.

## Related decisions

This page defines identifiers rather than citing decisions, so it lists none of its own.
Every record's `affects` field names the identifiers it touches, and the decision index carries the set that is still blocked on an experiment.
