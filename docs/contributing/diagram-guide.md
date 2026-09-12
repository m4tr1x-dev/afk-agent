---
title: Diagram conventions
description: "Mermaid conventions, the C4 shapes, and when to reach for something else."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing,documentation]
requirements: []
decisions: []
generated: false
---

# Diagram conventions

Mermaid, inline, for almost everything.

It is the only option that renders natively both on GitHub — so a reviewer sees the diagram in a pull request diff — and in the built site, with no build step.

## Do not use the C4 diagram types

Mermaid has dedicated C4 diagram types.
They have been experimental with unstable syntax for a long time, and the version GitHub bundles may lag the one the site ships.

Express C4 with a plain flowchart and the conventions below.
Less pretty, considerably more stable.

## Shapes

| Shape | Means |
| --- | --- |
| Rectangle | A container: a process, a service, a store |
| Rounded | An external system we do not control |
| Stadium | A person |
| Cylinder | A data store |
| Diamond | A decision point, in flowcharts only |

Labels carry the technology in a second line where it is relevant.

## Colours

Defined per diagram with class definitions, from a small palette:

| Class | Use | Fill |
| --- | --- | --- |
| Trusted | Our code | Blue-grey |
| Untrusted | Anything whose content we do not control | Red-brown |
| External | The platform, the game, the network | Neutral grey |

The trusted and untrusted distinction is used consistently in the architecture and security pages, and it carries meaning — do not use those colours decoratively elsewhere.

## Direction

Left to right for pipelines and dataflow.
Top to bottom for hierarchy and layering.

## Every diagram needs prose

A paragraph after each one, describing what it shows.

Search indexes prose and not SVG, and a reader using assistive technology gets nothing from the diagram alone.
If the paragraph is hard to write, the diagram is probably unclear.

## The escape hatch

If a diagram genuinely cannot be expressed in Mermaid, commit the source under `docs/diagrams-src/` and the rendered image under `docs/assets/diagrams/`, and render it in continuous integration so a stale image fails the build.

Use this rarely. A diagram that needs a build step is a diagram that stops being updated.

## When to revisit

If the container diagram has been edited more than about ten times and the levels have drifted out of agreement, a modelling tool that derives views from one description starts to earn its cost.

That would be a decision record, not a preference.
