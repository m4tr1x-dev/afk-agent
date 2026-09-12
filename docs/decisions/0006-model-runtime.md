---
title: ADR-0006 — The local model runtime
description: Pre-registered. Which inference runtime hosts the model, with the capability ladder written before any of them was measured.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, model, runtime]
requirements: [FR-MODEL-001, FR-MODEL-002, FR-MODEL-003]
decisions: [ADR-0013]
affects: [crates, contract]
spec: [docs/spec/17-model-contract.md]
evidence:
  - experiments/02-vision-encoder/findings.md
  - experiments/02-vision-encoder/results/2026-09-12-e4b-vulkan.txt
  - experiments/02-vision-encoder/results/2026-09-12-26b-a4b-vulkan.txt
  - experiments/03-model-latency/findings.md
  - experiments/03-model-latency/results/2026-09-12-tactical.txt
  - experiments/03-model-latency/results/2026-09-12-slots.txt
  - experiments/03-model-latency/results/2026-09-12-deliberative-cpu.txt
supersedes: null
superseded_by: null
generated: false
---

# ADR-0006 — The local model runtime

!!! note "This record was pre-registered"

    Everything except the Decision section was written **before** the
    measurements, and the Decision section was literally absent until they
    returned. The Validation section below is unedited since then.

    That ordering earned its keep. Driver 4 named a fallback for the per-request
    visual budget before anyone knew whether the runtime offered one — and it
    does not, so the fallback is what shipped. A record written afterwards would
    have presented rescaling as the plan all along.

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

## Decision

**`llama-server` from llama.cpp, Vulkan backend, pinned to build 10930, commit `56381e407`.**

Every capability in the ladder was measured on the reference hardware rather than read from documentation.

| # | Capability | Result |
| --- | --- | --- |
| 1 | Grammar-constrained decoding | **Yes.** Every measurement in both experiments constrains output with a GBNF grammar, and an answer outside it was never produced |
| 2 | Vulkan | **Yes.** 8.42x and 11.23x generation throughput against a processor-only baseline of the same model |
| 3 | Two concurrent pinned contexts | **Yes, and they do not evict each other.** Twenty alternating rounds; each slot reused its own prefix, 123 tokens of 128 and 312 of 317 |
| 4 | Per-request visual cost | **Not as a request parameter.** A process-level flag; the per-request dial is the resolution of the image sent — the fallback this record named in advance |
| 5 | Reasoning depth beyond on and off | **Available and unexercised.** `--reasoning-budget` takes a token count rather than a switch, so the capability exists; no measurement here used it |
| 6 | Multimodal input | **Yes.** 8 of 8 synthetic scenes read exactly on all three fields by 26B A4B, 7 of 8 by E4B |
| 7 | An interface stable enough to pin by commit | **Yes**, at `56381e407` |

Six of seven demonstrated, one available but unexercised, and the one that came back negative landed on the fallback written down before the run.

**Capability 4 is the one worth dwelling on**, because it is the only place the runtime disagreed with the specification.
`FR-MODEL-002` said "set the per-image visual token budget per call"; the runtime offers `--image-min-tokens` and `--image-max-tokens` as process arguments and nothing on the request.

The effect is available anyway, and more completely than a flag would give it.
The per-request cost tracks the resolution of the image sent — 83, 123, 258 and 443 image tokens at 256, 512, 768 and 1024 pixels — and rescaling before sending is something this project does itself, depending on nothing the runtime chose to expose.
The requirement was reworded from setting a budget to controlling a cost, which is satisfiable either way and does not silently become false when a runtime changes its flags.

**What this decision does not claim.**
The runtime is adequate, not fast enough.
A tactical tick measures 445 ms against a 150–400 ms target, and no runtime choice closes that gap: the cost is prefill and decoding rather than the server.
That belongs to `ADR-0007` and to the cadence, not here.

## Consequences

### Good

- The capability ladder was written before the measurements and one rung fell. That is the mechanism working rather than a cost: the fallback for capability 4 was chosen while the answer was still unknown, so it was not chosen because it turned out to be convenient.
- Grammar-constrained decoding is native and per-request, which is what `ADR-0013` needs and what every rejected candidate lacked.
- Two pinned slots are available, so the two cadences do not have to share one context and `17-model-contract.md` keeps the layout it describes.
- The processor fallback in the degradation ladder is real: a full deliberative call takes 41.5 seconds against a 60-second criterion.

### Bad

- llama.cpp moves quickly and its interface changes with it. The pin is by commit for that reason, and every upgrade is a pull request that re-runs both experiments rather than a version bump.
- The visual budget is a process argument, so changing it means restarting the model host. The agent's own rescaling covers the per-request case, but a session that wanted to change the *cap* would have to reload.
- Nothing here exercised reasoning depth. Capability 5 is recorded as available on the strength of the runtime's interface, not of a measurement, and that distinction is the reason this row says so.
- The decision is measured on one build. A runtime pinned by commit is reproducible, and it is also a commit that will be a year old before this project ships.

### Neutral

- A Windows ROCm build exists in the same releases. Vulkan stays the default because the design assumes it and the experiments ran it first; whether ROCm is faster on this card is unmeasured, and the known-good matrix now says so rather than asserting Vulkan is the only path.

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
