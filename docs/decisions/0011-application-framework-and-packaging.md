---
title: ADR-0011 — Build the shell and overlay with the platform's native framework
description: "Which framework the interface uses, and how the product is packaged."
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, gui]
requirements: [FR-GUI-001, FR-GUI-002, NFR-GUI-001, CON-001, CON-007]
decisions: [ADR-0002, ADR-0003]
affects: [FR-GUI-001, FR-GUI-002, CON-007]
spec: [spec/09-gui-and-overlay.md]
supersedes: null
superseded_by: null
generated: false
---

# ADR-0011 — Build the shell and overlay with the platform's native framework

## Context and problem statement

The product has two interface surfaces: a shell for setting up and reviewing sessions, and an overlay that sits over a game.

They have very different requirements.
The shell is an ordinary desktop application. The overlay is a transparent, click-through, non-activating window that must be excluded from screen capture and must update annotations at the reflex rate.

## Decision drivers

- The overlay needs specific window behaviour that only the platform exposes.
- The overlay must be possible to exclude from capture, which is a platform capability.
- The shell should look like a Windows application, including its material effects.
- Accessibility must work without being built from scratch.
- The product is Windows-only by constraint (`CON-001`).

## Considered options

1. The platform's current native application framework, in C#
2. A cross-platform desktop framework
3. A web-based shell in a bundled browser runtime
4. Immediate-mode rendering for both surfaces

## Decision

**The platform's current native framework, in C#**, for both surfaces, as separate processes per [ADR-0003](0003-process-topology.md).

The overlay's requirements decide this.
Transparent, layered, click-through, non-activating, always-on-top, per-monitor scaling aware, and **excluded from capture** — that last one is not optional, and a framework that cannot reach it produces [a perception feedback loop](../explanation/overlay-rendering-explained.md).
All of those are platform capabilities, and reaching them from a cross-platform framework or a browser runtime means dropping to the platform anyway.

Given the platform is fixed by `CON-001`, cross-platform portability buys nothing.

A browser runtime for the shell would work and adds a large dependency and a second rendering stack for a window that mostly shows text and a video feed.

Immediate-mode rendering would suit the overlay's annotation layer well and would mean building the shell's accessibility, text input and theming by hand.

### Within the framework

**Annotations are drawn on a composition surface, not as interface elements.**
At the reflex rate with dozens of boxes, an element per box means a layout pass per box, and the framework will not keep up.
Interface elements carry the static chrome only.

**Material effects are for the shell, not the overlay.**
A blur-behind over a game samples the game's output every frame, costs graphics time the game wants, and lags behind moving content.

### Packaging

A single installable package containing all executables.
No elevation, no service registration, no driver (`CON-007`).

Model weights are fetched on first run rather than bundled.
Whether bundling would be permissible is a licensing question answered in the notices file; the reason not to is size and giving the user a choice of variant.

## Consequences

### Good

- Every window behaviour the overlay needs is directly available.
- Accessibility, theming, text input and data binding come with the framework.
- The shell looks like a Windows application, which is what its users expect.

### Bad

- Windows only, permanently. Acceptable under `CON-001`, and it does mean the interface layer is not reusable if that constraint is ever revisited.
- The framework's release cadence and its preview channels are outside the project's control, and the project is already carrying preview-stage risk elsewhere. Use the stable channel unless a specific capability forces otherwise.
- Contributors need the platform toolchain even to work on the overlay.

### Neutral

- The core is a different language, and the boundary between them is a process rather than a call. That is [ADR-0002](0002-core-runtime-language.md) and is not a cost attributable here.

## Validation

Revisit the overlay's framework specifically if the composition path cannot hold the reflex rate with a realistic number of annotations — in which case a dedicated rendering surface for that one window is the answer, not a different framework for both.

Revisit packaging if the weight download proves to be the main obstacle to a first run.
