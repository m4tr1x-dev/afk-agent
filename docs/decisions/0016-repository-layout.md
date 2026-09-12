---
title: ADR-0016 — One repository, with the cross-language contract as its own directory
description: "How the repository is organised and where the shared schema lives."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, tooling]
requirements: [FR-ACT-008]
decisions: [ADR-0002, ADR-0004]
affects: [FR-ACT-008]
spec: [spec/10-ipc-protocol.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0016 — One repository, with the cross-language contract as its own directory

## Context and problem statement

The project has a Rust core, a C# shell and overlay, a documentation site, and a message catalogue that both languages need types for.

Those could live in one repository or several, and the message catalogue could live with either language or on its own.

## Decision drivers

- The core and the shell change together more often than separately.
- Both languages need types generated from one schema.
- A contributor should be able to build and run the whole thing from one clone.
- Documentation drift checks need to see the source and the documentation in the same change.

## Considered options

1. One repository, contract in its own directory
2. One repository, contract inside the Rust workspace
3. Separate repositories per language

## Decision

**One repository**, with this shape:

```text
contract/        message and tool schemas — the single source of truth
crates/          the Rust core, one crate per subsystem boundary
src/             the C# shell and overlay
tests/           corpus, benchmark harness, replay fixtures
docs/            the specification and all documentation
tools/           documentation and build scripts
```

Separate repositories are wrong here for a specific reason: a change to the message catalogue touches both languages and the reference documentation, and splitting it across repositories turns one reviewable change into three that must land in order.

`contract/` sits outside both language trees deliberately.
Putting it inside the Rust workspace would make one language the owner of a thing both depend on, and in practice that means the other language's needs get considered second.

The schema generates the Rust types, the C# types, and the generated reference page.
**Hand-written types on two sides of a boundary drift, and the drift is discovered at runtime.**

### Crate boundaries

One crate per subsystem boundary in the specification — capture, perception, model, action, safety, protocol, and so on — rather than one large crate.

The reason is `FR-ACT-008`: input synthesis lives in exactly one crate, and a crate boundary is something a reviewer can see. A module boundary inside one crate is a convention.

## Consequences

### Good

- One clone, one change, one review for anything that crosses the boundary.
- The documentation drift check can compare source and documentation in the same change.
- Generated types cannot disagree with the schema.

### Bad

- The repository needs both toolchains to build fully, even for a contributor touching one side.
- Two build systems orchestrated from one task runner, which is a layer that exists only to hide the split.
- Continuous integration is slower than either half alone.

### Neutral

- The documentation builds with a third toolchain, which is Python, and is independent of both.

## Validation

Revisit if the C# side grows to the point where its build dominates continuous integration time for changes that do not touch it — the fix then is conditional jobs rather than separate repositories.

Revisit the contract's location if a third consumer appears that is neither language.
