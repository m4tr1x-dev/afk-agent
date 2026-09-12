# Question 3 — what a tactical tick costs, with a game resident

**Status: answered, and two specification numbers are wrong.**

The tactical budget of 150–400 ms is not met by any variant measured. The
closest is 445 ms. And the prefix cache turns out not to be an optimisation at
all: at the prompt size the product actually uses, it is the difference between
a 596 ms tick and a 3282 ms one.

Run on 2026-09-12 on the reference hardware. llama.cpp build 10930, commit
`56381e407`, Vulkan, 8192 context, one slot. AssaultCube windowed at 1280x720,
resident throughout. The image is a real frame captured from that game through
Windows Graphics Capture, not a synthetic one.

## The answer

Realistic conditions: a product-sized stable prefix of about 4800 tokens, the
prefix cache on, and a **fresh image every request** — because the scene moved,
which is what a tick is.

| Variant | 256 px | 512 px | 1024 px |
| --- | --- | --- | --- |
| **E4B Q4_0** | **445 ms** | 452 ms | 517 ms |
| **26B A4B Q4_K_M** | **596 ms** | 639 ms | 1014 ms |

Against a budget of 150–400 ms.

| | |
| --- | --- |
| Best measured tick | **445 ms**, E4B at 256 px |
| Over the ceiling by | 11% |
| Maximum tactical cadence | **2.2 Hz** with E4B, **1.7 Hz** with 26B A4B |
| `05-reasoning-loop.md` says | 1–4 Hz |

**The upper half of the stated cadence range is unreachable.** The page to
change is the cadence, not the measurement — which is what `15-performance
budgets.md` §When a budget is exceeded already requires, and what the
prediction written before this run expected.

### The prediction, and how it did

Written into the probe before it ran, as `adr-process.md` asks:

> The honest expectation is 250–600 ms and that the 4 Hz upper bound in
> `05-reasoning-loop.md` is out of reach.

Directionally right and numerically optimistic. E4B landed at 445–517 ms, inside
the predicted band; 26B A4B landed at 596–1014 ms, above it.

## FR-CTX-002 is not an optimisation

This is the finding worth more than the latency number.

`17-model-contract.md` requires blocks 1 and 2 to be byte-identical between
calls so the prefix cache hits. With the probe's own forty-token prompt, turning
the cache on changed nothing — 427 ms against 430 ms — and it would have been
easy to write down that the cache buys little.

That would have been an artefact of the probe. The product's prompt is blocks 1
to 5: system prompt, tool schemas, safety framing, goal, notes, plan, history.
Padding the prefix to a realistic 4800 tokens changes the picture completely:

| 26B A4B, product-sized prefix | 256 px | 512 px | 1024 px |
| --- | --- | --- | --- |
| Prefix cache **off** | 3282 ms | 3298 ms | 3686 ms |
| Prefix cache **on**, fresh image | **596 ms** | 639 ms | 1014 ms |
| Saved per tick | **2686 ms** | 2659 ms | 2672 ms |

**Without byte-identical blocks 1 and 2, a tactical tick costs three and a half
seconds.** That is not a slower agent; it is a different architecture, at
roughly the cadence the specification reserves for deliberation.

So the requirement is load-bearing in the strict sense, and the page should say
so with this number rather than describing the rule and leaving the reader to
assume it is a tuning matter.

It also explains the shape of the layout rule. The cache is a *prefix* cache: it
hits up to the first byte that differs. The image changes every tick, so
everything after the image is uncacheable — which is precisely why
`17-model-contract.md` puts the image **last** and why an appended note is
cheap while an edited one is not.

## Three conditions, because two would have misled

Neither obvious condition is the product's tick, and they mislead in opposite
directions.

| Condition | What it measures | Why it is not the tick |
| --- | --- | --- |
| Cache off | Nothing reused | The product deliberately keeps its prefix byte-identical |
| Cache on, **same image** | Everything reused | No real tick sends the same frame twice |
| Cache on, **fresh image** | Prefix hits, image does not | **This is the tick** |

The middle row is the trap. E4B at 1024 px reads 229 ms there — comfortably
inside budget, and completely unreachable, because it requires the scene to have
stopped moving.

## Where the time goes

The text-only floor, same conditions, no image at all:

| Variant | Floor | With a 256 px image | The image costs |
| --- | --- | --- | --- |
| E4B | 333 ms | 445 ms | 112 ms |
| 26B A4B | 412 ms | 596 ms | 184 ms |

So two thirds of a tick is not the image. At 24 generated tokens and a measured
148 tokens per second, decoding accounts for about 160 ms of the E4B floor; the
rest is prefill of the uncached tail, request handling and the round trip.

That matters for where to look next. Reducing the visual budget is the dial the
specification reaches for first, and it is worth about 70 ms between 256 px and
1024 px on E4B. Shortening the answer would be worth more.

## What this does not settle

- **The deliberative figure**, 3–15 s, is untouched. A deliberative call has a
  different shape — extended reasoning, a larger visual budget — and question 6
  is its own experiment.
- **Two pinned slots.** Everything here uses one. Whether two contexts at
  different visual budgets evict each other is question 5, and this measurement
  says nothing about it.
- **The frame rate the game keeps.** That needs event-tracing instrumentation
  this project does not yet have, and it is the criterion in `vision.md` that
  decides whether any of this is usable.
- **A real prompt.** The 4800-token prefix is filler of the right size, not the
  product's actual blocks. The cache does not care what the bytes are, only that
  they repeat, so the saving should hold — but the number is measured against
  filler and the page should say so.
- **The 95th percentile at 256 px** is inflated in two runs by a single sample
  near 1400 ms, an artefact of the warm-up request using a different image from
  the measured ones. The medians and minima are clean; the spike is recorded
  rather than trimmed.

## A correction to the vision-encoder findings

`experiments/02-vision-encoder/findings.md` says the runtime's device
enumeration "returns a static figure — 23749 MiB free on this card whether a
model is resident or not". That was too strong, and this experiment's setup
disproved it:

| State | Free, as the runtime reports it |
| --- | --- |
| Nothing running | 23749 MiB |
| A game resident (about 2.3 GiB) | 23749 MiB |
| A 16 GiB model resident | 22550 MiB |

So the figure **does** move — by 1199 MiB for a 16 GiB model, and not at all for
another process's allocation. It is not static; it is unrelated to what is
actually resident.

The conclusion drawn from it stands and strengthens: a control of the form
"weights on disk should appear as consumed graphics memory" cannot work here,
and throughput against a zero-offload baseline is the control that does.

**And it has a consequence for the product.** `FR-MODEL-004` requires monitoring
available graphics memory and degrading before the game is starved. This
runtime's device report cannot serve that: it under-reports the agent's own
residency by a factor of thirteen and does not see the game at all. The poll has
to use a platform query against the adapter, not the inference runtime's view.

## Method

```bash
# A real frame from a real game, through the route the product will ship.
capture-probe --window "AssaultCube" --save E:/afk-agent/frames --frames 3

model-latency-probe \
  --frame E:/afk-agent/frames/frame-0000.png \
  --label "26B A4B Q4_K_M, game resident, product-sized prefix" \
  --requests 30 --prefix 300
```

Full transcripts in `results/`.
