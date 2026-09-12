---
title: Vision
description: What afk-agent is for, what it deliberately is not, and how we will know it works.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [vision, product]
requirements: []
decisions: []
generated: false
---

# Vision

Every trade-off in this project appeals to this page.
When two designs are defensible and the arguments tie, the one that better serves what is written here wins.

## The problem

A large amount of time in games is spent on work that is not play.
Grinding currency, walking a farming route, clicking through an upgrade tree, waiting out a timer, repeating a level for a drop.
People solve this today with per-game scripts and macro tools, and those solutions share one weakness: they encode the game, not the goal.
The game patches, the interface moves, and the script is dead.

The alternative most people reach for next is worse.
A per-game script has to be written by someone who already knows the game, for a game with an audience large enough to justify the work.
For everything else — a game released last week, a small game, the specific thing you personally want done — nothing exists.

## The thesis

**A general computer-use agent beats a per-game script, because the goal is stable and the game is not.**

If an agent can see a screen and operate a mouse and keyboard, and if it can read a guide when it does not know something, then it does not need to be told about the game in advance.
The instruction "collect money and buy upgrades in this tycoon" contains everything a competent person would need, and a competent person would not require a training course on the specific tycoon before starting.

This is the same shape as computer-use agents on the desktop, pointed at a harder target.
Games are harder than desktop applications in ways that matter: no accessibility tree, no document object model, stylised interfaces that general-purpose vision models have not been trained on, real-time state, and inputs that must persist across frames rather than firing once.
Those are the problems this project is actually about.

## Principles

These are not preferences.
A design that violates one of them is wrong regardless of its other merits.

**Local first.**
The model runs on the user's machine.
Screen contents never leave it.
This is a privacy position, but it is also a practical one: continuous screen capture sent to a hosted model would be expensive, slow, and an entanglement with a third party's terms on top of the game's.

**Out of process, always.**
No kernel component, no code injected into any game, no function hooking, no reading another process's memory, no drawing inside a game's swap chain.
The agent observes through public capture APIs and acts through the public input API.
This is not a stealth strategy — synthesised input is trivially detectable and we say so plainly.
It is the posture that makes the tool defensible: everything it does is something an accessibility tool does.

**The user is always in control.**
A panic key that works even when the core is wedged.
Automatic pause the instant a human touches the keyboard.
A visible indicator whenever the agent has hands.
Hard limits on time and on actions that the model cannot raise.

**Game-agnostic by construction.**
No per-game profiles, no per-game training, no per-game code paths.
Anything the agent learns about a game lives in its working context for the duration of the task and then is gone.
If a feature only works because someone configured this particular game, it is not the feature we want.

**The agent does not learn.**
The model's weights never change.
Quality comes from perception, prompting, research and a verification loop — not from training on the user's gameplay.
This is a deliberate limit and it is explained at length in [why the agent does not learn](explanation/why-the-agent-does-not-learn.md).

## Who it is for

**The person with a grind.**
Has a game they enjoy and a repetitive part they do not.
Wants to write one sentence and walk away.
Does not want to install a per-game tool, join a Discord, or learn a scripting language.

**The tinkerer.**
Interested in what a local model can actually do when given hands.
Will read the specification, run the benchmark, file an issue about the grounding arbiter.
This person is the project's early contributor base and the reason the specification is public.

**The researcher.**
Wants a reproducible computer-use harness on a hard, visually stylised domain, running entirely on consumer hardware.
Cares about the evaluation corpus more than the product.

## Success criteria

Measurable, and deliberately modest.
A first version that does one thing reliably is worth more than one that attempts everything.

1. **Cold-start competence.** Given a natural-language goal and a game the agent has never seen, it completes a defined twenty-minute objective in at least seven out of ten runs, with no per-game configuration of any kind.
2. **Generality across a set.** The same build, unmodified, achieves that on at least five games spanning three genres, including at least one real-time game and one menu-driven one.
3. **Safety is absolute.** Across every hour of testing, the panic key stops all input within its budget, no key is ever left held after a stop, and no input is ever delivered to a window other than the target. These are not statistics; a single failure is a defect.
4. **It does not degrade the game.** With the agent running, the game's frame rate stays within a stated fraction of its unattended value on the reference hardware. An agent that makes the game stutter will be uninstalled regardless of how well it plays.
5. **Research pays for itself.** On tasks that require knowledge the agent does not have — where a shop is, what unlocks a stage — enabling the research skills measurably improves the completion rate over disabling them. If it does not, the skills are decoration.

## Non-goals

Stated as firmly as the goals, because this is the list that gets eroded.

- **Competitive multiplayer.** Not supported, not tested, and not a use case we will accept contributions for.
- **Evading anti-cheat systems.** Never. Not obfuscation, not input-timing camouflage sold as realism, not process-name hiding. If a game's protection notices the agent, that is the correct outcome.
- **Account farming at scale.** One user, one machine, their own account.
- **Cloud inference.** The model is local. A remote model endpoint on the user's own network is a supported deployment; a hosted API is not.
- **Learning from gameplay.** See the principles above.
- **Platforms other than Windows 11.** Not now, and probably not later; the capture, input and composition layers are deeply platform-specific.
- **Being a general desktop automation tool.** The target is a game window. Constraining the blast radius to one window is a safety property, not a limitation to be removed.

## The ethical position

This software automates play.
For a single-player game that is between the player and the game, and we think it is unambiguously fine.

For an online game it is usually against the terms of service, frequently regardless of whether anyone is harmed, and the consequence lands on the user's account rather than on us.
We do not pretend otherwise, we do not help anyone hide, and we say so in the interface and not only in a document nobody reads.

We will not build features whose only purpose is to make automated play harder to detect.
Where that limits what the project can do, the limit stands.

The longer argument, including what anti-cheat systems actually detect and why our posture is what it is, is in [terms of service and anti-cheat](explanation/anti-cheat-and-terms-of-service.md).
