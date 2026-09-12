---
title: Model contract
description: "The interface between the agent and the local model: prompt layout, constrained output, and sampling."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, model]
requirements: [FR-MODEL-001, FR-MODEL-002, FR-MODEL-003, FR-MODEL-004, FR-MODEL-005, NFR-MODEL-001, INV-MODEL-001, FR-CTX-001, FR-CTX-002, FR-CTX-003]
decisions: [ADR-0013]
generated: false
---

# Model contract

## Purpose

This page specifies what the agent requires of a model, how it is asked, and how the answer is constrained.

It is deliberately written against *capabilities* rather than against one model, so that a different model can be substituted by meeting the contract.
The concrete choice is [a decision record](../decisions/index.md), not a specification.

## Required capabilities

A model satisfies this contract when it provides all of the following.

| Capability | Requirement | Why |
| --- | --- | --- |
| Image input | At least one image per request, at a resolution where interface text is legible | The agent's only sensor |
| Adjustable image cost | The number of tokens an image consumes is selectable per request | The main latency dial (`FR-MODEL-002`) |
| Context length | Enough for the layout below plus a multi-hour session's compacted history | Long-horizon goals |
| Constrained output | Output restricted to a supplied grammar | `FR-MODEL-003`; the reliability mechanism |
| Prefix reuse | A previously computed prompt prefix is reused when unchanged | Otherwise every tick re-reads the whole prompt |
| Concurrent contexts | Two independent conversations against one set of weights | The two cadences differ in every respect |
| Selectable reasoning depth | Extended reasoning can be turned off | The tactical cadence cannot afford it |

The selected model family provides all of these.
**Its measured latency is not yet known** — see open question 1.

## One model, two cadences

The [reasoning loop](05-reasoning-loop.md) runs a fast cadence and a slow one.
The intuitive arrangement is two models, a small one for speed and a large one for quality.

That is not what this specification does, for a measurable reason: within a model family, the small variants are substantially weaker at vision than the large ones, and the cadence that inspects the screen several times a second cannot be the one that sees worst.

Instead, **one set of weights serves both cadences**, differentiated by how it is asked:

| | Tactical | Deliberative |
| --- | --- | --- |
| Rate | 1–4 Hz | Every 10–60 s |
| Image cost | Small | Large |
| Extended reasoning | Off | On |
| Context | Short, pinned | Longer, pinned |
| Output | One action | A plan revision |

Adjustable image cost and selectable reasoning depth exist precisely to support this.
One set of weights in memory, two behaviours, no second model to load or to keep resident alongside the game.

Each cadence holds its own pinned context so that neither evicts the other.
They have entirely different prefixes, and re-reading a long invariant prefix on every deliberation would consume the deliberative budget by itself.

## Prompt layout

`FR-CTX-001`, `FR-CTX-002`, `FR-CTX-003`.

The eight blocks and their order are specified in [context and research](08-context-and-research.md), which owns the layout.
They are not restated here.
A layout whose entire purpose is byte-level stability, written down twice, is how two implementations end up agreeing on neither.

Four rules, each load-bearing.

**The image goes last.**
Everything to the right of a change must be recomputed.
An image is the most expensive single element in the prompt, so placing it at the end means a changed image invalidates the least.

**Old images are never resent.**
`FR-CTX-003`.
Historical observations appear as their textual element digest, never as retained pictures.
This is what makes a multi-turn visual loop affordable at all: an agent that carried ten past frames would spend its entire context on them and still see less than the digests convey.

**Blocks 1 and 2 are byte-identical between calls.**
`FR-CTX-002`.
The system prompt with its tool schemas and safety framing, and the goal.
Both are fixed when the session starts and neither changes until it ends.
No timestamps, no tick counters, no reordered object keys, no elapsed-time strings.
A single changed byte discards the whole cached prefix, and since block 1 is the largest invariant part of the prompt, that is the difference between a cheap tick and an expensive one.

This is easy to violate by accident.
Putting the current time into the system prompt is the classic version, and it costs the prefix on every single call.

**Measured 2026-09-12, and it is not an optimisation.**
At a realistic prompt size of about 4800 tokens, with a game resident and a fresh frame every request:

| 26B A4B Q4_K_M | Prefix cache off | Prefix cache on |
| --- | --- | --- |
| Tactical tick, 256-pixel image | 3282 ms | **596 ms** |

The rule is worth **2686 ms per tick**.
Without it a tactical call costs three and a half seconds, which is not a slower agent but a different architecture — roughly the cadence this page reserves for deliberation.

A smaller prompt hides this entirely. Measured against a forty-token prompt the cache appears to buy nothing, 427 ms against 430 ms, because there is no prefix to hit. That is a property of the measurement, not of the cache.
See `experiments/03-model-latency/`.

**Blocks 3 and 4 are append-only.**
The note block and the research excerpts grow during a session; nothing already written in either is edited or reordered.
Append-only is a weaker property than byte-identical and it buys most of the same thing: every byte before the append is unchanged, so the cached prefix survives up to that point and only the appended text is new work.
Editing a note in place, or rewriting the block in a different order, discards the prefix from that byte onwards — which is why `FR-CTX-005` gives the agent a way to add a note and not a way to revise one.

Compaction is the deliberate exception.
It rewrites blocks 3 and 4 and therefore costs the prefix from block 3 onwards, once, which is the price of not overflowing the window.

## Constrained output

`FR-MODEL-003`.

The grammar is **regenerated every tactical tick** and describes exactly what is valid right now:

- The tool names currently available.
- Their parameter types.
- The enumerated marks present in this observation.
- The enumerated grid cells, if the grid is in use.

So a reference to a mark that is not on screen is not merely unlikely — it cannot be produced.
The same applies to an unknown tool name or a malformed parameter.

This converts a large class of runtime failure into a class of output that cannot be expressed, and it is the reason the contract requires grammar support rather than treating it as an optimisation.

When output nevertheless fails validation — a semantically invalid combination that the grammar cannot express — the agent issues **one** structured repair request naming the problem, and on a second failure records a no-op and advances the no-progress counters.
It does not retry blindly, because a model that produced invalid output once under the same conditions will produce it again.

## Sampling

| Cadence | Temperature | Notes |
| --- | --- | --- |
| Tactical | Low | Consistency matters more than variety; the grammar already bounds the space |
| Deliberative | Moderate | Planning benefits from considering alternatives |
| Repair request | Lowest available | The task is to produce one specific correct form |

Every request records its sampling parameters and seed in the session recording, because replay determinism depends on them.

## Memory management

`FR-MODEL-004`.

The model and the game share one graphics device, and the game was there first.

Available memory is polled, and when the game's headroom shrinks the agent degrades in a fixed order:

1. Reduce the context length.
2. Reduce the image cost for the tactical cadence.
3. Move the deliberative cadence off the graphics device entirely.
4. Pause and tell the user.

Step 3 is viable because of the shape of the chosen model: only a fraction of its parameters are active for any token, so processor-only execution reads a few gigabytes per token rather than the whole model.
That yields single-digit to low-double-digit tokens per second on the reference processor — **an estimate, not a measurement** — which is acceptable for a call that fires twice a minute and whose result is applied asynchronously.

The model was chosen partly for that property: it degrades onto the processor gracefully, and the alternative to graceful degradation is a stuttering game.

## Weight handling

`FR-MODEL-005`, `INV-MODEL-001`.

Weights are downloaded by the user, verified by digest before loading, and never modified.

The runtime version used to load them is pinned and recorded, because model file formats change and a mismatch between the version that produced a file and the version that reads it can fail **silently** — the file loads and the results are wrong.

## Substituting a model

A different model is supported when it meets the capability table and passes the [evaluation harness](20-evaluation-harness.md) at no worse than the incumbent on grounding accuracy and action validity.

The agent must not contain anything that only works with one model.
Where a model-specific detail is unavoidable — a chat template, a token budget encoding — it lives in one adapter and nowhere else.

## Open questions

1. **Blocking.** The **deliberative** latency figure is still missing, and the escalation ladder's timing follows from it. The tactical figure was measured on 2026-09-12 with a game resident, a product-sized prefix and a fresh frame per request: 445 ms for E4B and 596 ms for 26B A4B, against a 150–400 ms target. That corrected this page, [performance budgets](15-performance-budgets.md) and the cadence in [the reasoning loop](05-reasoning-loop.md). See `experiments/03-model-latency/`.
2. **Non-blocking.** Whether any *further* variant needs the same check before it is shipped. Answered on 2026-09-12 for both variants that matter: E4B reads 7 of 8 synthetic scenes exactly on all three fields and 26B A4B reads 8 of 8, with the graphics processor confirmed in use at 8.42 and 11.23 times processor-only throughput. The tactical cadence has a confirmed implementation. See `experiments/02-vision-encoder/`.
3. **Blocking.** Whether prefix reuse behaves as assumed with two pinned contexts at different image costs. If the runtime shares one cache between them, the two cadences evict each other and the layout above buys nothing.
4. **Non-blocking.** Whether a per-request budget parameter is worth asking the runtime for. Measured 2026-09-12: the budget is a process-level flag (`--image-min-tokens`, `--image-max-tokens`), and the per-request cost is set by the resolution of the image sent — 83, 123, 258 and 443 image tokens at 256, 512, 768 and 1024 pixels. The two-stage refinement therefore has the control it needs, by rescaling. See `experiments/02-vision-encoder/`.
5. **Blocking.** Whether extended reasoning can be capped rather than merely switched off. Uncapped reasoning on the deliberative path is an unbounded pause.
6. **Blocking.** What the agent does when the model host reports a context overflow mid-session. Compaction should prevent it; "should" is not a mechanism.

## Related decisions

`ADR-0013` records constrained decoding, which this page turns into a hard requirement on the model host, and is accepted.
`ADR-0006`, the inference host and compute backend, and `ADR-0007`, the model selection, are not written and are blocked on the measurements above.
