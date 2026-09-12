---
title: Reference
description: Factual descriptions of what exists — schemas, codes, defaults and generated API documentation.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [reference]
requirements: []
decisions: [ADR-0001]
generated: false
---

# Reference

Reference pages describe what exists right now.
They are short, factual, and written in the third person.

Most of them are **generated from source** and carry a `generated: true` field in their front matter.
CI regenerates them on every pull request and fails if the committed copy differs, which makes structural drift between code and documentation impossible rather than merely discouraged.
Never edit a generated page by hand.

## Generated

None of these exist yet; each appears as soon as the code it derives from does.

| Page | Generated from |
| --- | --- |
| Command-line interface | The argument parser |
| Configuration schema | The configuration types |
| Skill manifest schema | The manifest schema |
| Action schema | The action vocabulary |
| Inter-process messages | The protocol definition |
| Error codes | The error enumerations |
| Telemetry events | The metrics registry |
| Requirements traceability | The specification, decision records, source and tests |

The traceability table is the payoff for the requirement identifier scheme.
It joins every identifier to the page that defines it, the decisions that govern it, the code that implements it and the test that covers it, and CI fails when an identifier is referenced without a definition, defined twice, or marked implemented without a covering test.

## Hand-written

| Page | Contents |
| --- | --- |
| Sensor expressions | The grammar for sensor predicates |
| Session recording format | What a recording contains and how to read it |
| Keybindings | Default hotkeys and conflict handling |
| File and data layout | Every path the application touches |
| Supported models | Tested models, quantisations and measured latency |
| Prompt templates | The shipped system prompts |

## Generated API documentation

See the [API reference](api/index.md).
