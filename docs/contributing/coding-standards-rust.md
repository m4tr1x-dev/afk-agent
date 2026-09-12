---
title: Rust coding standards
description: "Conventions for the core: lints, error handling, the unsafe policy, and documentation requirements."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing, rust]
requirements: []
decisions: []
generated: false
---

# Rust coding standards

For the core: capture, perception, reasoning, the executor.

Nothing here is unusual for a Rust project.
The three rules specific to this one are about `unsafe`, about the reflex loop, and about who may synthesise input.

## Formatting and lints

Standard formatting, no configuration, enforced in continuous integration.

Clippy runs with warnings denied.
A lint that is genuinely wrong for a case is allowed at the narrowest possible scope with a comment saying why — never crate-wide, because a crate-wide allow silences the next hundred instances too.

## Documentation

Public items in every crate carry documentation, enforced rather than encouraged.
Broken intra-documentation links fail the build.

Documentation examples are compiled and run in continuous integration, which makes them the one kind of example that cannot rot.

## Errors

Two crates, two roles, and the distinction matters.

**Library crates define their own error types**, with variants that a caller can match on and act differently for.
A library that returns an opaque error forces every caller to treat every failure identically.

**The binary uses a context-carrying error type.**
At the top level nobody matches on the variant; they want the chain of what was being attempted when it failed.

Errors carry a code from [the taxonomy](../spec/16-error-taxonomy.md) and a category.
The category is what decides whether the agent retries, pauses, or fails loudly.

Panics are for defects.
An assumption the code relies on being false is not a condition to recover from — see the error taxonomy's reasoning about continuing in software that synthesises input.

## Unsafe

Unavoidable: this code calls platform APIs, maps graphics resources, and links a C library.

Every `unsafe` block carries a `SAFETY:` comment stating the invariant that makes it sound.
Not what the code does — what must be true for it to be correct.

An unsafe block without one does not pass review.
This is the single convention most worth enforcing here, because the reason the core is written in Rust is memory safety under hours of unattended foreign-function work, and unexamined unsafe blocks give that away for nothing.

Prefer to wrap unsafe in a safe abstraction at the narrowest scope, so the number of places a reviewer has to think hard about stays small.

## The reflex loop

The one place with a hard deadline.

Inside it:

- **No allocation** on the steady path.
- **No blocking** — no locks a worker might hold, no network, no inference, no accessibility queries.
- **No unbounded work.** Every loop over a collection has a known bound.

The hazards are the ones that look harmless.
A lock that is almost never contended, a log line that formats a large structure, a collection that is usually short.
Each is fine until the day it is not, and the symptom is stutter in someone's game.

## Input synthesis

**One call site in the entire workspace.**

If a change adds a second, it is wrong, and review should say so without further discussion.
Every safety guarantee in the project terminates at that one place, and a second one is a second place they can be violated.

## Concurrency

Asynchronous execution for input and output, model calls, skills and the protocol.

The reflex loop is a thread, not a task.
It has a deadline, and a runtime scheduling it alongside other work will eventually miss it.

Long computations — text recognition, element detection, image matching — go to a worker pool and land asynchronously.
They are never on the reflex path.

## Naming

Types and traits in upper camel case, everything else in snake case.

Use the vocabulary from the [glossary](../glossary.md).
A type called `Task` in code and a "subgoal" in the specification is a small thing that makes every conversation slower.

## Requirement identifiers

Code implementing a requirement carries its identifier in a comment at the implementation site.

The traceability report is generated from those markers, and it is what lets anyone answer whether the thing specified was actually built.
