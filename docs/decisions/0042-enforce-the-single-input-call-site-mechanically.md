---
title: ADR-0042 — Enforce the single input call site in four independent layers
description: Make a second input call site fail to compile or fail the build, rather than fail review, and make the unsafe surface a six-item list rather than a search.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, safety, build]
requirements: [FR-ACT-008, INV-GND-002]
decisions: [ADR-0016, ADR-0025]
affects: [crates, src, tools]
spec: [docs/spec/06-action-and-input.md]
evidence:
  - clippy.toml
  - deny.toml
  - tools/check_call_sites.py
  - crates/afk-input/src/sys.rs
  - crates/afk-input/src/lib.rs
supersedes: null
superseded_by: null
generated: false
---

# ADR-0042 — Enforce the single input call site in four independent layers

## Context and problem statement

`FR-ACT-008` says input synthesis occurs in exactly one component, with no other call site anywhere in the system.
`CLAUDE.md` repeats it as a hard constraint and adds the instruction a reader is meant to act on: *"If you are adding a second call site, stop."*

Until now that rule was carried entirely by prose and by review.
Prose does not survive forty context compactions of an unattended build, and review does not survive a tired afternoon.
More to the point, a second call site is not a defect anybody would introduce deliberately; it is what happens when a crate adds a dependency that happens to expose the platform's input functions and somebody reaches for the obvious one.

This is worth taking seriously because the requirement's rationale is not stylistic.
Every release guarantee the project makes — release every held key on shutdown, never deliver input while the target is not foreground, stop within the panic budget — is implemented once, in one place. A second call site is a second place all of them can be violated, and it is found by a user rather than by a test.

`INV-GND-002` has the same shape: every coordinate reaching input synthesis has passed through the single transform chain, and coordinate arithmetic performed elsewhere is invisible until somebody has two monitors.

How is a rule like this made true rather than merely stated?

## Decision drivers

- **Cargo unifies features across the whole dependency graph.** This is the trap, and it is not obvious: if any crate in the workspace enables a Windows bindings crate's keyboard-and-mouse feature, *every* crate that depends on that bindings crate can call the platform's input functions, and a dependency audit sees nothing wrong.
- **A rule with an exception in the repository is a rule with a template for violating it.** If `afk-input` needs an `#[allow]`, that `#[allow]` is what somebody copies.
- **The layers have to be independent.** A lint knows only today's interfaces; a dependency policy knows only declared dependencies; a grep knows only spellings.
- **The `unsafe` surface should fit in a reviewer's head**, as a list rather than as the result of a search.

## Considered options

1. Prose and review, as today.
2. A single grep in continuous integration.
3. A Clippy `disallowed-methods` list.
4. Four independent layers, each catching what the others cannot.

## Decision

**Option 4.** Four layers, deliberately overlapping.

### Layer 1 — `afk-input` declares its own foreign interface

```rust
// REQ: FR-ACT-008 - the entire operating-system input surface of this project.
#[link(name = "user32")]
unsafe extern "system" { fn SendInput(/* ... */) -> u32; }
```

No bindings crate. This is the layer that defeats feature unification: the rest of the workspace cannot inherit an input interface from a dependency `afk-input` does not have.

It also means **`afk-input` needs no `#[allow]` of its own**, because it does not use the paths the next layer forbids — it uses its own.
There is therefore no exception anywhere in the workspace for anybody to copy. That property is what makes this survive two years rather than two months.

### Layer 2 — `clippy.toml`

`disallowed-methods` and `disallowed-types` on the platform's input entry points and the structure they take.
Catches a crate that adds a bindings dependency and reaches for the obvious call.

### Layer 3 — `deny.toml`

The other direction: the Windows bindings crate is permitted only to a named list of crates, and `afk-input` is deliberately **not** on it.
Catches the dependency before the call exists.

### Layer 4 — `tools/check_call_sites.py`

Three rules, because layers 2 and 3 know only today's interfaces and today's crate names.

- **Rule A.** The input-synthesis spellings appear only under `crates/afk-input/` and `crates/afk-guardian/`.
- **Rule B.** No platform import of `user32` under `src/`. The C# side has nothing to do with input synthesis, and the shortest way to say so is that it may not name the library.
- **Rule C.** `#![forbid(unsafe_code)]` in every crate outside a six-name allowlist, **failing in both directions**: a crate outside the list without the attribute is an error, and a crate on the list that no longer needs it is also an error.

**Rule C is the quiet one and the most valuable.** It converts the `unsafe` surface of the project from something you find by searching into a six-item list: `afk-input`, `afk-capture`, `afk-shm`, `afk-uia`, `afk-ipc`, `afk-guardian`. A reviewer can hold that in their head. A search result changes every week.

Failing in both directions matters because a list that only grows becomes a list nobody trusts.

### For `INV-GND-002`, structure rather than a lint

`afk-geometry` exports `CapturePoint`, `ClientPoint`, `ScreenPoint` and `InjectionPoint` as newtypes **with private fields, no public constructors, and no `Add`, `Sub` or `Mul` implementations**.
The only route to an `InjectionPoint` is `Transform::to_injection(ScreenPoint)`.

Coordinate arithmetic elsewhere does not fail a lint. **It does not compile.**

### Consequence for the specification

`FR-ACT-008` and `INV-GND-002` were verified by `Manual, review` and `Unit, review`.
Both are now checkable by machine, and `docs/spec/01-requirements.md` carries a `Lint` method for both.
Without that, the traceability rule that every implemented requirement has a covering test had no satisfiable path for either, and an unsatisfiable rule is one somebody eventually exempts.

## Consequences

### Good

- A second call site fails the build rather than review, and it fails on the pull request that introduces it.
- The feature-unification trap is closed structurally. It is the failure most likely to slip past a careful reviewer, because nothing in the diff looks wrong.
- The `unsafe` surface is a list of six names, checked in both directions.
- Coordinate arithmetic outside the transform chain is a compile error, not a convention.
- `tools/check_call_sites.py` runs unfiltered on every pull request and ran before `afk-input` existed, so the rule was never retrofitted onto code that had already worked around it.

### Bad

- Four layers is four things to maintain, and three of them are spelling-based. A platform interface renamed upstream defeats layers 2 and 4 until somebody updates them.
- Hand-declaring the foreign interface means hand-maintaining structure layouts. `afk-input` carries compile-time size and alignment assertions for exactly this reason, and those assertions are the only thing between a layout change and silent memory corruption.
- The newtype design makes legitimate geometry code more verbose. Every transform is a named function call, including the ones that are obviously right.
- Rule C's allowlist is a judgement call. Six crates is defensible; nobody can prove it is minimal.

### Neutral

- `afk-guardian` appears in the allowlist for rule A because it must be able to release held input when the core is wedged. Its dependency set is separately constrained to at most four nodes, which is a requirement in its own right.

## Validation

This is wrong if the layers start producing false positives that people route around.

Observable conditions, any of which reopens it:

- **A legitimate change needs an `#[allow]` for a disallowed method.** The moment one exists, it is a template, and the design's central property is gone.
- **Rule C's allowlist reaches nine crates.** That is the point at which "a list a reviewer can hold in their head" stops being true, and the answer is a smaller `unsafe` surface, not a longer list.
- **A platform interface rename defeats rules 2 and 4 without anybody noticing.** Detectable: `afk-input` is the only crate that should ever need updating for such a rename, so a rename that touches a second crate is the signal.

## Pros and cons of the options

### Prose and review

- Good, because it costs nothing and reads well.
- Bad, because it is exactly what already existed, and the requirement's verification column said `Manual, review` — which is honest about how much it was worth.

### A single grep

- Good, because it is ten lines and catches the obvious case.
- Bad, because it knows only spellings. A crate that acquires an input interface through feature unification never spells anything the grep looks for.

### Clippy alone

- Good, because it understands paths rather than text and produces a good message.
- Bad, because `afk-input` would then need an `#[allow]`, and that `#[allow]` is the thing somebody copies.
- Bad, because it says nothing about the C# side.

### Four layers

- Good, because each layer fails differently, and the one that catches feature unification is the one nothing else would.
- Bad, because it is four files instead of one.

## More information

- `docs/spec/06-action-and-input.md` is the page these rules serve.
- `ADR-0025` excludes hooking, injection and driver-level input independently. This record is about the code we do write; that one is about the code we never will.
- `afk-probe` is required to use the real `afk-input` rather than its own synthesis. A probe with its own call site validates something the product does not send, which would make every reachability result inapplicable.
