---
title: Known-good matrix
description: Verified facts with their sources, pinned versions, and the register of open questions with what would resolve each.
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [reference, tooling]
requirements: []
decisions: []
generated: false
---

# Known-good matrix

The entire stack is on preview or recently released components, and a large part of the design rests on facts that were checked once, on one day.
This page separates what was verified from what was assumed, and gives every open question a concrete experiment rather than a research topic.

Two rules.
Anything in the **Verified** tables carries the date it was checked and the source.
Anything in the **Open questions** table is not to be built on until it is resolved, and resolving it means running the experiment, not finding a forum post.

## Reference hardware

The development and benchmarking machine.
Every measured number in the specification is measured here unless it says otherwise.

| | |
| --- | --- |
| GPU | AMD Radeon RX 7900 XTX, 24 GiB, RDNA3 |
| Processor | Intel Core i7-13700K, 16 cores |
| Memory | 32 GiB |
| Operating system | Windows 11 Pro, build 26200 |

The choice of vendor is load-bearing and shapes several decisions.
This is an AMD card, so there is no CUDA, and on Windows the practical inference path is Vulkan rather than ROCm.
Because the project does no training, **Vulkan is the only GPU path in the system** — there is no second toolchain and no seam between a training runtime and an inference runtime.

## Verified — models

Checked 2026-09-12 against the Gemma 4 model card.

| Variant | Parameters | Context | Modalities | MMMU Pro |
| --- | --- | --- | --- | --- |
| E2B | 2.3B effective | 128K | text, image, audio | 44.2% |
| E4B | 4.5B effective | 128K | text, image, audio | 52.6% |
| 12B | 11.95B | 256K | text, image, audio | 69.1% |
| 26B A4B | 3.8B active of 25.2B | 256K | text, image | 73.8% |
| 31B | 30.7B | 256K | text, image | 76.9% |

Also verified: the per-image visual token budget is configurable, with supported values of 70, 140, 280, 560 and 1120.
Native function calling and configurable thinking modes are documented.
The weights are published under Apache 2.0, which means they may be redistributed — relevant to whether an installer may bundle them rather than fetching them.

**The gap between E4B and 26B A4B on vision is the reason the model configuration looks the way it does.**
A model that inspects the screen several times a second cannot be the weakest one available at looking at screens, which rules out the intuitive arrangement of a small fast model for the tactical cadence and a large one for deliberation.

## Verified — the grounding baseline

Checked 2026-09-12.

A parser-plus-general-model pipeline reaches roughly 39.5% on a high-resolution GUI grounding benchmark.
Purpose-built native GUI agents do considerably better — a recent 32B specialist reports 72.9% on the same benchmark, and 80.3% with a two-stage refinement pass.

Two things follow, and only the second is directly usable.

The specialist-versus-generalist gap is not available to us, because we do not train.

The **refinement gap is**: the same model, asked twice with a localisation step in between, gains a substantial margin over the same model asked once.
That is a property of how the model is queried rather than of the model, and it is why the grounding design uses a cheap wide pass followed by an expensive narrow one.

**No public GUI grounding benchmark results exist for Gemma 4**, and the model card makes no computer-use claim.
Its direct coordinate grounding ability is therefore unknown, in both directions, and question 4 below exists to settle it.

## Verified — platform

Checked 2026-09-12.

| Component | State |
| --- | --- |
| Windows App SDK | 2.4.0 stable, released 2026-08-13. The 1.8 line left maintenance on 2026-09-09. |
| .NET | **10.0.401 is what is installed**, and it is the long-term support release. 11 is at release candidate with a go-live licence; general availability 2026-11-10, **standard-term support**. |
| C# | 14 with .NET 10; 15 ships with .NET 11. |
| Rust | 1.98.1, installed 2026-09-12. `rustup` 1.29.1. |

Use the **stable** Windows App SDK channel, not the experimental one, unless a specific API forces otherwise.
The project is already carrying enough preview-stage risk elsewhere.

**Target .NET 10 rather than 11**, and the reason is in the table above: 10 is the long-term support release and 11 is standard-term.
A product that ships an installer to users belongs on long-term support unless a capability forces otherwise, and the earlier version of this page recorded the support terms without drawing that conclusion.

The one cited reason for 11 was C# 15 union types, which suit the inter-process message envelope and the tool-result sum type.
That reason weakens under the contract design: the union is a representation the code generator owns, so the closed-hierarchy form emitted today and the union form emitted after general availability differ by a generator flag rather than by hand-written code.

Revisit when .NET 11 reaches general availability **and** the stable Windows App SDK channel declares support for it.
That is one pull request bumping `global.json`, the target frameworks and the generator flag.

The Rust toolchain is pinned to an exact version rather than to `stable`.
Clippy gains lints every six weeks and the coding standards run it with warnings denied, so on `stable` a toolchain release turns an unrelated pull request red on a morning when nothing in this repository changed.

## Pinned versions

| Component | Pin | Where |
| --- | --- | --- |
| Documentation toolchain | Exact versions, **resolved and verified 2026-09-12** | `docs/requirements.txt` |
| GitHub Actions | By major version today; move to commit hashes before a public release | `.github/workflows/` |
| .NET SDK | 10.0.401 with `rollForward: disable`, **verified 2026-09-12** | `global.json` |
| Rust toolchain | 1.98.1, exact, **installed and verified 2026-09-12** | `rust-toolchain.toml` |
| NuGet packages | Central package management | `Directory.Packages.props`, not yet created |
| Inference runtime | To be pinned by commit, recorded in the model metadata | Not yet created |

### Verified toolchain

Installed and run on the reference machine on 2026-09-12.
Every documentation check below passed with zero findings.

| Tool | Version | Checks |
| --- | --- | --- |
| Python | 3.12.10 | Site build, front-matter and identifier checks |
| MkDocs | 1.6.1 | `mkdocs build --strict` |
| Material for MkDocs | 9.7.7 | |
| PyMdown Extensions | 11.0.2 | Snippets, Mermaid fences |
| Node.js | 24.19.0 | Markdown and spelling checks |
| markdownlint-cli2 | 0.23.2 | 0 issues |
| cspell | current | 0 issues |
| Vale | 3.17.1 | 0 errors, with the Microsoft style package |

**A risk surfaced by the build itself.**
Material for MkDocs prints a warning that the next major version of the
underlying framework removes the plugin system and rewrites theming, with no
migration path.

That affects this project directly: the status banner is a theme override and
the snippet mechanism is a plugin extension, and both are load-bearing.

The mitigation is the pin, which is already in place.
The consequence is that the documentation toolchain is on a branch that will not
receive the next major version, and a future decision to move will be a
migration rather than an upgrade. Recorded here rather than discovered later.

The inference runtime pin matters more than it looks.
Model file formats churn, and a mismatch between the version that produced a file and the version that reads it can fail quietly — the file loads, and the results are wrong.

## Open questions

Ordered by what blocks the most.
Each is an experiment with an end, not a topic.

| # | Question | Experiment | Blocks | Resolved |
| --- | --- | --- | --- | --- |
| 1 | Does synthesised relative mouse movement reach a real game at all? | Build the reachability probe: a minimal program that captures a window, synthesises relative movement, and confirms the camera turned. Run it against five games of different engines. | Everything. If this fails there is no project. | **Four engines tested, four reached, 2026-09-12.** Breathedge (Unreal Engine 4), Xonotic (DarkPlaces, 0.892 columns per unit at R2 0.957), AssaultCube (Cube, 1.072 at R2 1.000, 0.0909 degrees per unit) and OpenArena (id Tech 3, 1.091 at R2 0.975, 0.1111 degrees per unit). Four of four is stronger than the four-of-five the criterion asks for, but the denominator is not the one the protocol names, so this is recorded rather than closed. Two of the four **apply** an absolute reposition rather than ignoring or clamping it, which is a third outcome `06-action-and-input.md` did not allow for and now does. See `experiments/01-reachability/results/`. |

| 2 | Does the vision encoder load on the Vulkan backend for the 26B A4B variant? | Start the model host with the projection file on the reference hardware and send one image. | Model contract | |
| 3 | What are the latency and memory figures at visual budgets of 140 and 1120, at 8K and 16K context, **with a game running**? | Measure. The figure without a game in memory is not the figure that matters. | Model contract, performance budgets | |
| 4 | How good is Gemma 4 at direct coordinate grounding on game interfaces? | Record 200 to 500 frames across three to five games, label the true element boxes, and measure both grounding paths. | Grounding, and the grounding decision record | |
| 5 | Does the prompt prefix cache behave as assumed with two pinned slots at different visual budgets? | Measure prefix reuse across consecutive ticks on each slot. | Model contract | |
| 6 | What is the 26B A4B throughput on the processor alone? | Measure on the reference hardware. This is the fallback when the game needs the graphics memory, so the number decides whether the fallback is usable. | Model residency | |
| 7 | Can the capture border be suppressed for an unpackaged application on this Windows build? | Prototype. | Overlay, capture | |
| 8 | What do the current terms of service of the games we intend to use as examples actually say about automation? | Read them. Not a summary, and not a similar game's terms. | Vision, and the terms-of-service decision record | |
| 9 | Does the newer solution file format work in the installed Visual Studio, and does ahead-of-time compilation work with the UI framework? | Prototype both. | Repository layout | |

Question 1's experiment column says the probe "confirms the camera turned",
which is not a method.
It is one now: each frame is reduced to a profile of column intensities over its central band, the two profiles are cross-correlated, and the lag at the peak is how far the world moved — with four conjunct tests and four null controls around it.
The method and what it cost to get right are in `experiments/01-reachability/findings.md`.

One of those controls is worth repeating here, because it was learned the expensive way.
A correlation of exactly 1.000 is not a measurement; it is the signature of comparing a frame with itself.
The probe's first run reported a confident, complete and wrong "not reached" on that basis, and the project's gating question deserves better than an answer nobody checked.

Question 4 is the one worth doing carefully.
It is the only one whose answer is not available anywhere, it decides an architectural default, and it is cheap — a few hundred labelled frames and an afternoon.

## What is no longer on this list

An earlier iteration of the design included a training pipeline, which brought four further open questions with it: whether the training framework worked on this vendor's stack on Windows, whether an adapter survived conversion into the inference format, whether adapters could be applied over a quantised base on Vulkan, and whether changing an adapter invalidated the prompt cache.

That direction was dropped — [the agent does not learn](explanation/why-the-agent-does-not-learn.md) — and those questions went with it.

The fourth was the dangerous one.
If changing an adapter did not invalidate the cached prompt prefix, the cached prefix would have been computed by a different model from the one decoding the rest.
It would not have crashed.
It would have made the agent subtly worse in a way indistinguishable from a bad adapter, and the investigation would have started in the wrong place.

## Maintaining this page

Add a row the moment something is assumed rather than known.
Fill in the resolved date when the experiment runs, and move the finding into the verified tables with its source.

A question that has been open for months without its experiment being run is telling you that it is not actually blocking anything, or that the project is building on it anyway.
Both are worth noticing.
