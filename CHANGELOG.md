# Changelog

All notable changes to this project are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

There is no release. This section records the documentation set that the first
implementation will be built from.

### Added

**Specification** — 21 normative pages defining the system boundary, 101
identified requirements across eleven areas, the three-cadence reasoning loop,
the action vocabulary and executor, the grounding and verification design,
context as the only memory, the safety interlocks, the threat model, the
performance budgets, the error taxonomy, the evaluation harness and the
non-goals.

**Decision records** — 15 accepted and one proposed, covering the core
language, process topology, both transports, input injection, the interface
framework, the loop architecture, constrained decoding, repository layout,
memory, the terms-of-service posture, sustained-action execution and
three-valued verification. The grounding strategy is deliberately left
proposed, because it is blocked on a measurement.

**Explanation** — 13 essays on the product thesis, memory, grounding, sustained
actions, token economics, failure modes, the language split, the overlay,
research, and the project's position on game protection systems.

**Contributing** — the documentation guide, development environment, repository
layout, build and run, coding standards for both languages, testing strategy,
the decision process, diagram conventions and the release process.

**Reference** — the sensor expression grammar. The remaining reference pages are
generated from source and appear when the source does.

**Tooling** — a strict documentation build, front-matter and requirement
identifier checks, Markdown linting, prose linting with four project-specific
rules, spell checking, link checking, and two CI workflows.

### Changed

- `README.md` rewritten as an entry point rather than a description.
- `.gitignore` now covers Rust and .NET rather than Python and Node.

### Notes

The prose is British English, and the identifier scheme treats constraints as
global rather than belonging to a subsystem area.
