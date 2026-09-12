# Question 1 — does synthesised relative mouse movement reach a real game?

**Status: in progress.** The detection method is built and tested. Capture and
input synthesis are not yet wired, so the question is not answered.

## Why this probe exists

The known-good matrix puts it plainly: this blocks everything, and if it fails
the project as specified does not exist. It is far better to learn that from a
small probe than from three months of building on the assumption.

## The method the matrix did not specify

The matrix says the experiment "confirms the camera turned". That is not a
method, and confirming it by eye would make the answer to the project's gating
question a matter of opinion.

A small yaw rotation translates the projected image almost uniformly sideways.
So each frame is reduced to a one-dimensional profile of column intensities
over the **central band** — 60% of height, 80% of width — and the two profiles
are cross-correlated. The lag at the correlation peak is how far the world
moved.

The band matters. Interface elements are pinned to the edges of the screen and
do not move when the camera does; including them anchors the correlation at
zero lag and hides exactly the signal being measured.

## Four conjunct tests, because a lag alone proves nothing

An explosion moves pixels too. A result counts only when all four hold:

1. **Sign.** Positive and negative deltas produce opposite lags, consistently.
   A scene changing on its own has no reason to correlate with the sign of the
   input.
2. **Monotonicity and linearity.** Sweeping the total displacement produces
   lags that increase monotonically, and a line fitted through the origin
   reaches an acceptable fit. Its slope is pixels per mouse unit.
3. **Return to origin.** Emitting a delta train and then its inverse must bring
   the view back: the first and third frames correlate at lag zero. This single
   test proves both that input arrived and that the mapping is stable.
4. **Wrap.** Keep emitting until the view comes all the way round. Total units
   for one revolution gives degrees per mouse unit — the calibration figure
   `FR-ACT-004` needs, derived without knowing the field of view or the
   player's sensitivity setting, and failing informatively on a game with a yaw
   limit.

## Three null controls

Without these, a positive result is not evidence of anything.

- **No input.** Same timing, nothing synthesised. Lag must be near zero and the
  linear fit must fail. If the null control produces a signal, the detector is
  measuring animation and every positive result is worthless.
- **Wrong window.** Emit while a different window is in the foreground. No lag
  in the game's frames.
- **Absolute movement.** The same total displacement as one cursor
  reposition. `FR-ACT-004` predicts no turn. If absolute movement works as well
  as relative, that requirement's rationale is weaker than the specification
  claims and the page needs amending — a result worth catching.

## Four outcomes, not pass or fail

| Outcome | Means |
| --- | --- |
| Reached and linear | The specification holds |
| Reached but clamped | The lag saturates above some magnitude. That magnitude is the number `06-action-and-input.md` open question 4 asks for |
| Reached but non-linear | Acceleration or smoothing. Calibration needs a table rather than a ratio, and `ADR-0009` must say so |
| Not reached | Split further: input rejected, or capture broken. Distinguish by checking whether consecutive frames of a visibly moving game are identical |

## What is built

`src/correlate.rs`, with seven tests. Three of them exist because of failures
found while writing it:

- **A repeating pattern has no single answer.** A tiled floor or a railing
  correlates equally well at every multiple of its period. The probe emits
  small per-tick deltas, so ties resolve towards the smallest displacement;
  picking the largest would turn an ordinary wall into a reported ninety-degree
  turn.
- **The sign convention was backwards in my first draft.** Positive lag means
  the scene moved left, which is what turning the camera right produces. Test 1
  above depends entirely on getting this right.
- **A black frame correlates perfectly with another black frame** at every lag.
  Reporting "the camera did not turn" on that evidence would be wrong in the
  most misleading direction, so a featureless profile is reported as a capture
  failure instead — which is `FR-PERC-009`'s distinction, arriving early.

## What is not built

Window capture and input synthesis.

Input synthesis raises a question this probe cannot settle on its own.
`FR-ACT-008` requires exactly one call site, and `tools/check_call_sites.py`
enforces it against `experiments/` as well as `crates/`. A probe with its own
`SendInput` would validate something the product does not ship, and would need
the rule relaxed to compile.

The resolution is to build the minimum of `crates/afk-input` first and have the
probe depend on it, so that what E1 measures is what ships. That is `M3` work
pulled forward, and it is the right order: the experiment's whole purpose is to
learn whether the public input API reaches a game, and answering that about a
different piece of code would answer a different question.

## Results

None yet. `results/` is empty.
