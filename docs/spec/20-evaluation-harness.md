---
title: Evaluation harness
description: "The recorded corpus, the grounding benchmark, and the regression suites."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, evaluation]
requirements: [FR-OBS-002, FR-GND-004, FR-GND-008, INV-OBS-001, NFR-MODEL-001]
decisions: [ADR-0014, ADR-0036]
generated: false
---

# Evaluation harness

## Purpose

This page specifies how the project measures itself.

It exists early because two decisions are blocked on measurements this harness produces, and because a project whose central component is stochastic cannot be tested by running it and looking.

One tool serves three purposes: the grounding benchmark, regression testing, and deterministic replay of a failure.

## The corpus

A set of recorded frames and sessions, labelled, committed to the repository or to a release artefact.

| Set | Contents | Used for |
| --- | --- | --- |
| **Grounding** | Individual frames with true element boxes and a described target | Measuring both grounding paths |
| **Sessions** | Complete recordings: frames, observations, prompts, actions, outcomes | Replay, regression |
| **Perception** | Frames with true text, element boxes and scene class | Measuring extraction |

### Collection

Frames come from real games played by a human, captured with the agent in dry-run mode.
Dry run means the perception pipeline runs in full while nothing is synthesised, which is what makes collection safe on any game.

**Breadth matters more than depth.**
Frames from five games across three genres are worth more than the same number from one, because the thing being measured is whether the approach generalises.

At least one game is held aside entirely, never used while tuning, so there is one measurement that has not been fitted to.
That rule is enforced by the harness rather than by intention: the held-aside game lives in a separate directory, the harness refuses to read it without `--final`, and `--final` writes an immutable timestamped result file.
The failure mode here is convenience rather than dishonesty, so the mechanism has to be mechanical.

**Size: 1000 to 1500 frames, not the few hundred this page first named.**
The arithmetic is what changed the number, and it is worth stating because the original figure does not support the reporting this same page requires.

At 300 frames the standard error on a proportion near one half is 2.9 points, so the interval is about ±5.7 — usable in aggregate.
But the grounding results are reported **per scene class and per game**, and with five games and three or four classes the count per cell falls to 30–60, where the interval widens to ±13–18 points.
That is too wide to decide anything per cell.

So: the **aggregate with its interval is the decision statistic**, and per-cell figures are printed with their own intervals and labelled directional.
Collection is cheap enough to make the larger number reachable, because the method below records only frames where something changed.

### Labelling

Three labels, with very different exposure to circularity, and the distinction is the most useful thing on this page.

| Label | Circular when produced by |
| --- | --- |
| The true box | The perception pipeline — which is the thing under test for `FR-PERC-005` |
| The target description | The model under test — which measures self-agreement, not accuracy |
| Reachability per path | Both |

**The true box comes from interaction, not from a detector.**
A person plays each title for 15 to 20 minutes with the recorder running, and the recorder captures **their own clicks**.
A click at a point followed by a screen change within 200 ms gives a point that is *by definition* inside a real interactive element, the frame immediately before it, and evidence that the element was interactive — with no model, no detector, and no circle.

This is not a new capability.
`FR-SAFE-002` already requires observing the user's real input and distinguishing it from synthesised input, and `ADR-0025` excludes hooking the *game* and injecting into the *game's* process.
Observing the user's own input inside the agent's own process is neither.

A box is accepted when the click point lies inside a box from one source and a second source agrees with it at an intersection over union of 0.5 or better.
Sources are the accessibility tree where the game exposes one, classical proposals, and the changed region after the click.
Everything else goes on a **rejected pile that is counted and reported**, because its size is itself a finding about how complete perception is.

Each accepted label also records **`proposed_by`**: which perception sources put forward this element in the run that produced the frame.
It is written from a real perception pass and reviewed by hand, which makes it an observation rather than an assumption — and it is what answers open question 4 with data instead of a definition.

**The description comes from anywhere except the model under test.**
In order of preference: derived from text, where the element carries readable text and the description is that text plus a generic noun; typed by a person, for targets in the world with no text, which is 50 to 80 descriptions and ten minutes of scrubbing a recording; or drafted by a different model family and accepted by a person.

### What these numbers deserve to be used for

Stated here rather than left for a reader to assume, because the wrong use of them is one sentence away.

- **The relative comparison of Path A, Path B and refinement is sound in aggregate.** All three are measured against the same ground truth, so its bias applies to all three equally. This is the number `ADR-0014` actually needs.
- **The absolute hit rate is not sound**, and must never be compared with published figures from other benchmarks. Those have different labels, different tasks and different definitions of a hit.
- **The benchmark over-represents easy targets.** Two-source agreement accepts what is easy to detect, so the measured rate is an **upper bound rather than an estimate**. The rejected pile quantifies the bias: if two clicks in five land on something no source boxed, the benchmark covers three fifths of the real problem, and that fraction goes in the report.

### Privacy

Corpus frames are images of a real screen.
Anything captured for the corpus is reviewed before it is committed, and games are played on an account created for the purpose.

## The grounding benchmark

The measurement that unblocks `ADR-0014`.

For each labelled frame:

1. Run perception, producing elements and marks.
2. Ask the model for the described target via **Path A**, direct coordinates.
3. Ask via **Path B**, mark selection.
4. Ask via **two-stage refinement**.
5. Record, for each, whether the result falls inside the true box.

Reported per scene class and per game, not only in aggregate.
An approach that is excellent on menus and useless in a world view has an aggregate number that describes neither.

### What it decides

- Whether Path A is viable at all for this model.
- The arbiter's global starting preference.
- Whether the refinement stage earns its extra call.
- Where perception is the limiting factor rather than the model.

That last one is why the perception set exists separately.
A frame where the target was never proposed is a perception result, and reporting it as a grounding failure would send the next month of work to the wrong subsystem.

## Replay

`FR-OBS-002`, `INV-OBS-001`.

A recorded session replays deterministically: the same recording and the same sampling seed produce the same action sequence.

Used for three things:

- **Regression.** A change that alters behaviour on a recorded session shows up as a diff in the action sequence.
- **Debugging.** A failure that happened overnight is reproducible on demand.
- **Cheap iteration.** Perception and planning changes can be evaluated without a game running.

### Where determinism breaks, and what to do about it

Two places, both known.

**Compaction is a model call**, so a replay that crosses a compaction boundary may diverge from the original run.
The harness records compaction results in the recording and replays them from there rather than re-deriving them.

**The model host may not be bit-reproducible** across versions or hardware.
Replay is therefore pinned to a runtime version, and a version change invalidates the expected outputs rather than silently producing failures.

## Regression suites

| Suite | Asserts | Runs |
| --- | --- | --- |
| Perception | Text and element extraction against labels, within tolerance | Every pull request |
| Grounding | Hit rate per path, not worse than the recorded baseline | Every pull request |
| Replay | Recorded sessions produce their recorded action sequence | Every pull request |
| Coordinate transform | Exact mapping under at least four scaling and monitor configurations | Every pull request |
| Safety | Release on stop, panic, focus loss and crash | Every pull request |
| Latency | Per-phase budgets from [performance budgets](15-performance-budgets.md) | Nightly, on reference hardware |

The coordinate suite is the one that looks least interesting and catches the most.
A transform defect under mixed display scaling is invisible in ordinary testing and breaks every grounded action for the users who have that configuration.

The safety suite is not a statistic.
A single failure is a defect, not a degraded score.

## Measuring honestly

Three rules, because a benchmark that flatters is worse than none.

**Split by session and by time, never at random.**
Adjacent frames are nearly identical. A random split puts a frame's neighbour in the other half and produces a number that is both excellent and meaningless.

**Report an interval, not a point.**
At a few hundred samples the standard error on a proportion near one half is a couple of percentage points, which means a two-point improvement is one standard error — not evidence. The harness reports a confidence interval, and a change that does not clear it is noise.

**Measure in the deployment path.**
The benchmark runs against the same runtime, the same quantisation and the same configuration the product uses, not a convenient approximation. The gap between those two is the standard source of results that look fine and behave worse.

## The blame histogram

`FR-GND-008`.

Not a suite but the most useful thing the harness produces: the distribution of failure causes over real sessions.

It answers "what should be fixed next", and it answers it with evidence rather than with the instinct to blame the model.
A system whose text recognition misreads a counter presents as a system whose model makes poor decisions, and only the histogram separates them.

## Open questions

1. **Blocking.** Whether the corpus can be published. It is images of real games, which raises questions this project has not answered.
2. **Blocking.** Whether replay should tolerate small divergence or require exact equality. Exact is a clear signal and will make every model-host update look like a regression.
3. **Non-blocking.** Whether `in_mark_list` and `visually_distinct` are the right two replacements for the per-path reachability flag. The flag as written was not well defined — Path A may emit any coordinate, so every element is nominally reachable by it and the flag is always empty — and it conflated two different facts. It is replaced by `in_mark_list`, which is **computed and reported** rather than labelled, and a separate human label `visually_distinct` on the perception set. Whether that pair is sufficient is what remains open.
4. **Limitation.** Whether the latency suite can run anywhere but the reference machine. Almost certainly not, which makes it a nightly job on a self-hosted runner.
5. **Non-blocking.** What baseline the grounding suite compares against before there is a baseline.

## Related decisions

`ADR-0036` records the verification model this harness measures and is accepted.
`ADR-0014` is proposed and depends on this harness existing first.
`ADR-0026`, the testing strategy for a non-deterministic agent, is not written.
