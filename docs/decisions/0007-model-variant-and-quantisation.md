---
title: ADR-0007 — The model variant and its quantisation
description: Pre-registered. Which Gemma 4 variant and quantisation the product ships, with the quality axis fixed before any of them was measured.
status: proposed
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, model, pre-registered]
requirements: [FR-MODEL-001, FR-MODEL-004, FR-MODEL-005]
decisions: [ADR-0006, ADR-0014]
affects: [crates, contract]
spec: [docs/spec/17-model-contract.md]
evidence:
  - experiments/02-vision-encoder/findings.md
  - experiments/02-vision-encoder/results/2026-09-12-e4b-vulkan.txt
  - docs/known-good-matrix.md
supersedes: null
superseded_by: null
generated: false
---

# ADR-0007 — The model variant and its quantisation

!!! warning "Pre-registered: the Decision section is deliberately absent"

    Everything except the decision was written **before** the measurements that
    will settle it, and the Validation section in particular. A threshold
    written after seeing the table is a threshold the winner clears.

    This record is the one most at risk of that, because the known-good matrix
    has already argued for one of the candidates. See **The argument already in
    circulation**.

## Context and problem statement

The reference card has 24 GiB, and the game was there first.
A title at the high end takes 10 GiB, leaving under 14 for weights, the attention cache, the vision tower, the perception models and the capture surfaces.
That is the whole budget, and the variant is the largest single line in it.

Five variants and several quantisations are available.
The choice fixes tactical latency, graphics memory, how many frames the game keeps, and how well the agent finds things on screen — and those four do not order the candidates the same way.

Which variant and which quantisation does the product ship?

## The argument already in circulation

`docs/known-good-matrix.md` reasons from the MMMU Pro column that the tactical model "cannot be the weakest one available at looking at screens", which points at 26B A4B and rules out the arrangement of a small fast model for the tactical cadence and a large one for deliberation.

**That argument is a hypothesis and this record must not inherit it as a premise.**
MMMU Pro measures graduate-level multimodal reasoning.
Locating a button in a stylised heads-up display is a different task, and the same page records that no public grounding results exist for this family at all.

The matrix has been corrected to say so.
It is recorded here as well, because a record that quietly adopts the conclusion its own evidence page has withdrawn is how a guess becomes a citation.

## Decision drivers

- **Grounding accuracy on game interfaces**, which is the quality axis. Not MMMU Pro.
- **Tactical latency at the 95th percentile**, with a game resident.
- **Graphics memory for weights and cache**, against a 10 GiB game.
- **Frames left to the game**, measured rather than inferred from memory.
- **Whether the projector stays unquantised.** It does, for a reason in Validation below.

## Considered options

Six weight configurations, plus the projector question.

| Variant | Quantisations to measure |
| --- | --- |
| E4B | Q4_K_M, Q8_0 |
| 12B | Q4_K_M, Q5_K_M |
| 26B A4B | Q4_K_M, Q3_K_M or IQ3_M |
| 31B | Q3_K_M, for the benchmark only — almost certainly not shippable |

The projector is measured at F16 or BF16 for every family, and a quantised projector is measured only to demonstrate what it costs.

## Validation

Written before the measurements.

**One table, all six configurations, one row each**, with columns for grounding accuracy and its confidence interval, tactical latency at the 95th percentile with a game resident, weights plus cache in graphics memory, and the game's frame rate with and without the agent.
A configuration missing a cell does not win by default; it is not eligible.

**The quality axis is the grounding benchmark, not MMMU Pro.**
A record that ranks these by a reasoning benchmark has not answered the question this project asks.

**The threshold that would pick the smaller model, stated now:**

> If 12B at Q4_K_M lands **inside the confidence interval** of 26B A4B at Q3_K_M on the grounding benchmark, the 12B wins — on latency, on memory, and on frames left to the game.

That is written before the numbers exist precisely because the matrix has already argued the other way.

**The projector stays unquantised** unless the measurement shows otherwise, and the measurement has to be the grounding benchmark rather than a loading check.
A quantised projector is a known route to a model that loads, answers fluently and localises badly — the same shape of quiet failure the matrix warns about for a version mismatch, and the reason `experiments/02-vision-encoder/` runs a discriminative assertion rather than checking that the server started.

**What would change our minds after shipping:**

- **Tactical latency at the 95th percentile exceeds 500 ms** with a game resident. Past the upper bound in `15-performance-budgets.md`; the variant is reconsidered before the cadence is.
- **The mean frame rate falls below 90% of the value without the agent, or the 1% low below 80%.** Criterion 4 in the vision. A variant that fails it does not ship regardless of its grounding number.
- **A grounding gap of more than 10 points** opens between the shipped variant and one that fits the budget. Worth the migration.

## On what the digest proves

`FR-MODEL-005` requires a manifest, and the manifest carries per variant: the repository, the **revision commit — never a branch, because a branch moves and a digest recorded against it quietly stops matching**, the filename, the size, the SHA-256, the projector file with its own SHA-256, and the runtime commit the digest was validated against.
Internal GGUF metadata — architecture, quantisation type, tokeniser — is checked before loading rather than after, and the discriminative assertion from `experiments/02-vision-encoder/` runs as a self-check whenever a weights file changes.

**A digest proves the bytes match what this project measured. It does not prove authenticity.**
That would need a publisher signature, and there is not one.
This sentence belongs in Consequences → Bad when this record is decided, and it is written here so that it cannot be forgotten there: do not write "verified" and let a reader infer provenance.

## Pros and cons of the options

### E4B

- Good, because it is the smallest thing that might work, and it leaves the most memory to the game.
- Good, because it is measured and it sees: 7 of 8 synthetic scenes exactly correct on all three fields, at 148 generation tokens per second on the reference card.
- Bad, because MMMU Pro puts it lowest of the five — which is evidence about reasoning and, as above, is not the axis this record ranks on.

### 12B

- Good, because it is a dense model with a strong reasoning score and no routing behaviour to reason about.
- Good, because Q4_K_M fits the budget comfortably beside a 10 GiB game.
- Unmeasured on grounding, like every row here.

### 26B A4B

- Good, because only a fraction of its parameters are active per token, so its throughput is far better than its size suggests — which is also why it is the candidate for the processor-only fallback in the degradation ladder.
- Bad, because Q4_K_M is 16.9 GiB of weights before any cache, which does not fit beside a 10 GiB game. Q3_K_M at 12.7 GiB is the configuration that would actually ship.
- **This is the candidate the matrix argued for, and the one this record is most careful about.**

### 31B

- Good, because it scores highest on the published column.
- Bad, because it does not fit beside a game at any useful quantisation. Measured for the benchmark so that the table has a ceiling, not as a shipping candidate.

## More information

- `experiments/02-vision-encoder/` has the first row of the table, for E4B.
- `ADR-0006` chooses the runtime and is pre-registered alongside this record.
- `ADR-0014` depends on the grounding benchmark that this record's quality axis also uses.
- `docs/contributing/adr-process.md` explains why the Decision section is absent.
