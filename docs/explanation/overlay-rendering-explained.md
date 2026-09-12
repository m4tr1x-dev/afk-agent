---
title: Overlay rendering explained
description: "Compositing, transparency, click-through, and why the overlay sometimes does not appear."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [explanation, gui]
requirements: []
decisions: []
generated: false
---

# Overlay rendering explained

Drawing a status panel on top of a game sounds like a solved problem.
It is, for the case where you are allowed inside the game's rendering pipeline — which is exactly what this project [refuses to do](anti-cheat-and-terms-of-service.md).

Everything awkward about the overlay follows from that refusal.

## The window

The overlay is an ordinary top-level window with an unusual combination of properties:

- **Layered and transparent**, so the game shows through.
- **Click-through**, so the pointer passes to the game beneath.
- **Non-activating**, so it never takes focus.
- **Always on top.**

The click-through and non-activating properties are not conveniences.
Taking focus from the game would break the game, and it would also trip the agent's own foreground guard, which stops all input when the target is not in front.
An overlay that stole focus would halt the agent by existing.

## Why it draws on a composition layer

The obvious implementation is interface elements: a rectangle control per mark box, a label per number.

At thirty updates a second with fifty boxes, that is 1500 layout operations a second in a framework that was built for forms, and it will not keep up.

So annotations go onto a composition layer — one drawing surface, updated per tick, with no layout pass.
Interface elements are used only for the static parts: the goal text, the budget, the panic key reminder.

## Why there is no blur behind it

The platform's material effects are pleasant and the overlay does not use them.

A blur-behind effect samples what is underneath and processes it, which over a game means sampling the game's output every frame.
That costs graphics time the game wants, and it looks wrong over fast-moving content — the blur lags, which is more distracting than no effect at all.

The overlay uses flat semi-transparent panels with a clear edge.
Mica and Acrylic live on the shell — its navigation, its settings, its review surfaces — where they cost nothing and look right.

## The bug that makes everything strange

The overlay draws boxes.
The agent captures the screen.
Without one specific call, the agent captures the boxes.

Then it treats them as part of the game, proposes elements for them, draws boxes over those, and captures those too.
The element count climbs every tick and the agent's behaviour degrades into nonsense.

The fix is a single flag excluding the window from capture, and the reason it deserves this much space is that the symptom looks nothing like the cause.
Nobody's first guess is "the agent is looking at itself".

It applies to **every** window the overlay owns.
Tooltips and flyouts are separate top-level windows, they inherit nothing, and they are the ones that get missed.

## Display scaling

Two monitors at different scaling factors is an ordinary configuration and it is where overlays usually break.

The overlay is declared per-monitor scaling aware, which means the system hands it the scale factor rather than stretching it.
It rebuilds its transform when the factor changes, when the monitor topology changes, and when the game moves between monitors.

It performs no coordinate arithmetic of its own.
There is [one conversion chain](grounding-clicks-in-pixels.md) in the project, and the overlay uses it like everything else.

## Why it sometimes does not appear

Almost always exclusive fullscreen.

A composited window cannot draw over a surface the game owns exclusively.
The desktop compositor is not in the path, so there is nothing to compose the overlay into.

Four responses, in order:

1. **Usually it is not actually exclusive.** Modern Windows composites most nominally fullscreen games, and the overlay works normally. The genuinely exclusive case is now rare.
2. **Where it is, detect it and say so**, recommending borderless windowed — a one-line settings change in nearly every game, with no meaningful cost to the player.
3. **Fall back to the shell** on a second monitor, showing the annotated mirror. The user loses nothing but the convenience of looking at one screen.
4. **Never hook the game's rendering to draw inside it.** This is the technique that would work in every case. It requires injecting into the game's process, and it is precisely what the project does not do.

Point four is the trade.
An overlay that worked everywhere would need the one capability this project has given up, and the capability is worth more than the overlay.

## What it must always show

Whenever the agent can deliver input — not only while it is acting — the overlay shows that it can.

An agent that is thinking will move in half a second.
A user returning to the desk must never have to work out whether the thing on screen has hands, and the panic key binding is on the overlay rather than in a document for the same reason.

The overlay dying stops the session.
That looks harsh for a status window and it is deliberate: the overlay carries the only always-visible stop control, and an agent with hands and no visible stop control is the thing this project must never produce.
