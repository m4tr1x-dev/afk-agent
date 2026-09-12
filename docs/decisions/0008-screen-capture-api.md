---
title: ADR-0008 — Capture the game through Windows Graphics Capture
description: Which capture route the perception pipeline uses, chosen from a measured comparison rather than from reputation, with the failure signatures FR-PERC-009 needs.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, perception, capture]
requirements: [FR-PERC-006, FR-PERC-009]
decisions: [ADR-0025]
affects: [crates]
spec: [docs/spec/04-perception.md]
evidence:
  - experiments/07-capture/findings.md
  - experiments/07-capture/results/2026-09-12-routes.txt
  - experiments/07-capture/results/2026-09-12-overlay.txt
  - experiments/07-capture/src/measure.rs
supersedes: null
superseded_by: null
generated: false
---

# ADR-0008 — Capture the game through Windows Graphics Capture

## Context and problem statement

Perception begins with a frame, and the reflex loop has 33 milliseconds to spend.
Whatever produces that frame is the first item in every tick's budget and the only component that touches the game at all.

`CON-002` narrows the field before performance does: no injection into the game's process, no hooking of its functions, no drawing inside its renderer.
That excludes the approach most capture tools take and leaves the platform's own interfaces.

The harder half of the question is not which route is fastest.
`FR-PERC-009` asks the system to tell an unsupported window mode apart from a transient failure, and the routes that fail here mostly **succeed** while returning something useless: a black bitmap, a surface that never updates, a frame of whatever window is in front. A choice made without those signatures written down leaves the requirement unimplementable.

Which route does the capture layer use, and how does each one fail?

## Decision drivers

- **Cost per frame**, against a 33 millisecond tick that also has to fit the pre-pass, classification, sensors and the executor.
- **Whether it sees a swap chain at all.** Every game renders through one.
- **Whether its failures are distinguishable.** A route that fails silently is worse than one that fails loudly and slower.
- **Cursor exclusion.** `FR-PERC-006`: the pointer in a captured frame is the agent's own, and perception that treats it as an element will click on itself.
- **`CON-002`.** Not a driver so much as a filter applied before the list.

## Considered options

1. **Windows Graphics Capture**, per window.
2. **`PrintWindow`** with full-content rendering.
3. **`BitBlt`** from the window's own device context.
4. **A screen crop** of the region the window occupies.
5. **Desktop Duplication**, cropped to the window.

## Decision

**Windows Graphics Capture, scoped to the target window.**

Measured on the reference hardware against a windowed OpenGL game at 1280x720, eight seconds per route at a requested 30 Hz:

| Route | Frames per second | p50 | Share of a 33 ms tick |
| --- | --- | --- | --- |
| **Windows Graphics Capture** | **55.1** | **2.27 ms** | **7%** |
| `BitBlt` | 29.8 | 3.56 ms | 11% |
| Screen crop | 29.7 | 10.33 ms | 31% |
| `PrintWindow` | 29.7 | 12.92 ms | 39% |

It is four to six times cheaper per frame than anything else and the only route that delivered **above** the requested rate — 55 frames a second, because it delivers on the game's own presentation rather than on a clock the caller sets.

Two properties beyond the frame rate settled it.

**Cursor suppression is accepted**, which `FR-PERC-006` needs and which no other route offers: a screen crop sees the pointer and has no way not to.

**The overlay is excluded by construction.** A per-window capture composites that window alone, so an overlay drawn as a separate top-level window never appears in a captured frame — measured across 2400 frames, zero pixels. `INV-GUI-001` makes that catastrophic to get wrong, and this route gives the property without depending on a flag anyone could remove. `WDA_EXCLUDEFROMCAPTURE` is set as well, and it is the belt rather than the braces.

### The failure signatures, which are the part that cannot be reasoned out

Against a compositor-drawn window, three of four routes failed **silently**. Every call succeeded. Nothing returned an error.

| Signature | What it means | What it is not |
| --- | --- | --- |
| 100% black frames | The route cannot see content drawn through a swap chain | A window that is not rendering |
| 100% repeated frames | A surface that succeeds for ever and never updates | A still scene |
| Under five frames in eight seconds | The compositor delivers on change and nothing changed | A broken route |

`FR-PERC-009` can be implemented against those. Without them it is a requirement with no observable to test.

The third row is why the probe's verdict vocabulary has five values rather than two: its first version read a single frame from a static window and called the route **usable**. One sample supports neither verdict, and the same mistake in the product would report a working capture on a paused game.

### Desktop Duplication is not implemented and not chosen

It is the one candidate that was not measured. Two reasons, and the second is the decisive one: the two routes that were measured already separate by a factor of five, and duplication captures the **whole display** — which sees other windows, sees the overlay, and would make the exclusion property above depend entirely on a flag.

It remains the fallback if window-scoped capture turns out not to support a window mode that matters, and that is the condition under which this record is reopened rather than a gap in it.

## Consequences

### Good

- 7% of a reflex tick, leaving the budget for the work that follows.
- The overlay cannot appear in a captured frame, by construction rather than by configuration, which removes the perception feedback loop `INV-GUI-001` is about.
- The cursor is excluded at the source rather than cropped out afterwards.
- Three named failure signatures, so `FR-PERC-009` has something to detect.
- Nothing is injected, hooked, or drawn into the game. `CON-002` is satisfied by the mechanism rather than by discipline.

### Bad

- It delivers on change, so a static scene yields almost nothing. That is correct behaviour and it means the perception layer cannot treat frame arrival as a clock — a paused game and a broken capture look identical from the frame rate alone. The classifier needs its own timeout.
- It is the newest of the five interfaces and the only one that needs a graphics device, a frame pool and interop between two object models. The other four are a dozen lines each.
- **Borderless and exclusive fullscreen are unmeasured**, and they are where capture routes historically differ most. Reaching them needed a person to change a game's video settings. This is the largest gap in the evidence and it is not small.
- Multiple displays and mixed scaling are unmeasured. `INV-GND-002` exists because a wrong transform lands clicks near the target, and "near" reads as a bad model rather than bad arithmetic.

### Neutral

- Graphics memory per route was not measured. The probe reads frames back into main memory, so its own footprint would have dominated the reading.

## Validation

This decision is wrong if window-scoped capture cannot see a window mode the roster needs.

Three conditions reopen it, each observable:

- **A roster title in borderless or exclusive fullscreen produces black frames or no frames.** This is the untested case and the most likely one.
- **The achieved rate falls below the reflex rate on a title that is actually rendering** — distinguished from a static scene by the classifier's own timeout rather than by frame arrival.
- **A window mode is found where the overlay appears in a captured frame.** That would mean the exclusion is not by construction after all, and the assertion in `experiments/07-capture/` has to be repeated per mode rather than once.

If any of these fires, Desktop Duplication cropped to the window is the named fallback, and the cost of taking it is that overlay exclusion stops being structural and starts depending on a flag.

## Pros and cons of the options

### Windows Graphics Capture

- Good, because it is four to six times cheaper per frame and the only route that keeps up with the game rather than with a clock.
- Good, because it sees swap-chain content, excludes the cursor, and excludes the overlay by construction.
- Bad, because it delivers on change, so absence of frames is ambiguous.
- Bad, because it is the most machinery.

### `PrintWindow` with full-content rendering

- Good, because it works on a compositor-drawn window where `BitBlt` does not, and it is a dozen lines.
- Good, because it sees a window that is covered by another.
- Bad, because it costs 39% of a reflex tick.
- Bad, because it returned **100% repeated frames** on one of the two windows tested: a stale surface that succeeds for ever, which is the hardest failure to notice.

### `BitBlt` from the window's device context

- Good, because it is the cheapest of the window-manager routes at 3.56 ms.
- Bad, because it returned **100% black frames** on a compositor-drawn window. It cannot see content the window did not draw through the window manager, which is all modern content.

### A screen crop of the window's region

- Good, because it reads exactly what a person sees, which makes it the right instrument for questions about what is *visible* — it is how the capture-border comparison was done.
- Bad, because it costs 31% of a tick, sees the cursor, sees any window in front, and would see the overlay.
- Retained as a diagnostic rather than as the capture path.

### Desktop Duplication cropped to the window

- Good, because it is the established high-performance route and sees everything.
- Bad, because seeing everything is the problem: other windows, the cursor, and the overlay.
- Unmeasured, and named as the fallback rather than dismissed.

## More information

- `experiments/07-capture/` holds the measurements, including two instruments that produced confident wrong answers before they had positive controls.
- `ADR-0025` excludes hooking and injection, which is why the field was five platform interfaces rather than the approaches most capture tools use.
- The overlay's own rendering approach is `ADR-0010`, which this record does not decide. What it establishes for that record is that the overlay cannot appear in a window-scoped capture, so the two questions are independent.
