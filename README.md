# afk-agent

An autonomous agent that plays PC games while you are away from the keyboard.

You write what you want done — *collect money and buy upgrades in this tycoon*, *get through this level*, *collect carrots* — and the agent watches the screen, works out what to do, and drives the mouse and keyboard until it is finished or you stop it.

> **Status: pre-implementation.**
> This repository contains a specification and no code.
> Nothing here runs yet.

## What makes it different

It is a **general computer-use agent**, pointed at games.
There is no per-game setup, no profile to configure, no script to write, and no training run.
It works the same way on a game released this week as on one released a decade ago, because the goal is stable even when the game is not.

When it meets a mechanic it does not understand, it searches the web the way an agent should, keeps what it learned in its working context for the rest of the task, and forgets it when the session ends.

The model runs **entirely on your machine**.
Nothing about your screen leaves the computer.

## What it deliberately does not do

- **It injects nothing.** No kernel driver, no code injected into any game, no function hooking, no reading game memory, no drawing inside a game's renderer. It captures the screen through the same public API a screen recorder uses and sends input through the same one an accessibility tool uses.
- **It does not learn.** The model's weights never change. Quality comes from perception, prompting, research and a verification loop — not from training on your gameplay.
- **It does not hide.** Synthesised input is detectable by design, and we make no attempt to evade detection.
- **It is not for competitive multiplayer**, for anything involving real money, or for accounts that are not yours.

## Before you point it at an online game

Automating an online game usually violates its terms of service, frequently regardless of whether anyone is harmed, and the consequence lands on your account rather than on this project.

Read [terms of service and anti-cheat](docs/explanation/anti-cheat-and-terms-of-service.md) first.
It is blunt about what is detectable, what is not, and what we will not build.

The supported case is single-player, offline and self-hosted games.

## Requirements

| | |
| --- | --- |
| Operating system | Windows 11 |
| GPU | A discrete GPU with enough memory for the model alongside the game |
| Disk | Several gigabytes for model weights, downloaded on first run |
| Network | Only for the research skills and the initial weight download |

Exact model variants and memory tiers are in the [model contract](docs/spec/17-model-contract.md).

## Documentation

| Start here | For |
| --- | --- |
| [Vision](docs/vision.md) | What this is for and how we will know it works |
| [Glossary](docs/glossary.md) | The vocabulary — this domain overloads ordinary words |
| [Specification](docs/spec/index.md) | What the system must do; the instruction set for building it |
| [Decision records](docs/decisions/index.md) | Why it is built this way, and what would change our mind |
| [Explanation](docs/explanation/index.md) | The arguments behind the hard parts |
| [Contributing](docs/contributing/index.md) | How to help — reviewing the specification is the most useful thing right now |

## Contributing

The most valuable contribution today is **reading the specification and telling us where it is wrong**.

There is no code, so everything is still cheap to change.
An incorrect assumption caught on a page costs an afternoon; the same assumption caught in a subsystem costs a fortnight.

See [contributing](docs/contributing/index.md), and read the [documentation guide](docs/contributing/documentation-guide.md) before writing a page — most of its rules are enforced by CI.

## Licence

MIT, for the code and the documentation — see [LICENSE](LICENSE).

Model weights are licensed separately by their publisher and are not distributed with this repository; they are downloaded on first run.
