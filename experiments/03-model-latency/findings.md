# Questions 3, 5 and 6 — what the model costs, with a game resident

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

## Question 5 — the two slots keep their caches

`17-model-contract.md` assumes two pinned contexts at different visual budgets
do not evict each other. `ADR-0006` listed that as negotiable and wrote the
fallback down before the measurement: one slot, and a deliberative call costs
the tactical slot one prefill.

Twenty alternating rounds, E4B, slot A at 256 pixels and slot B at 1024:

```text
  slot                 reused    prefilled   prefill ms
  A tactical              123            5         27.3
  B deliberative          312            5         37.8

  A mutated                 7          121        137.7   (one byte changed)
```

Each slot reuses its own prefix while the other alternates against it. Five
tokens prefilled out of 128 and 317.

**The negative control is the part that makes this mean anything.** A runtime
can report a cache hit and prefill anyway, so the counter alone proves nothing —
which is why prefill time is recorded beside it, and why one byte of slot A's
prefix is changed at the end.

The reuse collapses from 123 tokens to 7, the prefill rises from 5 tokens to
121, and the time goes from 27.3 ms to 137.7. That is a five-fold jump, in the
direction and of the size a real cache miss produces. Without it, every number
above would be consistent with a server reporting whatever it liked.

**So `ADR-0006`'s one-slot fallback is not needed**, and `17-model-contract.md`
is claiming a property the stack actually has.

## Question 6 — the processor fallback is usable, and marginal

Step 3 of the degradation ladder moves the deliberative cadence off the graphics
processor when the game needs the memory. `ADR-0006` fixed the criterion before
the run: **a full deliberative call under 60 seconds** is a degradation; above
that it is a hang, and both `15-performance-budgets.md` and
`17-model-contract.md` would need rewriting.

26B A4B with every layer on the processor, a 1024-pixel frame, a 4800-token
prefix, and an answer free-running to 512 tokens:

| Condition | Deliberative call |
| --- | --- |
| Prefix cache **off** | **57.5 s** |
| Prefix cache **on**, fresh image | **41.5 s** |

**It passes, with 31% headroom, and only because of the prefix cache.**

Two things follow that the page should say rather than leave to be discovered.

At 41.5 s a deliberative call consumes most of the 10–60 second cadence the
specification gives it. The fallback sustains the slow end of that range and not
the fast end, so degrading to the processor also means degrading the cadence.

And `FR-CTX-002` is load-bearing here too. Without the byte-identical prefix the
same call is 57.5 s — inside the criterion by 4%, which is not a margin anybody
should design against.

The answer had to run free to measure this at all. Under the tactical grammar
the model stops after one short tool call, the 512-token limit never binds, and
the call reads a comfortable 3.7 s — a number that says nothing about
deliberation.

## What this does not settle

- **The deliberative figure on the graphics processor**, 3–15 s, is untouched.
  Question 6 measured it on the processor alone, which is the fallback rather
  than the normal path.
- **Extended reasoning is off** in every measurement here. Question 6's answer
  runs free to 512 tokens, which is a long answer but not a reasoning trace.
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
