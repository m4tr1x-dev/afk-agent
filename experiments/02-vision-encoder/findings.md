# Question 2 — does the vision encoder load on the Vulkan backend?

**Status: answered. It loads, it sees, and the question behind the question is
answered too.**

Run on 2026-09-12 against **both** variants on llama.cpp build 10930, commit
`56381e407`, Vulkan backend, on the reference hardware: Gemma 4 E4B at four-bit
quantisation with a BF16 projector, and **26B A4B at Q4_K_M with an F16
projector**, which is the variant the matrix question names.

| Question | E4B Q4_0 | 26B A4B Q4_K_M |
| --- | --- | --- |
| Does the encoder load? | Yes | Yes |
| Does the model actually **see** the image? | **7 of 8** exact on all three fields | **8 of 8** |
| Was the graphics processor used? | Yes, **8.42x** against processor-only | Yes, **11.23x** |
| Is the visual token budget a per-request knob? | No. A server flag; per request the caller sets the cost by input resolution | Identical, to the token |

The last row is the one the runtime decision turns on, and the answer changes a
requirement. See **The budget is a flag, not a parameter** below.

## Why "it loaded" was not the question

A vision tower wired to the wrong projector loads without complaint and answers
fluently about the general image. That is the quiet failure
`docs/known-good-matrix.md` warns about for a version mismatch, and no assertion
of the form "the server started" detects it.

So the assertion is discriminative. Each of eight synthetic scenes carries three
independent facts, drawn at run time from a printed seed so a scene cannot have
been memorised:

| Fact | Values | Chance |
| --- | --- | --- |
| A three-digit number | 100 to 999 | 1 in 900 |
| A named background colour | six | 1 in 6 |
| A shape | four | 1 in 4 |

All three right by luck is one in 21,600. The bar, fixed before the run, was
seven of eight exactly correct on all three fields.

### Result: 7 of 8

```text
0  expected 396 on green with a circle    got 396 / green  / circle    ok
1  expected 142 on purple with a square   got 142 / purple / square    ok
2  expected 151 on grey with a cross      got 151 / grey   / cross     ok
3  expected 872 on green with a triangle  got 872 / green  / triangle  ok
4  expected 744 on purple with a cross    got 744 / purple / cross     ok
5  expected 211 on green with a square    got 211 / green  / square    ok
6  expected 203 on grey with a circle     got 203 / grey   / circle    ok
7  expected 702 on red with a cross       got 102 / red    / cross     MISS
```

Every colour and every shape was correct in all eight. The single miss is a
digit: 7 read as 1.

**That is a property of the fixture as much as of the model, and it is worth
saying rather than explaining away.** The digits are drawn in a seven-segment
face, where 7 and 1 differ by exactly one bar. Two of the ten digits being
one segment apart is a discrimination task the fixture chose, not one the
product cares about.

The fixture was **not** changed afterwards. Redrawing the digits having seen
which pair was confused is fitting the test to the result, and the number would
stop meaning anything.

Reproduced exactly across two runs at seed 424242, including the same miss.

## The device control, and the one that did not work

The first version of this control read free graphics memory before and after the
load, and compared the difference against the weights on disk.

**It reported a fallback that had not happened.**

This runtime's device enumeration returns a static figure — 23749 MiB free on
this card, before the load, during it, and after the server stopped. The reading
never moves, so the control could only ever say "it fell back".

That is the project's own recorded lesson arriving from the other direction.
`experiments/01-reachability` learned that a control which catches a false
positive does not catch a false negative. This is the mirror image: a control
that fires when nothing is wrong is worth no more than one that stays silent
when something is, and it costs more, because it gets believed once.

**The control that works supplies its own baseline.** The same model,
benchmarked with every layer on the graphics processor and with none of them:

| Layers offloaded | Prefill, tokens per second | Generation, tokens per second |
| --- | --- | --- |
| all | 3207.26 | 148.37 |
| none | 366.53 | 18.30 |
| **ratio** | **8.7x** | **8.1x** |

A silent fallback is not a few percent slower. The assertion is a ratio of three
or better, and the measurement is 8.42x for the run recorded here.

This is also a usable number in its own right: **18.3 tokens per second is the
processor-only generation rate for a 4.5B-effective model on this machine**,
which is the first real data point for step 3 of the degradation ladder in
`15-performance-budgets.md`.

## The budget is a flag, not a parameter

`FR-MODEL-002` says the system must **set the per-image visual token budget per
call**. The model card documents budgets of 70, 140, 280, 560 and 1120.
`17-model-contract.md` open question 4 asks whether image cost is selectable per
request or only per session, and notes that the two-stage refinement in
`18-grounding-and-verification.md` needs per-request.

Three measurements answer it.

**1. The cost tracks the resolution of the image we send.** A text-only request
under the same grammar costs 50 prompt tokens; subtracting it gives the image
cost:

| Input | Image tokens |
| --- | --- |
| 256 px | 83 |
| 512 px | 123 |
| 768 px | 258 |
| 1024 px | 443 |

Not linear in area, and not equal to the card's named budgets — consistent with
the runtime tiling a dynamic-resolution image rather than exposing Gemma's
fixed steps.

**2. There is a cap, and it is a server flag.** `--image-min-tokens` and
`--image-max-tokens` are process-level arguments. Started with
`--image-max-tokens 128`, the same sweep saturates:

| Input | Image tokens, capped at 128 |
| --- | --- |
| 256 px | 83 |
| 512 px | 123 |
| 768 px | 123 |
| 1024 px | 123 |

The cap binds from 512 px upwards. Below it, resolution still decides.

**3. Nothing on the request selects a budget.** The chat completion endpoint
takes no image-token parameter, and the flags above cannot be changed without
restarting the process.

### What this means for the specification

The mechanism `FR-MODEL-002` names does not exist. The **effect** does, and it
is under the caller's control more completely than a runtime flag would be:
rescaling before sending is something this project does itself, on every
request, with no dependency on what the runtime chose to expose.

So the requirement should say what the system needs rather than how a runtime
might provide it:

> The system MUST **control the per-image visual token cost** per call.

That is satisfiable today by rescaling, satisfiable by a per-request parameter
if one ever appears, and it does not silently become false when a runtime
changes its flags. `ADR-0006` already anticipated this outcome and named it
"negotiable in mechanism, not in effect" before the experiment ran.

It also answers `17-model-contract.md` open question 4 in full: **per session by
flag, per request by resolution.**

## The named variant, and the digit E4B missed

The 26B A4B run used the same eight scenes at the same seed, so the two results
are directly comparable rather than merely similar.

```text
0  396 on green with a circle    ok    1  142 on purple with a square   ok
2  151 on grey with a cross      ok    3  872 on green with a triangle  ok
4  744 on purple with a cross    ok    5  211 on green with a square    ok
6  203 on grey with a circle     ok    7  702 on red with a cross       ok
```

**8 of 8**, including the scene E4B read as 102 — the seven-segment 7 that
differs from a 1 by one bar. The larger model resolves it; the smaller one does
not. That is a difference on a task the fixture happens to make hard, which is
worth one sentence and not more: it is not the grounding benchmark, and nothing
here ranks the variants for `ADR-0007`.

**The visual token counts are identical to E4B, to the token**: 83, 123, 258 and
443 at 256, 512, 768 and 1024 pixels, against the same 50-token text baseline.
The two families share the projector's tiling behaviour, which means the budget
finding is a property of the runtime rather than of a variant, and does not have
to be re-established per model.

### Throughput, both variants

| | E4B Q4_0, 4.26 GiB | 26B A4B Q4_K_M, 15.77 GiB |
| --- | --- | --- |
| Prefill, all layers | 3207 tokens/s | 1652 tokens/s |
| Prefill, no layers | 367 | 79.7 |
| Generation, all layers | 148.4 | 141.8 |
| **Generation, no layers** | **18.3** | **12.6** |
| Request latency, idle machine, 123 image tokens | ~0.37 s | ~0.65 s |

**The bottom row is a number the specification has been waiting for.**
`15-performance-budgets.md` describes step 3 of the degradation ladder — moving
the deliberative cadence to the processor — as viable at "an estimated
single-digit to low-double-digit tokens per second", marked **Unvalidated**.

Measured: **12.6 tokens per second** for 26B A4B on this processor. The estimate
was right, and the row can lose its marking once the deliberative call's shape
is measured against it. That is question 6, and this is not that measurement —
it is the throughput figure that question 6 needs, taken on an idle machine with
no game resident.

Generation throughput barely separates the two variants (141.8 against 148.4)
because only a fraction of the 26B's parameters are active per token. Prefill
separates them by a factor of two, and prefill is what a tactical tick pays.

## What was not tested

- **The named budgets 70, 140, 280, 560 and 1120.** The runtime does not offer
  them as steps, so the closest thing measurable is the cap, which was measured.
- **Latency with a game running**, which is question 3 and a different
  experiment. The sub-second figures here are on an idle machine and are not
  the figures that matter. Nothing in this page should be read as a tactical
  latency measurement.
- **The 12B variant**, and every quantisation other than the two run. `ADR-0007`
  needs all six rows and a grounding benchmark; this is one column of one of
  them.
- **A quantised projector.** BF16 throughout, deliberately: a quantised
  projector is a known way to get a model that loads, answers fluently and
  localises badly, and the point of this probe is to avoid exactly that.

## Method

```bash
bash experiments/02-vision-encoder/run.sh \
  --model     E:/afk-agent/models/gemma-4-E4B/gemma-4-E4B-it-Q4_0.gguf \
  --projector E:/afk-agent/models/gemma-4-E4B/mmproj-gemma-4-E4B-it-BF16.gguf \
  --seed 424242 \
  --out experiments/02-vision-encoder/results/2026-09-12-e4b-vulkan.txt
```

Weights from `ggml-org/gemma-4-E4B-it-GGUF` at revision
`b8093469224f83f5c38f691eb906c380e9e63114` — a commit, never a branch, because
a branch moves and a digest recorded against it quietly stops matching.

Full transcripts in `results/`.
