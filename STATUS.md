# afk-agent — build status

Last written 2026-09-12. Rewritten at milestone boundaries and when a blocker
opens or closes; otherwise at most every four hours.

**To stop the run: `touch .agent/HALT`**

## Where things are

Milestone **M1 — experiments**. Phase 0 and M0.5 are complete.

| Exit criterion | State |
| --- | --- |
| Reachability across engines | **Done, with a caveat.** Four engines tested, four reached. Four of four is stronger than the four-of-five asked for, but the denominator is not the one the protocol names, so it is recorded rather than closed |
| Camera calibration without a model | **Done.** 0.0909 degrees per mouse unit on AssaultCube, cross-checked two independent ways |
| Model loads on Vulkan with its vision encoder | **Done for the encoder.** Latency with a game running is still to measure |
| Grounding benchmark against a labelled corpus | **Blocked on you.** See below |

## Needs you

**One package, about 100 minutes, and it is the only thing I cannot do myself.**

The grounding benchmark needs labelled frames, and a benchmark labelled by the
system under test measures agreement rather than accuracy. The best available
ground truth is a real person's clicks: when you click at a point and the screen
changes within 200 ms, that point is by definition inside a real interactive
element. No model, no detector, no circular reasoning.

What it costs you:

| Step | Time |
| --- | --- |
| Play each of five titles for 15 to 20 minutes with the recorder running | ~90 minutes |
| Type 50 to 80 short descriptions for targets in the world that have no text | ~10 minutes |

The tedious part the specification called unavoidable — drawing boxes by hand —
disappears entirely under this method.

**Nothing is waiting on you today.** The recorder does not exist yet; it is built
as part of the capture experiment. This is flagged now so the request does not
arrive as a surprise later, and so you can say no early if the answer is no.

**If you never provide it**, the honest outcome is that `ADR-0014` stays
`proposed` with a Validation section saying what is missing. It will not be a
made-up number in a smart format.

## What has landed

Ten pull requests merged today. The ones worth knowing about:

- **The contract generator.** One TOML schema produces Rust types, C# types, the
  published reference page and the model grammar. Nothing off the shelf produces
  the last two, which is why we own it.
- **The toolchain is pinned** in five files, `just` is the task runner, and the
  first workflow that builds source rather than documentation now exists.
- **Ahead-of-time compilation of the interface framework works**, contradicting
  what the plan expected. The flag stays off for reasons now written down, but
  the decision rests on a measurement rather than on folklore.
- **All 102 open questions carry a disposition** — 38 blocking, 56 non-blocking,
  8 permanent limitations — and CI refuses an accepted page carrying the wrong
  kind.
- **The vision encoder sees.** 26B A4B reads 8 of 8 synthetic scenes exactly; the
  graphics processor is confirmed in use at 11.23 times processor-only.
- **The roster refuses.** A probe pointed at a title whose terms prohibit
  automation stops before synthesising anything, rather than being trusted to
  remember.

## Numbers measured today

| What | Value |
| --- | --- |
| Degrees per mouse unit, AssaultCube | 0.0909 |
| Frame correlation stops being usable | between 40 and 80 mouse units |
| Generation, 26B A4B Q4_K_M, graphics processor | 141.8 tokens/s |
| Generation, same, processor only | **12.6 tokens/s** — the degradation ladder's step 3, previously an estimate |
| Image tokens at 256/512/768/1024 px | 83 / 123 / 258 / 443 |
| Native binary, interface framework, ahead-of-time | 4.19 MiB, runs |

## Open blockers

None that stop work. The corpus above is the only human dependency, and it is
not yet on the critical path.

## Next

1. Capture and overlay: three capture backends, then the assertion that the
   overlay never appears in a captured frame.
2. The corpus recorder, which is what your 100 minutes would feed.
3. Model latency with a game actually running — the figure the specification
   says is the only one that matters.
