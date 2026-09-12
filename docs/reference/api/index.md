---
title: API reference
description: Generated API documentation for the Rust core and the C# application shell.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [reference, api]
requirements: []
decisions: [ADR-0001]
generated: false
---

# API reference

The project is written in two languages, and each has a native documentation generator that the other does not understand.
Rather than force them together, both are generated independently and mounted under this site.

| Component | Language | Location |
| --- | --- | --- |
| Core, capture, perception, action, model host | Rust | `/api/rust/` |
| Application shell, overlay | C# | `/api/dotnet/` |

Neither sub-site exists until the publish workflow has run against code that exists.

## Linking into these from the specification

Link at **module granularity, not item granularity**.
Item paths in young code move on every refactor, and a link checker that fails on every refactor gets switched off — which costs more than the precision was worth.

Once the public surface has settled, item-level links become reasonable, and the link checker then earns its keep by catching the ones that rot.

## Why they are not merged

Feeding one generator's output into the other is a maintenance sink with no benefit to any reader.
The seam is that this site's search covers the hand-written pages but not the generated API documentation; a reader looking for a type name uses the sub-site's own search.
That is an acceptable cost.
