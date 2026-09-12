---
title: Action and input
description: "The action vocabulary, the executor, the input state machine, and how input reaches the game."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, action]
requirements: [FR-ACT-001, FR-ACT-002, FR-ACT-003, FR-ACT-004, FR-ACT-005, FR-ACT-006, FR-ACT-007, FR-ACT-008, NFR-ACT-001, INV-ACT-001, INV-ACT-002, INV-ACT-003, INV-LOOP-002]
decisions: [ADR-0009, ADR-0013, ADR-0035, ADR-0042]
generated: false
---

# Action and input

## Purpose

This page specifies how a decision becomes input in a game.

It is the largest document in the specification, and the reason is a single mismatch: **a model decides roughly once a second, and a game expects input every frame.**
Almost everything here exists to reconcile those two rates.

It is also where every safety guarantee in the project terminates.
The component described below is the only one in the system that synthesises input, which is what makes those guarantees checkable.

## The mismatch

A desktop computer-use agent can get away with a vocabulary of instantaneous actions: click, type, scroll.
Each completes in the moment it is issued.

Games are not played that way.
They are played by holding a key for two seconds while turning the camera, releasing at the right moment, and pressing something else during the turn.
An agent that can only click is not playing; it is operating a menu.

So the vocabulary distinguishes three kinds of action, and the executor exists to run the second and third.

| Kind | Completes | Example |
| --- | --- | --- |
| **Instantaneous** | Immediately | Click a mark, press a key, type text |
| **Sustained** | After a stated duration | Hold a key for 1500 ms, move forward for two seconds |
| **Continuous** | Over a stated duration, as a stream | Turn the camera by a given amount |

## The action vocabulary

`FR-ACT-002`.
The complete set for the first version.

### Pointer

| Action | Parameters | Kind |
| --- | --- | --- |
| `click` | target, button | Instantaneous |
| `double_click` | target, button | Instantaneous |
| `press_pointer` | target, button | Sustained, held until released |
| `release_pointer` | button | Instantaneous |
| `drag` | from target, to target, button | Sustained |
| `scroll` | target, direction, amount | Instantaneous |
| `move_pointer` | target | Sustained, duration derived from distance |

A **target** is resolved by [grounding](18-grounding-and-verification.md) and is either a mark identifier or a coordinate.
The executor never receives a target it has to interpret.

### Keyboard

| Action | Parameters | Kind |
| --- | --- | --- |
| `press_key` | key | Instantaneous, with a natural hold duration |
| `hold_key` | key, duration | Sustained |
| `release_key` | key | Instantaneous |
| `type_text` | text | Sustained, duration derived from length |
| `key_chord` | keys | Instantaneous |

`type_text` is disabled by default.
It is the route by which an agent could be induced to say something to another player, and it is also the highest-consequence thing a confused agent can do.

### Camera and movement

The actions that make this a game agent rather than an interface agent.

| Action | Parameters | Kind |
| --- | --- | --- |
| `look` | horizontal delta, vertical delta, duration | Continuous |
| `look_at` | target, duration | Continuous |
| `move` | direction, duration | Sustained |

`look` is specified in **relative deltas**, never as a destination.

### Control

| Action | Parameters | Kind |
| --- | --- | --- |
| `wait` | duration | Sustained |
| `cancel` | action identifier or subgoal identifier | Instantaneous |
| `release_all` | — | Instantaneous |
| `done` | outcome | Instantaneous |
| `abort` | reason | Instantaneous |

## Why camera control cannot be a cursor move

`FR-ACT-004`.
This deserves its own section because it is the single most common way an agent that works on desktop applications fails completely in a game.

Games that control a camera with the mouse do not read the cursor's position.
They read **relative movement deltas** from the raw input stream, and they typically capture and hide the cursor entirely.

Consequently:

- Repositioning the cursor produces **no delta at all**, one enormous delta that the game clamps or discards as implausible, or one enormous delta that the game simply applies.
  All three have been measured across four engines: two ignore the reposition entirely, and two apply it, swinging the view about a quarter of the frame width from a single event.
  The third case is the one that matters most, because it is the one that looks like it worked.
- A single large movement is frequently rejected by the game's own sanity checks.
- The cursor's absolute position is meaningless while the game holds it captured.

A `look` action is therefore decomposed by the executor into a sequence of small relative movements, emitted across reflex ticks, each within a plausible per-tick magnitude.

The measured case makes the reason sharper than the original wording did.
A reposition that a game ignores costs an action; a reposition that a game applies costs control of the camera, and the agent has no way to know which kind of game it is pointed at.

**How far a given delta turns the camera is unknown and varies per game**, because it depends on the player's own sensitivity setting.
It is measured at session start by a calibration procedure: emit a known sequence of deltas, detect when the view returns to where it started using frame correlation, and derive the ratio.
The procedure needs no model, and without it `look_at` cannot be implemented at all.

## The executor

`FR-ACT-008`.
**The only component in the system that synthesises input.**

Stated as a requirement rather than a convention because a second call site is a second place where the release guarantees can be violated, and it would be found by a user rather than by a test.

### The tick

The executor runs at the reflex rate (`NFR-ACT-001`) and does the same five things every tick:

```text
1. Read the halt flag. If set: release everything, stop. Nothing below runs.
2. Check the foreground guard. If the target is not foreground:
     release everything, pause, return.
3. Advance every active sustained action by one tick, computing the
   input state each one requires now.
4. Reconcile the desired input state against the actual held set,
   producing the minimal set of events.
5. Emit those events. Publish the held set to shared memory.
```

Step 1 comes first and returns immediately.
Step 4 is a reconciliation rather than a replay: a key that should stay held produces no event at all, which keeps the event rate proportional to change rather than to time.

Step 5 publishes the held set for the [guardian](03-container-architecture.md), which is what allows something outside this process to clean up after a crash.

### The input state

`FR-ACT-007`, `INV-ACT-002`.

The authoritative record of what is currently held:

```text
key or button -> { owner: subgoal id, since: timestamp, reason: action id }
```

Ownership (`INV-LOOP-002`) is what makes selective cancellation possible.
When the deliberative cadence abandons a subgoal, everything that subgoal was holding is released, and everything another subgoal is holding is not.

`INV-ACT-002` requires this record to match reality.
The two can diverge — an event lost, a game that captures input and never releases it — and the reconciliation in step 4 is what converges them.

### Release

`FR-ACT-001`, and the most important behaviour in this document.

**Every held input is released when any of these happens:**

- The session stops, for any reason.
- The panic key fires.
- The target window loses foreground.
- A budget is exhausted.
- The core exits, cleanly or otherwise. The [guardian](03-container-architecture.md) covers the otherwise.

Release is idempotent, ordered so that modifier keys are released last, and takes no locks that anything else can hold.

An agent that stops while holding a movement key leaves the character walking, and the user cannot undo that from outside the game.
This is the failure the whole input layer is shaped around.

### Composition and conflict

`FR-ACT-003`.

Sustained actions run concurrently when they do not contend.
Walking forward while turning the camera while holding aim is three concurrent actions and is ordinary play.

Contention is resolved before anything is emitted:

| Situation | Resolution |
| --- | --- |
| Two actions want the same key held | Permitted; the key is held once, with two owners |
| One wants a key held, another released | Rejected at validation, reported to the model |
| Two `look` actions at once | Rejected; deltas are composed by the caller, not the executor |
| A new action from a cancelled subgoal | Discarded |

### Pre-emption

When the deliberative cadence replaces the plan, actions belonging to abandoned subgoals are cancelled and their inputs released.
Actions belonging to surviving subgoals continue.

A plan change does not release everything, because releasing everything mid-traversal is itself a hazard — dropping a movement key at the wrong moment is not neutral.

## Motion synthesis

`FR-ACT-005`.

The agent does not teleport the pointer or press keys for zero time.

!!! warning "This is not evasion, and it must not become evasion"

    Three functional reasons, none of which are about detection:

    - Interfaces that activate on hover discard a click that arrives in the same frame as the pointer.
    - Input code that debounces filters a press with no measurable duration.
    - Games reading raw input reject or clamp a single implausible movement.

    Without this the agent cannot operate a game at all.

    Tuning it to be harder to detect is a [non-goal](19-non-goals.md), and synthesised input remains detectable by design.

**Pointer paths** follow a curve rather than a straight line, with a velocity profile that accelerates and decelerates, sampled at the tick rate.
Total duration scales with distance and target size — a small target legitimately takes longer to hit.

**Key hold durations** are drawn from a distribution rather than fixed, as are the gaps between events in a sequence.

**Click timing** respects the system's own double-click interval, so that two deliberate single clicks are not delivered as a double click.

## Validation

`FR-ACT-006`, `INV-ACT-003`.

Every action passes through validation before any input is synthesised:

1. **Schema.** Well-formed, known action, correct parameter types.
2. **Target resolution.** A mark that exists in the current observation; a coordinate inside the captured region.
3. **Denied keys.** `INV-ACT-003`. Window and session control combinations are never emitted, regardless of what produced the action.
4. **Rate.** Within the action and click budgets.
5. **Confirmation gates.** Actions matching a destructive pattern require human confirmation — see [safety and limits](12-safety-and-limits.md).
6. **Conflict.** Against the current input state.

A rejected action is **logged and reported back to the model** with its reason.
Silently dropping it produces a loop: the model has no way to learn that the thing it asked for cannot happen, so it asks again.

## Grounding a target

The executor resolves a target to a point, and does not decide what the target should be.

For a **mark**, it looks the identifier up in the current observation and samples a point inside the element's box, biased towards the centre.
Sampling rather than taking the exact centre absorbs small layout drift and avoids the pathological case of an element whose centre is a transparent gap.

For a **coordinate**, it converts through the chain in [perception](04-perception.md) and clips to the captured region.

If a mark cannot be resolved — the observation moved on — the action is rejected rather than guessed at.

## Interfaces

| Direction | Interface |
| --- | --- |
| In | Validated actions from the reasoning loop |
| In | The halt flag, from shared memory |
| In | Element geometry, from the current observation |
| Out | Input events, to the operating system |
| Out | The held-input set, to shared memory for the guardian |
| Out | Action lifecycle events, for verification and the recording |

## Invariants

Restated from [requirements](01-requirements.md), where they are defined:

- [`INV-ACT-001`](01-requirements.md) — no input while the target is not foreground.
- [`INV-ACT-002`](01-requirements.md) — the recorded held set matches what is actually held.
- [`INV-ACT-003`](01-requirements.md) — denied keys are never synthesised.
- [`INV-LOOP-002`](01-requirements.md) — every action belongs to exactly one subgoal.

## Open questions

1. Whether camera calibration survives the user changing their sensitivity setting mid-session. Currently nothing detects that, and every `look_at` afterwards is wrong.
2. What happens when a game captures the cursor and the agent needs to click an interface element. Many games release capture when a menu opens, but not all, and there is no detection for it.
3. Whether `move` should be expressed as a direction or as a key. A direction is more portable; a key is what the game actually reads, and the mapping between them is a per-game fact the agent is not supposed to have.
4. The per-tick magnitude cap for relative movement is a guess until measured against real games.
5. Whether the executor should refuse to hold a movement key while the scene class is a loading screen. It sounds obviously right and it is the kind of special case that accumulates.
6. How `type_text` interacts with games that consume keystrokes for hotkeys rather than text. There is no way to tell from outside which mode a game is in.

## Related decisions

`ADR-0009` records the injection method, `ADR-0013` the constrained action representation, and `ADR-0035` the sustained-action execution model.
All three are accepted.
