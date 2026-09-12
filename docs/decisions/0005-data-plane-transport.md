---
title: ADR-0005 — Shared memory for telemetry, a shared texture for frames, a separate flag for panic
description: "How high-rate data crosses the process boundary without copying or blocking."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, ipc]
requirements: [FR-SAFE-001, NFR-OBS-001, NFR-GUI-001]
decisions: [ADR-0004]
affects: [FR-SAFE-001, NFR-OBS-001]
spec: [spec/10-ipc-protocol.md]
evidence: [docs/spec/10-ipc-protocol.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0005 — Shared memory for telemetry, a shared texture for frames, a separate flag for panic

## Context and problem statement

Three kinds of traffic do not fit the control channel.

The overlay needs what to draw, every tick.
The shell needs the annotated frame when its live view is open.
And the panic flag needs to work when everything else has stopped working.

## Decision drivers

- Per-tick data must not be serialised through a pipe.
- Frames must not be copied through main memory.
- A slow reader must never block the core.
- The panic path must not depend on any other mechanism.

## Considered options

1. Everything over the control pipe
2. A ring for telemetry, a shared texture for frames, a separate flag for panic
3. A second pipe per channel

## Decision

**Three separate mechanisms**, each matched to its traffic.

**Telemetry: a shared memory ring** with a per-slot sequence counter.
The writer increments the counter before and after writing; a reader that observes an odd or changed value retries, and is free to drop a frame it could not read in time.

The overlay is a display, not a subscriber that must see everything.
Dropping is correct behaviour, and a protocol that could block the core because a reader was slow would be a defect.

**Frames: a shared texture.**
The core renders the annotated frame into a shareable texture, passes the handle over the control pipe, and the shell binds it directly.
No copy through main memory, no serialisation.

**Panic: one value in its own allocation**, polled by the reflex loop every tick, with no framing and no queue.

It is separate from everything else because it must work when the pipe is dead, the ring is full, and the core is servicing nothing.
A halt mechanism that depends on machinery that has stopped working is not a halt mechanism.

## Consequences

### Good

- Per-tick data costs no serialisation and no copy.
- The core cannot be stalled by a slow interface.
- The panic path has no dependencies on anything that can fail first.

### Bad

- Three mechanisms to implement and reason about rather than one.
- Shared memory and texture handles are platform-specific and need unsafe code on the Rust side.
- A dropped telemetry frame is invisible to the user. Intended, and still a loss of information.

## Validation

Revisit the frame channel if the live view proves rarely used — publishing unconditionally costs graphics time for nothing, and the protocol already has an attach-on-demand path.

The panic channel is not subject to revision on performance grounds.
