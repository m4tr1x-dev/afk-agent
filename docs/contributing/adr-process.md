---
title: Decision process
description: "When a decision record is required, how it moves through its lifecycle, and who accepts it."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing,process]
requirements: []
decisions: []
generated: false
---

# Decision process

Architecture is decided in issues and recorded in decision records.
A pull request that makes an architectural choice in passing will be asked to come back and do this first.

## When a record is required

If a choice satisfies any of these, it needs one:

- It would be expensive to reverse once code depends on it.
- It closes off an option somebody would reasonably expect to be open.
- It contradicts something already written in the specification.
- Two competent people could disagree and both be defensible.
- It concerns a constraint, a non-goal, or a safety property.

If none apply, it is an implementation choice. Make it, and explain it in the code.

## The lifecycle

```mermaid
stateDiagram-v2
    [*] --> proposed: issue opened, record drafted
    proposed --> accepted: agreed
    proposed --> rejected: not doing this
    accepted --> deprecated: no longer relevant
    accepted --> superseded: replaced by a later record
    rejected --> [*]
    deprecated --> [*]
    superseded --> [*]
```

A rejected record stays in the repository.
The reasoning for not doing something is worth as much as the reasoning for doing it, and without the record the same proposal returns every six months.

## The rule that matters most

**A record is not accepted without evidence behind it.**

For most of this project's blocking decisions that means a timeboxed experiment: capture a thousand frames from a real game, inject input into a real game, measure inference on the target hardware.

A record written without one is a guess in a smart format, and it is sent back.
The [known-good matrix](../known-good-matrix.md) lists the experiments that are outstanding.

### The rule is enforced, not merely stated

Every record carries an `evidence` field listing the artefacts behind it, as
repository-relative paths. `tools/lint_frontmatter.py` rejects an accepted or
proposed record whose list is empty, and rejects any path that does not exist.

Until that check existed the rule lived only in prose, which for a project
partly built by an unattended agent is the same as not existing: the party
writing the record is the party who would have to hold themselves to it.

### Two kinds of evidence, and they are not interchangeable

An **experiment** is a measurement: a findings file under `experiments/`, a
benchmark result, a recorded corpus. It is the only admissible evidence for a
decision that turns on a number — which inference backend, which model
variant, which capture interface, what the panic latency is.

An **argument** is an explanation page that works the question through. It is
admissible for a decision that turns on reasoning rather than measurement:
whether the core is a separate process from the shell, whether the project ever
attempts to evade detection, where the message catalogue lives.

Anyone can tell which a record has by looking at the paths. A record whose
decision needs a number and whose evidence is an essay is the failure this
field exists to make visible, and it is visible at a glance rather than after
reading the record.

## Numbering

Four digits, assigned once, never reused, never renumbered.

Gaps are normal and carry information — `0037` and `0038` are unused because a direction was considered and dropped, and renumbering would break every reference that already points at a record.

## Superseding

A record is never edited to change its decision.
It gains a `superseded_by` field, and the new record explains what changed and why the original reasoning no longer holds.

Editing a record to say something different destroys the only thing it was for.

## Who accepts

The architecture owner, listed in the code owners file.
The specification and the decision records require that review; everything else is open.
