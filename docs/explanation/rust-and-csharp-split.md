---
title: The Rust and C# split
description: "What the process boundary between the core and the interface buys, and what it costs."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, architecture]
requirements: []
decisions: []
generated: false
---

# The Rust and C# split

The project is written in two languages.
The core — capture, perception, reasoning, the executor — is Rust.
The shell and the overlay are C# with the platform's own interface framework.

Two languages is a real cost, so it should have a real reason.

## The case for one language

Taking it seriously first, because it is stronger than people who like Rust tend to admit.

**A single language means a single build, a single test story, and a single set of types across the whole product.**
No schema file generating two sets of structures. No serialisation between components that could have been a function call. One toolchain to install.

**Garbage collection is not the problem it is usually claimed to be at this scale.**
Modern collectors produce sub-millisecond young-generation pauses. Against a 33 ms tick budget, that is survivable, and "the collector will ruin your loop" is folklore at these timings.

**The platform bindings are all there.**
Window capture, input synthesis, composition — all reachable from the managed side without difficulty.

A competent single-language version of this project is a real thing that would work.

## What actually decides it

Three arguments, in increasing order of weight.

### The frame pipeline is the allocation-heaviest part of the system

A 4K frame is about 33 MiB.
Even after the [economising](perception-token-economics.md) that keeps most of that on the graphics device, the parts that do reach main memory are large, short-lived, and arriving continuously.

In a managed runtime this is large-object-heap churn, which is the collector's least favourite kind of work.
It can be avoided — native allocations, pooled buffers, pinned spans over mapped resources — but doing so means writing unsafe managed code with manual lifetimes, which surrenders the safety that was the argument for the managed runtime while keeping its overhead.

In Rust the same code is borrowed slices over a mapped resource, checked at compile time, with no pooling ceremony and no copies.
**You get the performance profile of unsafe native code with compile-time safety, which is the precise shape of this workload.**

### It runs unattended for hours doing continuous foreign-function work

The product claim is "runs while I sleep".

That means a process doing continuous interop with platform APIs, holding graphics resources, and calling into at least one C library, for eight hours, with nobody watching.

Rust's guarantee that this cannot corrupt memory is not a nicety in that setting.
The failure mode it prevents is a process that has been subtly wrong for two hours and is still synthesising input.

### The interop cost is zero either way

The argument that usually decides against a second language does not apply here, and this is the part worth understanding.

**A process boundary is required regardless of language.**
The core must outlive the shell, the overlay must outlive both, and a graphics-device fault must not take down the agent. Those requirements come from [safety](../spec/12-safety-and-limits.md), not from language choice.

Once there is a process boundary, a C# core and a Rust core pay **identical** communication costs.
The "one language" benefit shrinks from "no serialisation" to "shared types and one mental model" — real, but no longer architectural.

## Why the interface is not also Rust

Because the platform's interface framework is a managed framework, and using it from anywhere else is an exercise in friction.

The shell wants data binding, styling, the platform's own material effects, and an accessibility story that works without being built by hand.
All of that exists, is maintained, and is written for C#.

There is no serious argument for reimplementing it.

## What the split costs

**Two toolchains.** A contributor installs both, and continuous integration builds both.

**A schema in the middle.** The message catalogue is defined once and generates types for both sides. This is more work than a shared header, and it is also the thing that stops the two sides drifting.

**Debugging across the boundary.** A failure that starts in the core and manifests in the shell needs two debuggers and correlated logs. This is genuinely annoying.

**A contributor needs both.** Someone who wants to improve the overlay has to build the core to run it.

## What the split buys, beyond the memory argument

**Crash isolation, which is a safety property.**
A fault in the shell cannot take down something that is holding a key down in a game. The guardian exists for the case where the core does die, and the split is why that is a recoverable situation rather than a stuck keyboard.

**The core can run without an interface.**
Headless operation, the benchmark harness, and replay all use the core with no window in sight. That falls out of the split rather than being built for.

**The overlay stays responsive.**
It does no inference, no capture, no allocation of consequence. When the core is fully loaded, the window carrying the panic key is not — which is exactly when it matters.

## What would flip this

The decision record states the condition, and it is not hypothetical.

If the perception loop dropped to event-driven keyframes at a low rate, with all downscaling on the graphics device so that main memory never sees more than a small image, *and* the inference runtime were accessed through a first-party managed binding with no lag behind upstream, then the native advantages would largely evaporate and single-language velocity would win.

The first half of that is plausible — the design already gates the model to keyframes.
What keeps it from being true is the reflex loop: 20–30 Hz sensor evaluation, input reconciliation, and stepping held actions, every tick, with a hard deadline.

If that loop turned out to be unnecessary, this decision should be revisited honestly rather than defended.
