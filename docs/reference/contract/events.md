---
title: Inter-process events
description: Generated from the contract schema - every message, its wire tag and its fields.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [reference, contract]
requirements: []
decisions: [ADR-0004]
generated: true
generated_from: contract/schema/events.toml
---

# Inter-process events

Generated from `contract/schema/events.toml`.
Editing it by hand does not change the protocol; it changes a file the next generator run overwrites, and continuous integration fails before that happens.

Wire tags are assigned once and never reused.
A decoder that meets a tag it does not know skips the frame, which is what lets an older process ignore a message it was never taught.
That holds only while a tag keeps meaning what it meant.

## Event

Core to interface. A new variant is a minor bump; older clients skip it.

### SessionState (tag 1)

FR-LOOP-008. The state machine's current state, and why it is there.

| Field | Type | Since | Meaning |
| --- | --- | --- | --- |
| `state` | `u8` | 1.0 | Idle, Briefing, Running, Stopping, Stopped. |
| `reason` | `string` | 1.0 | One sentence, for the overlay. |

### VerificationOutcome (tag 7)

FR-GND-006. Exactly one of three values; INV-GND-001 forbids collapsing the third.

| Field | Type | Since | Meaning |
| --- | --- | --- | --- |
| `action_id` | `u64` | 1.0 | The action this outcome belongs to. |
| `outcome` | `u8` | 1.0 | 0 confirmed, 1 refuted, 2 inconclusive. |
| `evidence` | `string` | 1.0 | What was observed, in one sentence. |

## Tools

The grammar admits these and nothing else, and it is rebuilt every tactical tick.
A parameter of type `mark` is filled from the marks in the current observation, so naming a target that is not on screen is inexpressible rather than merely unlikely.

### `click`

Click a mark. FR-GND-001: choosing from a list is structurally safer than producing coordinates.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `mark` | `mark` | One of the marks in the current observation. The grammar enumerates them. |
| `button` | `left`, `right`, `middle` | Which button. Most interfaces want the left one. |

### `look`

Turn the camera by a relative delta over a duration. FR-ACT-004.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `dx` | `i32` | Horizontal delta in mouse units. Negative turns left. Calibrated per session. |
| `duration_ms` | `u32` | How long to spread the movement over, in milliseconds. |
