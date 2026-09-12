# Breathedge 1.1.0.3, 2026-09-12 — reached

**Synthesised relative mouse movement through the public input API turns the
camera in a real game.**

That is the answer to known-good-matrix question 1 for one title, and it retires
the risk the roadmap describes as "if this fails there is no project".

Question 1 is **not fully resolved**: its criterion is four of five games across
different engines, and this is one. What is gone is the possibility that the
answer was no everywhere.

## Setup

| | |
| --- | --- |
| Game | Breathedge 1.1.0.3, Steam, single player |
| Engine | Unreal Engine 4 |
| Window mode | Windowed 1280×720, set in `GameUserSettings.ini` (original backed up alongside it) |
| State | Live gameplay, free look, first objective active |
| Input | `afk-input`, relative movement only, through the public synthesis API |

The game shipped configured for exclusive fullscreen, which
`00-scope-and-goals.md` lists as an assumption the system does not work around.
Changing it to windowed is a user responsibility the page already names.

## The decisive test

Automated verdicts rest on a correlation method that can be wrong, so the test
that settles this is one a person checks directly: capture the window, emit
600 mouse units horizontally, capture again, look.

The world rotated. The interior that filled the view before is gone; a
different section of the same cabin is centred. The head-up display did not
move, which is exactly right — it is anchored to the screen, not to the world.

Evidence: `E:\afk-agent\evidence\q1\turn-before.png` and `turn-after.png`. They
are frames of a commercial game and are deliberately kept outside this public
repository.

## Measured

Correlation runs against the `ScreenCrop` route, which reads the display and so
sees what a person sees.

| Requested, mouse units | Scene displacement, columns | Confidence |
| --- | --- | --- |
| 10 | 13 | 0.954 |
| 20 | 16 | 0.910 |
| 40 | 15 | 0.761 |
| 80 | 0 | 0.642 |
| 100 (earlier run) | 198 | 0.938 |

Two things are visible in that table and neither is the calibration ratio.

**The method degrades as the turn grows.** Confidence falls monotonically with
requested displacement, and by 80 units the correlation has lost the match
entirely. A one-dimensional column profile assumes the scene translates; a real
camera turn in a confined interior produces parallax, occlusion and perspective
change, and past some angle the two frames no longer share a profile to match.

**The numbers do not form a usable ratio.** 10 units gives 13 columns, 100 gives
198. That is not linear and the intermediate points contradict both readings.
The explanations — the game's own mouse acceleration or smoothing, the
correlation saturating, per-step quantisation in the emitter — are not separated
by this data.

So `degrees_per_unit` is **not** measured here, and writing one down would be a
guess. What is established is that input arrives and the camera responds.

## Three earlier verdicts that were wrong for an interesting reason

Before reaching free look, the probe returned "NOT REACHED by relative
movement" three times: on Xonotic sitting at a Join prompt, on Breathedge at a
disclaimer screen, and on Breathedge during a scripted tutorial segment with a
controls overlay open.

In every case the report was well formed, internally consistent and wrong. The
camera was locked, not unreachable.

**The probe cannot tell those apart, and no amount of care in the correlation
would let it.** "No displacement" has two causes and the difference is not in
the frames. The four-outcome table in `findings.md` now says so, and the
`NOT REACHED` wording names the camera-locked possibility rather than asserting
the interesting one.

That is the same failure as the first Xonotic run, in a different disguise: a
measurement with an unstated precondition, reported as a conclusion.

## What is not established

- **Four more games.** The criterion is four of five across engines. Xonotic,
  0 A.D., Battle for Wesnoth and SuperTuxKart are the remaining roster.
- **The clamp magnitude.** `06-action-and-input.md` open question 4 asks for the
  per-tick cap. Nothing here saturates in a way that identifies one.
- **Linearity.** The sweep is not monotonic and the cause is not separated.
- **Anything about the capture interface.** `PrintWindow` was live for this
  window and agreed with `ScreenCrop` on every measurement. That is a data point
  for `ADR-0008`, not a decision.

## What this changes in the specification

Nothing yet, and deliberately. One game is not the criterion the matrix sets,
and `known-good-matrix.md` gains a note rather than a resolved date.
