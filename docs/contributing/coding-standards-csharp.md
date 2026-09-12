---
title: C# coding standards
description: "Conventions for the shell and the overlay: nullability, analysers, asynchronous code, and documentation."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing, csharp]
requirements: []
decisions: []
generated: false
---

# C# coding standards

For the shell and the overlay.

Most of this is ordinary.
The parts specific to this project concern the overlay, which has constraints no ordinary window has.

## Language and analysers

Nullable reference types enabled, and nullable warnings are errors.
A project that enables nullability and then ignores its warnings has the annotations without the guarantee.

Analysers run at the strictest useful level, warnings as errors.
A rule that is wrong for a case is suppressed at the narrowest scope with a justification.

Formatting is settled by the editor configuration and is not a matter of opinion in review.

## Documentation

Documentation comments are required on public members of any project that feeds the generated API reference, and the missing-comment warning is an error there.

Without that, the generated reference is a list of names.

## Asynchronous code

Asynchronous all the way down.
No blocking on a task, ever — it deadlocks against the interface thread, and it does so intermittently, which is the worst way to find out.

Cancellation tokens are accepted and honoured.
A session that is stopping should not be waiting on a request that nobody cares about any more.

Every call across the process boundary has a timeout.
A shell that hangs because the core is busy is a shell whose stop button does not work, at the moment it is most needed.

## The interface thread

The shell performs no inference, no capture and no perception.
It renders what the core publishes.

That is a structural property from the [process split](../explanation/rust-and-csharp-split.md) rather than a discipline, and it is why freezing under load is not expected.

Even so: work triggered by interface events that could take more than a few milliseconds moves off the thread.

## The overlay

Different rules, because it is not an ordinary window.

**Annotations are drawn on a composition surface, not as interface elements.**
At thirty updates a second with dozens of boxes, an element per box means a layout pass per box, and the framework will not keep up.
Interface elements are for the static chrome only.

**Every overlay window is excluded from capture.**
Including tooltips and flyouts, which are separate top-level windows and inherit nothing.
Missing one produces [a perception feedback loop](../explanation/overlay-rendering-explained.md) whose symptom looks nothing like its cause.

**The overlay never takes focus and never intercepts a click.**
Doing either breaks the game and trips the agent's own foreground guard, which stops all input when the target is not in front.

**No material effects on the overlay.**
A blur-behind over a game samples the game's output every frame, costs graphics time the game wants, and lags behind moving content.
Mica and Acrylic belong on the shell.

## Interface architecture

Model-view-viewmodel with dependency injection, which is what the framework expects and what its data binding is built for.

Code-behind is for view concerns only.
Logic in code-behind is logic that cannot be tested without a window.

## Strings

Every user-visible string is externalised from the first commit.

This is the one place in the project where a language other than English is expected, and retrofitting externalisation is tedious while doing it from the start costs nothing.

## Accessibility

Every control is keyboard reachable and labelled for assistive technology.
Contrast meets the platform guidance in both themes.

The overlay is a status display rather than an interactive surface, so its obligation is different: everything it shows is also available in the shell and in the logs, because an overlay is useless to a user who cannot see it.

## Naming

Follow the platform conventions.
Use the vocabulary from the [glossary](../glossary.md) — a control named after a concept the specification calls something else makes every conversation slower.

## Requirement identifiers

Code implementing a requirement carries its identifier in a comment, and its test carries it in an attribute.

The traceability report is generated from those markers.
