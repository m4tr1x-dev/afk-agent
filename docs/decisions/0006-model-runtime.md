---
title: ADR-0006 — The local model runtime
description: Pre-registered. Which inference runtime hosts the model, with the capability ladder written before any of them was measured.
status: proposed
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, model, runtime, pre-registered]
requirements: [FR-MODEL-001, FR-MODEL-002, FR-MODEL-003]
decisions: [ADR-0013]
affects: [crates, contract]
spec: [docs/spec/17-model-contract.md]
evidence:
  - experiments/02-vision-encoder/findings.md
  - experiments/02-vision-encoder/results/2026-09-12-e4b-vulkan.txt
  - experiments/02-vision-encoder/run.sh
supersedes: null
superseded_by: null
generated: false
---

# ADR-0006 — The local model runtime

!!! warning "Pre-registered: the Decision section is deliberately absent"

    Everything here except the decision was written **before** the measurements
    that will settle it. That ordering is the point, and it is the Validation
    section that carries it: a threshold written after seeing the numbers is a
    threshold the winner clears.

    `docs/contributing/adr-process.md` requires this for any record whose
    decision is a number, a ranking, or a choice a measurement distinguishes.

## Context and problem statement

`CON-003` puts the model on the user's own machine or a host they control, and the reference hardware is an AMD card with no CUDA.
That removes most of the inference ecosystem before the question is even asked.

The runtime is not a component that can be swapped later without cost.
`ADR-0013` makes grammar-constrained decoding the mechanism the whole safety argument rests on, and `17-model-contract.md` assumes two pinned contexts at different image costs.
A runtime lacking either is not a slower runtime; it is a different architecture.

Which local inference runtime hosts the model?

## Decision drivers

Seven capabilities, and the order below is the order in which they are given up.
That ordering is written here, before any runtime was measured, because otherwise the fallback we choose is whichever one turns out to be convenient.

1. **Grammar-constrained decoding. Not negotiable.** `ADR-0013` is accepted and the safety argument in `ADR-0014` stands on it. If no Vulkan-capable runtime offers it, the correct answer is to implement constrained decoding as a logits processor, not to relax the requirement.
2. **Vulkan. Not negotiable.** It is a hardware fact, not a preference.
3. **Two concurrent pinned contexts. Negotiable.** Fallback: one slot, and a deliberative call costs the tactical slot one prefill. At 300 ms of prefill every 10 to 60 seconds that is 0.5% to 3% of duty cycle, which is acceptable. If slots evict each other, `17-model-contract.md` says so rather than claiming a property the stack does not have.
4. **Per-request visual cost. Negotiable in mechanism, not in effect.** If the runtime cannot do it, the agent rescales the image before sending.
5. **Reasoning depth as more than on and off. Negotiable**, degrading to a switch.
6. Multimodal input at all.
7. An HTTP interface stable enough to pin by commit.

## Considered options

1. **llama.cpp `llama-server`, Vulkan backend.**
2. **llama.cpp `llama-server`, ROCm backend.**
3. Ollama.
4. LM Studio.
5. vLLM.
6. ONNX Runtime with DirectML.
7. MLC and TVM on Vulkan.

## Validation

Written before the measurements, as the process requires.

**The measurement that decides it** is a table with one row per candidate and one column per capability above, each cell carrying a date and the invocation that produced it.
A cell filled from documentation rather than from a run is marked as such and does not count toward a pass.

**Pass** is every non-negotiable capability demonstrated on the reference hardware, with the negotiable ones recorded as present or absent rather than assumed.

**What would change our minds later**, stated as conditions rather than moods:

- **Tactical latency at the 95th percentile exceeds 500 ms** with a game resident, at the visual cost the tactical cadence uses. That is past the upper bound `15-performance-budgets.md` sets, and it reopens the runtime as well as the variant.
- **The two-slot fallback costs more than 5% of duty cycle** in the measurement E5 produces. Above that, one slot stops being an acceptable degradation.
- **A runtime the table rejected gains grammar-constrained decoding on Vulkan.** The table is dated; a capability that arrives later does not invalidate the decision, it schedules a revisit.
- **The ROCm path measures materially faster than Vulkan on this card.** Not a reason to switch on its own — the rest of the design assumes Vulkan — but a reason to record the gap and decide deliberately.

## Pros and cons of the options

### llama.cpp `llama-server`, Vulkan

- Good, because it is the only candidate that plausibly meets all seven, and the project's vocabulary already contains `llama.cpp`, `GBNF` and `mmproj` — the design was drafted around it.
- Good, because grammar-constrained decoding is native and the grammar is regenerated per request, which is exactly what `ADR-0013` needs.
- **Partly measured, 2026-09-12.** The Vulkan backend loads on the reference card, the vision tower works — 7 of 8 synthetic scenes exactly correct on all three fields — and the graphics processor is genuinely in use at 8.42 times processor-only generation throughput. See `experiments/02-vision-encoder/`.
- **Measured and not as assumed:** the per-image visual token budget is a process-level flag (`--image-min-tokens`, `--image-max-tokens`), not a per-request parameter. Capability 4 therefore lands on its fallback: the agent controls cost by rescaling. That is the outcome this record anticipated in driver 4 before the run.
- Bad, because it is a fast-moving project and the interface changes; the pin is by commit for that reason.

### llama.cpp `llama-server`, ROCm

- Good, because it is the same runtime and therefore the same capability set.
- Good, because a Windows ROCm build ships in the same releases, which an earlier version of the known-good matrix assumed did not exist.
- Unmeasured. Whether it is faster than Vulkan on this card is a number nobody here has.

### Ollama

- Good, because it is the easiest thing to install and operate.
- Bad, because it wraps llama.cpp and does not expose raw grammar or slot pinning, which is capability 1 and capability 3.

### LM Studio

- Good, because the interface is pleasant and the model management is done for you.
- Bad, for the same capability reasons as Ollama.
- Bad, because its licence is not appropriate as a dependency of an Apache-2.0 project.

### vLLM

- Good, because it is the strongest server in this space on hardware it supports.
- Bad, because there is no practical Vulkan path on Windows. Rejected on the hardware, not on the merits.

### ONNX Runtime with DirectML

- Good, because DirectML is the genuine first-party AMD path on Windows.
- Bad, because it offers neither constrained decoding nor slot semantics. This is the **reserve** if the Vulkan multimodal path turns out to be broken, and taking it means building our own host — months, not weeks. Recorded so that the cost is known before it is needed rather than discovered while needing it.

### MLC and TVM on Vulkan

- Good, because it is a real Vulkan inference path with its own compiler stack.
- Bad, because the multimodal and grammar stories are weaker, and the build is a second toolchain.
- Named here as a genuine second option rather than a straw one: if llama.cpp's Vulkan multimodal path had failed, this is what would have been tried before DirectML.

## More information

- `experiments/02-vision-encoder/` holds what has been measured so far, including the device control and why the first version of it did not work.
- `ADR-0007` chooses the variant and quantisation and is pre-registered alongside this one.
- `17-model-contract.md` is the page this record governs.
