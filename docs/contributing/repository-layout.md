---
title: Repository layout
description: "One line per directory, and what belongs where."
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

# Repository layout

## Today

```text
docs/            the specification and all other documentation
  spec/          normative pages
  decisions/     architecture decision records
  explanation/   the reasoning behind the hard parts
  reference/     factual pages, mostly generated once code exists
  how-to/        task-oriented guides
  tutorials/     guided lessons
  contributing/  how to work here
  assets/        images and rendered diagrams
  snippets/      shared documentation fragments
styles/          Vale prose rules and vocabulary
tools/           documentation lint scripts
.github/         workflows, templates, code owners
```

Everything else at the root is configuration: the site, the linters, the editor, and the repository metadata.

## When the code exists

Set by `ADR-0016`, which is not accepted.
The shape the specification implies:

```text
crates/          the Rust core, one crate per subsystem boundary
src/             the C# shell and overlay
contract/        the message and tool schemas, the single source of truth
tests/           corpus, benchmark harness, replay fixtures
tools/           documentation and build scripts
```

`contract/` is the one worth noting.
The inter-process message catalogue and the tool schemas are defined once and generate types for both languages and the reference documentation.
Hand-written types on two sides of a boundary drift, and the drift is found at runtime.

## Keeping this honest

A continuous integration check verifies that every top-level directory appears on this page.
A directory that exists and is not listed fails the build, which is the cheapest way to stop this page becoming fiction.
