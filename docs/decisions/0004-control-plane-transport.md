---
title: ADR-0004 — Use a named pipe for the control plane
description: "How commands and events travel between the core and the interface processes."
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
requirements: [NFR-GUI-001, FR-GUI-004]
decisions: [ADR-0003]
affects: [NFR-GUI-001, FR-GUI-004]
spec: [spec/10-ipc-protocol.md]
evidence: [docs/spec/10-ipc-protocol.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0004 — Use a named pipe for the control plane

## Context and problem statement

The core, the shell and the overlay exchange commands, events and queries: start a session, the plan changed, an action executed, a confirmation is required.

Low volume, latency not critical, and it must be access-controlled so that no other account can drive the agent.

## Decision drivers

- Single machine, few clients, no remote requirement today.
- A slow reader must not be able to stall the agent.
- Should not add a large dependency or a code-generation toolchain for its own sake.

## Considered options

1. A named pipe with length-prefixed binary messages
2. A remote-procedure-call framework over a local socket
3. A local HTTP interface
4. Shared memory for everything

## Decision

**A per-user named pipe in message mode**, carrying length-prefixed messages in a compact binary encoding, access-controlled to the current user.

A remote-procedure-call framework would work, and brings framing, code generation and a dependency for a single-machine, few-client link whose latency is already adequate.
Revisit if a remote control plane is ever wanted.

A local HTTP interface adds a listening socket for no benefit.

Shared memory alone cannot express request and response cleanly, and it is already used for the two channels where it is the right answer — see [ADR-0005](0005-data-plane-transport.md).

**The core never blocks on a slow reader.**
Writes to a full pipe are dropped after a bounded buffer, with a counter, so the gap is visible rather than silent.

## Consequences

### Good

- Platform-native, no dependency, straightforward on both sides.
- Access control is a property of the pipe rather than something to implement.
- A disconnected shell does not stop a session.

### Bad

- Message definitions must be generated for two languages from one schema.
- Dropping events under backpressure means the shell can miss things. The counter makes that visible; it is still a loss.
- No remote capability, by design.

### Neutral

- Binary rather than readable on the wire. The logs are the readable artefact.

## Validation

Revisit if a remote control plane becomes a requirement, or if the message catalogue grows large enough that hand-rolled framing becomes a maintenance cost rather than a simplification.
