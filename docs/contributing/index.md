---
title: Contributing
description: How to work on this project, and what is most useful right now.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing]
requirements: []
decisions: []
generated: false
---

# Contributing

The most valuable contribution to this project today is **reviewing the specification**.

There is no code.
Everything is still cheap to change, and a wrong assumption caught on a page costs an afternoon, while the same assumption caught in a subsystem costs a fortnight.
If you read a specification page and something does not add up, that is the contribution — open an issue.

## Ways in

- **Review a specification page.** Especially [action and input](../spec/06-action-and-input.md), [grounding and verification](../spec/18-grounding-and-verification.md) and [safety and limits](../spec/12-safety-and-limits.md), which carry the most risk.
- **Challenge a decision.** Every record has a Validation section naming what would make us revisit it. If you think one of those conditions already holds, say so.
- **Answer an open question.** The [known-good matrix](../known-good-matrix.md) lists what is unresolved. Several can be settled by anyone with the hardware and an afternoon.
- **Write documentation.** Read the [documentation guide](documentation-guide.md) first; most of its rules are enforced by CI.

## Before you write code

Nothing is ready to implement until its governing decisions are accepted.
The blocking set is listed in the [decision index](../decisions/index.md), and each one is preceded by a timeboxed experiment.
A decision record written without one is a guess in a smart format.

## Planned contributor documentation

- Development environment — exact toolchain versions and how to verify them
- Repository layout
- Build and run
- Coding standards for C# and for Rust
- Testing strategy — what you assert when the model is stochastic
- Decision process
- Diagram conventions
- Release process

## House rules

English for everything in the repository: code, comments, commit messages, issues and documentation.
The reasoning is in the [documentation guide](documentation-guide.md).

One logical change per pull request, with a commit message in Conventional Commits form.

Any change to behaviour updates the matching specification or reference page in the same pull request.
CI will tell you when a generated reference page is stale; it will not tell you when a hand-written one is, which is what review is for.
