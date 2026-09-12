---
title: Performance budgets
description: "Per-phase latency budgets, memory budgets, and what degrades first when a budget is exceeded."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, performance]
requirements: [NFR-PERC-001, NFR-PERC-002, NFR-LOOP-001, NFR-ACT-001, NFR-MODEL-001, NFR-GUI-001, NFR-OBS-001, NFR-SAFE-001, FR-MODEL-004]
decisions: []
generated: false
---

# Performance budgets

## Purpose

This page states what each part of the system is allowed to cost, and what gives way first when something exceeds it.

!!! warning "Most numbers here are targets, not measurements"

    The model latency figures in particular cannot be written until the model
    has been run on the reference hardware **with a game in memory**, which is
    open question 1 in [model contract](17-model-contract.md).

    Every unvalidated figure is marked. Do not treat one as a specification
    until it has been measured and this page updated.

## The governing constraint

The agent and the game share one machine and one graphics device, and **the game was there first**.

This is the framing for everything below.
A perception pipeline that is a little slow costs the agent some responsiveness.
A perception pipeline that starves the game of graphics memory costs the user their game, and an agent that makes a game stutter is uninstalled regardless of how well it plays.

## The reflex tick

Rate: 20–30 Hz, so a budget of 33 ms at the lower bound.

| Phase | Budget | Status |
| --- | --- | --- |
| Frame arrival and pre-pass, on the graphics device | 4 ms | Target |
| Signature read-back and change classification | 0.5 ms | Target, `NFR-PERC-001` |
| Sensor evaluation | 1 ms | Target |
| Executor step: sustained actions, reconciliation | 0.5 ms | Target, `NFR-ACT-001` |
| Safety interlocks | 0.1 ms | Target |
| Telemetry publish | 0.2 ms | Target, `NFR-OBS-001` |
| **Headroom** | **~26 ms** | |

The headroom is deliberate and large.
The reflex loop is the one path with a hard deadline, and it is the path that keeps a key held; anything that erodes it shows up in-game as stutter, which is exactly the symptom users blame on the agent.

Extraction work — text recognition, element detection, template matching — runs on a worker pool and lands asynchronously.
It is never in this budget.

## Model calls

| Call | Target | Status |
| --- | --- | --- |
| Tactical, small image cost, no extended reasoning | 150–400 ms | **Unvalidated** |
| Deliberative, large image cost, extended reasoning | 3–15 s | **Unvalidated** |
| Two-stage refinement, both passes | Under the deliberative budget | **Unvalidated** |

`NFR-MODEL-001` is written against the tactical figure, and the tactical cadence's rate follows from it.
Until it is measured, the rate in [the reasoning loop](05-reasoning-loop.md) is an intention.

Neither call is on the reflex path.
`NFR-LOOP-001` requires the other cadences to keep running while either is in flight, which is what makes a fifteen-second deliberation invisible.

## Graphics memory

The budget must hold all of this at once:

| Consumer | Notes |
| --- | --- |
| **The game** | 4–10 GiB depending on title and settings. Assume the high end. |
| Model weights | The dominant agent cost |
| Vision encoder | Loaded alongside the weights |
| Attention cache | Scales with context length; the main dial |
| Perception models | Text recognition and element detection, well under a gigabyte |
| Capture surfaces and pyramids | Small |

On the reference hardware — 24 GiB — a game at the high end leaves under half for everything else, which is why context length rather than model size is the first thing reduced.

### Degradation ladder

`FR-MODEL-004`.
Available memory is polled once a second, and when the game's headroom shrinks:

1. **Reduce context length.** Cheapest, and the attention cache is the largest variable cost.
2. **Reduce tactical image cost.** Directly trades grounding accuracy for memory and latency.
3. **Move the deliberative cadence to the processor.** Viable because only a fraction of the chosen model's parameters are active per token — an estimated single-digit to low-double-digit tokens per second, which is acceptable for a call that fires twice a minute. **Unvalidated.**
4. **Pause and tell the user.**

The ladder is ordered by what the user notices last.
A shorter context is invisible until a session runs long; a stuttering game is noticed within seconds.

## Safety latency

`NFR-SAFE-001`.

| Path | Budget | Status |
| --- | --- | --- |
| Panic key to last release event | 100 ms | **Asserted, not measured** |
| Foreground loss to release | One reflex tick | Target |
| User input detected to pause | One reflex tick | Target |
| Core exit to guardian release | 250 ms | Target |

The panic figure is the one that matters most and is the least substantiated.
It needs measuring against hotkey delivery latency under load, and it is open question 1 in [safety and limits](12-safety-and-limits.md).

## Interface responsiveness

`NFR-GUI-001`.

| Surface | Budget |
| --- | --- |
| Overlay annotation update | One reflex tick |
| Shell interaction response | 100 ms |
| Shell live view | At least 15 frames per second |

The shell performs no inference, capture or perception, so its responsiveness is a property of the process boundary rather than of tuning.

## Disk

| Consumer | Budget |
| --- | --- |
| Session recording, default | 2 GiB per session, then oldest keyframes dropped |
| Retention, default | 7 days |
| Logs | 100 MiB, rotated |

An unattended overnight session at full frame rate would fill a disk.
The quota is a default, it is user-adjustable, and purging is a single action.

## What is measured, and where

| Budget | Method | Where |
| --- | --- | --- |
| Reflex phases | Instrumented timings, 95th percentile over a session | Nightly, reference hardware |
| Model calls | Timed at the host boundary | Nightly, with a game running |
| Graphics memory | Polled, recorded per session | Every session |
| Panic latency | Instrumented from key press to last event | Nightly and manually |
| Game frame rate impact | Measured with and without the agent | Manual, per game |

The last row is the one that decides whether the project is usable, and it is the only one that cannot be automated.

Its threshold is two numbers, not one, and both come from [the vision](../vision.md):

| Statistic | Floor, against the same scene without the agent |
| --- | --- |
| Mean frame rate | 90% |
| 1% low | 80% |

Measured over ten minutes on the reference hardware, on every game in the roster.

A mean hides a stall and a stall is the symptom a player notices.
A criterion on the mean alone would pass a build that hitches every time the deliberative model runs, which is the build this design is most likely to produce.

Two details of the method matter as much as the numbers.
Runs alternate — with, without, with, without, with, without, two minutes each — because a single ordered pair measures thermal drift as well as the agent, and thermal drift alone produces 5% to 8% from nothing.
The measurement is never taken while a screenshot-driven tool is running, because that path costs frames of its own.

## When a budget is exceeded

| Budget | Response |
| --- | --- |
| Reflex tick | Log, drop optional work, then reduce the reflex rate |
| Tactical call | Reduce image cost, then reduce the tactical rate |
| Deliberative call | Reduce reasoning depth, then reduce context |
| Graphics memory | The ladder above |
| Panic latency | **A defect.** Not degraded; broken. |
| Disk | Drop oldest keyframes, then stop recording, then stop the session |

Everything degrades except safety.
A safety budget exceeded is a defect to fix, not a threshold to relax.

## Open questions

1. **Blocking.** Every model figure is unvalidated. This blocks `NFR-MODEL-001` and the cadence rates.
2. **Blocking.** The 100 ms panic budget is asserted. It may be unachievable through the ordinary hotkey path under load, in which case the mechanism needs changing rather than the number.
3. **Non-blocking.** Whether the reflex rate should adapt to the game's own frame rate. A game at 30 fps does not need a 30 Hz agent.
4. **Blocking.** Whether the graphics memory poll at one second is frequent enough. A game loading a level can allocate gigabytes faster than that.

## Related decisions

`ADR-0006` and `ADR-0007` govern the model figures, and `ADR-0024` the safety path.
None are written, and the first two are blocked on measurements taken with a game running.
