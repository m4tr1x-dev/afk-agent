---
title: Decision records
description: "The architecture decisions behind this project, the reasoning at the time, and what would make us revisit them."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [adr]
requirements: []
decisions: [ADR-0000]
generated: false
---

# Decision records

Most of this project's architecture is being decided before there is running software to validate it against.
That makes the conclusions less valuable than the reasoning: in a year the conclusion is visible in the code, and the alternatives that were weighed are visible nowhere unless they were written down.

Each record carries the context, the drivers, every option seriously considered, the decision, its costs, and — the section that matters most here — what would make us change our mind.

Format and process are set by [ADR-0000](0000-use-madr-for-architecture-decisions.md).
Copy [the template](adr-template.md) and take the next free number.

## Accepted

| # | Title | Governs |
| --- | --- | --- |
| [0000](0000-use-madr-for-architecture-decisions.md) | Use MADR for architecture decision records | Process |
| [0001](0001-documentation-toolchain-and-docs-as-spec.md) | Documentation toolchain and the docs-as-spec workflow | Documentation |
| [0002](0002-core-runtime-language.md) | Write the core in Rust | Architecture |
| [0003](0003-process-topology.md) | Split into five processes | Architecture |
| [0004](0004-control-plane-transport.md) | Use a named pipe for the control plane | Protocol |
| [0005](0005-data-plane-transport.md) | Shared memory for telemetry, a shared texture for frames, a separate flag for panic | Protocol |
| [0009](0009-input-injection-method.md) | Synthesise input through the public API, with no kernel component | Action, policy |
| [0011](0011-application-framework-and-packaging.md) | Build the shell and overlay with the platform's native framework | Interface |
| [0012](0012-agent-loop-architecture.md) | Run three concurrent cadences | Reasoning |
| [0013](0013-constrained-decoding.md) | Constrain model output with a grammar regenerated every tick | Model |
| [0016](0016-repository-layout.md) | One repository, with the cross-language contract as its own directory | Tooling |
| [0018](0018-context-as-the-only-memory.md) | Context is the only memory | Architecture |
| [0025](0025-terms-of-service-and-anti-cheat-posture.md) | Stay in user space, and never attempt to evade detection | Policy |
| [0035](0035-sustained-action-execution.md) | Model issues intentions with durations; an executor steps them | Action |
| [0036](0036-three-valued-verification.md) | Verification is three-valued, and failures are attributed | Grounding |

## Proposed

| # | Title | Blocked on |
| --- | --- | --- |
| [0014](0014-grounding-strategy.md) | Build both grounding paths and let measurement choose | The grounding benchmark |

## Not yet written, and why

The remaining blocking decisions are **blocked on experiments**, not on writing time.

A record written without evidence is a guess in a smart format, and each of these turns on a number nobody has measured.
The experiments are listed in the [known-good matrix](../known-good-matrix.md) with what each one settles.

| # | Title | Blocked on |
| --- | --- | --- |
| 0006 | Inference host and compute backend | Whether the vision encoder loads on the required backend |
| 0007 | Model selection and quantisation | Latency and memory, measured with a game running |
| 0008 | Screen capture API | A prototype against several window modes |
| 0010 | Overlay rendering approach | A prototype, including capture-border suppression |
| 0017 | Interop binding technology | A prototype across the process boundary |

Per-subsystem records, written when the subsystem is designed rather than in advance:

0015 perception preprocessing · 0019 context compaction · 0020 skill format and execution · 0021 web research and fetch policy · 0022 configuration format · 0023 logging and recording format · 0024 safety architecture · 0026 testing a non-deterministic agent · 0027 error handling and recovery.

Before a public release:

0028 versioning · 0029 update mechanism · 0030 licensing · 0031 crash reporting and privacy · 0032 localisation · 0033 multi-monitor and fullscreen policy · 0034 telemetry policy.

## A note on the gaps

Numbers 0037 and 0038 were reserved during planning for a continual fine-tuning pipeline and an adapter lifecycle.

That direction was dropped: the agent does not learn, and the model's weights never change.
The numbers stay unused rather than being reassigned, because renumbering breaks every reference that already points at a record.

Gaps carry information.
This one records that an option was considered and rejected, which is worth more than a tidy sequence.
