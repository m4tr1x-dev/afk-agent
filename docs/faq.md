---
title: Frequently asked questions
description: "Short answers to the questions this project reliably provokes."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [faq]
requirements: []
decisions: []
generated: false
---

# Frequently asked questions

## Will I get banned?

Possibly, and it is your account.

Automating an online game is usually prohibited by its terms, frequently regardless of whether anyone is harmed.
Detection is also possible: synthesised input is marked by Windows, any process can read that mark, and we make no attempt to hide it.

The supported case is single-player and offline games.
Read [terms of service and anti-cheat](explanation/anti-cheat-and-terms-of-service.md) before pointing this at anything else — it is blunt about what is detectable and what we refuse to build.

## Does it send my screen anywhere?

No.

The model runs on your machine.
Screen contents are not transmitted, and there is no setting that changes that.

The only network traffic the agent generates comes from the research skills: a search query and a page fetch.
Recognised screen text is never sent verbatim, because the agent cannot tell which parts of your screen are private.

Session recordings are images of your screen and they stay on disk, quota-bounded, deletable in one action. See [observability](spec/13-observability.md).

## Does it read game memory or inject code?

No. None of it.

No kernel driver, no code injection, no function hooking, no reading another process's memory, no modifying game files, no drawing inside a game's renderer.

It captures the screen through the same public API a screen recorder uses and synthesises input through the same one an accessibility tool uses.
That is the whole surface, and it is an architectural commitment recorded as `CON-002` rather than a current state of the code.

## Is this a cheat?

It automates play, which for a single-player game is between you and the game.

We do not use that word about this software, for two reasons.
Technically it is wrong — nothing is injected, hooked or modified.
And it is the exact word that appears in the terms of service we are asking you to read carefully, so using it concedes a characterisation we do not accept.

None of which means a given game permits it. That is a separate question, and the answer is frequently no.

## Why does it need such a large graphics card?

Because the model runs locally, and it has to fit alongside the game.

The game typically wants 4 to 10 GiB.
The model, its vision encoder and its attention cache want the rest.

When the game needs more, the agent degrades in a fixed order rather than competing: shorter context, then cheaper images, then the slow model moves to the processor. See [performance budgets](spec/15-performance-budgets.md).

You can also run the model on **another machine on your own network**, which is the best answer when the game wants all of the graphics memory. The endpoint is a setting.

## Why not use a hosted model

It would, and it would also mean streaming continuous capture of your screen to someone else's computer.

That trade is not one this project makes. `CON-003`.

## Does it get better the more I use it?

No, and that is deliberate.

The model's weights never change.
The agent does not learn from your gameplay and carries nothing from one session to the next.

Within a session it does get faster — it remembers interface elements it has already used and stops asking the model about them — but that is a cache, and clearing it makes the agent slower rather than worse.

The reasoning is in [why the agent does not learn](explanation/why-the-agent-does-not-learn.md).

## So how does it know anything about my game?

It looks.
And when looking is not enough, it searches the web, exactly as a person handed an unfamiliar game would.

What it finds goes into its working context and stays there for the rest of the task.
When the session ends, so does that knowledge — which is also why a mechanic from one game never leaks into another.

## Can it play *my* game?

Unknown until you try, and there is deliberately no compatibility list.

A list would become a gate, and a gate would become per-game configuration, and per-game configuration is the thing this project exists to avoid.

Two things make it more likely to work: the game runs in windowed or borderless-windowed mode, and its interface is readable at the capture resolution.
The first thing to verify is whether synthesised input reaches the game at all — some games ignore it, and that is question one in the [known-good matrix](known-good-matrix.md).

## Why Windows only?

Capture, input synthesis and window composition are platform-specific to the point where a port would be a rewrite of every layer below the reasoning loop.

It is a resourcing judgement rather than a principle, and it is the softest entry in [non-goals](spec/19-non-goals.md).

## Why is it slow?

Because a vision model is slow, and the agent is looking at your screen several times a second.

The design works around this rather than solving it: three loops at different rates, so the fast one keeps playing while the slow one thinks.
A deliberative pass taking fifteen seconds is invisible because the agent is still acting on its previous plan throughout.

## What happens if I come back to the computer?

It stops, immediately, on the first key or mouse movement you make.

Windows marks synthesised input, so the agent can tell your input from its own.
It releases everything it was holding, shows why it paused, and waits for you to resume it deliberately.

There is also a panic key that works even if the agent is wedged, and it releases everything held within 100 ms.

## What if it crashes while holding a key down?

A separate small process watches the core and releases everything it was holding.

This is the failure that matters most in input automation: a key left held that you cannot release from outside the game. `FR-SAFE-003` exists for it, and so does the guardian process.

## Can it spend my money or trade my items?

Not without you confirming it.

Real currency, trading, deleting and account settings are behind confirmation gates enforced in code, below anything the model can reach.

Recognising a destructive action without understanding the game is genuinely unsolved, and it is the weakest part of the design — see [safety and limits](spec/12-safety-and-limits.md).

## Can I use it to farm accounts for sale?

No, and that is not a use case this project accepts contributions for.

## When can I try it?

There is no build.

The repository contains a specification and no implementation.
The [roadmap](roadmap.md) describes what has to be true before code starts, and the first milestone is a set of experiments that could still show the approach does not work.
