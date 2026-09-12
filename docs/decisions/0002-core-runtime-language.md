---
title: ADR-0002 — Write the core in Rust
description: "Which language the perception, reasoning and execution core is written in."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, architecture, rust]
requirements: [NFR-ACT-001, NFR-PERC-001, NFR-PERC-002, FR-ACT-001, FR-ACT-008]
decisions: [ADR-0003]
affects: [NFR-PERC-002, NFR-ACT-001, FR-ACT-008]
spec: [spec/03-container-architecture.md, spec/04-perception.md, spec/06-action-and-input.md]
evidence: [docs/explanation/rust-and-csharp-split.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0002 — Write the core in Rust

## Context and problem statement

The core owns frame capture, perception, the reasoning cadences and the executor.
It runs unattended for hours, holds graphics resources, calls platform APIs continuously, links at least one C library, and has one loop with a hard deadline.

The interface is settled: the shell and overlay are C# with the platform's own framework, which is a project requirement.
The question is whether the core should be the same language, or a different one.

## Decision drivers

- The reflex loop has a 33 ms deadline and holds keys down. Missing it produces stutter in the user's game.
- The frame pipeline is the allocation-heaviest part of the system.
- The core runs for hours with nobody watching, doing continuous foreign-function work.
- A second language costs a toolchain, a schema in the middle, and cross-boundary debugging.
- Developer velocity matters for a small team.

## Considered options

1. Rust
2. C# with ahead-of-time compilation
3. C++

## Decision

**Rust**, with C# for the shell and overlay.

Three arguments, in increasing order of weight.

**The frame pipeline is where a managed runtime does worst.**
The parts of a frame that reach main memory are large, short-lived and continuous.
That can be handled in C# — native allocations, pooled buffers, pinned spans — but doing so means writing unsafe managed code with manual lifetimes, which surrenders the safety that was the reason for the managed runtime while keeping its overhead.
In Rust the same code is borrowed slices over a mapped resource, checked at compile time.

**Unattended operation is a memory-safety requirement.**
Eight hours of continuous interop, graphics resource handling and calls into C libraries, unsupervised.
The failure this prevents is a process that has been subtly wrong for two hours and is still synthesising input.

**The interop cost is zero either way, which neutralises the strongest argument for one language.**
A process boundary is required regardless — the core must outlive the shell, the overlay must outlive both, and a graphics fault must not take down the agent. Those requirements come from [safety](../spec/12-safety-and-limits.md), not from language choice.
Once there is a process boundary, a C# core and a Rust core pay identical communication costs.

C++ is dominated.
It offers nothing here that Rust does not, and costs memory safety in exactly the code that is most dangerous: raw handles, graphics resource lifetimes, mapped buffers, and foreign-function calls.

## Consequences

### Good

- The allocation-heavy path is safe at compile time with no runtime cost.
- The core runs headless, which the benchmark harness and replay both need, and that falls out of the split rather than being built for.
- Crash isolation is a safety property: a fault in the shell cannot take down something holding a key down in a game.

### Bad

- Two toolchains for contributors and for continuous integration.
- A schema in the middle, generating types for both sides. More work than a shared header, and it is also what stops the two sides drifting.
- Cross-boundary debugging needs two debuggers and correlated logs. Genuinely annoying.
- Slower development than a single-language version, for the orchestration-heavy parts.

### Neutral

- The shell stays C# regardless. Reimplementing the platform's interface framework elsewhere was never a serious option.

## Validation

Revisit if **both** of these become true:

1. The perception loop drops to event-driven keyframes at 5 Hz or below, with all downscaling on the graphics device, so that main memory never sees more than a small image and the per-tick jitter budget rises above 15 ms at the 95th percentile.
2. The inference runtime is reached through a first-party managed binding that does not lag upstream.

The first half is plausible — the design already gates the model to keyframes.
What keeps it from being true is the reflex loop: 20–30 Hz sensor evaluation, input reconciliation and stepping of held actions, with a hard deadline.

**If that loop turns out to be unnecessary, this decision should be revisited honestly rather than defended.**

## More information

The narrative version, including the case for a single language taken seriously, is in [the Rust and C# split](../explanation/rust-and-csharp-split.md).
