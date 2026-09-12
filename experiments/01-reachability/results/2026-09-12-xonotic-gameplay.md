# Xonotic 0.8.6, 2026-09-12 — reached, and linear

Second engine, and the one that matters most: DarkPlaces reads raw relative
input and captures the cursor, which is the case `FR-ACT-004` is written about.

```text
capture route: ScreenCrop
profile width: 1028 columns; movement threshold 5 columns

control, no input      lag     0  confidence 1.000  -> quiet
sweep    10 units    lag     6  confidence 0.922  -> moved
sweep    20 units    lag    12  confidence 0.863  -> moved
sweep    40 units    lag    28  confidence 0.805  -> moved
sweep    80 units    lag    77  confidence 0.780  -> moved
fit                    0.892 columns per unit, R2 0.957
sign                   +40 ->    28, -40 ->   -25  -> opposite
return to origin       lag     0  confidence 0.999  -> returned
control, absolute move lag     0  confidence 1.000  -> no turn, as specified

verdict: REACHED, and linear - one ratio describes the response
```

All four conjunct tests pass and all three null controls are clean. It is the
first run of this probe that has done both.

## The measurement

**0.892 columns of scene displacement per mouse unit**, on a 1028-column
profile taken from a 1286-pixel window, at the game's default sensitivity.

R² of 0.957 against a line through the origin. Through the origin rather than
with an intercept, deliberately: zero input must mean zero displacement, and an
intercept would be the fit quietly absorbing the scene's own movement — which
is what the control measures instead.

That number is in columns rather than degrees. Turning it into
`degrees_per_unit` needs the field of view, or the wrap test described in
`findings.md`, and neither has been run.

## The absolute control earns its place here

`FR-ACT-004` claims that repositioning the cursor produces no camera turn in a
game reading raw input, and that the claim is why the whole executor is built
around relative movement.

Against Xonotic, in the same run, under the same timing: relative movement
produced a clean linear response, and an absolute reposition produced a lag of
zero at confidence 1.000.

The requirement's rationale holds for this game, measured rather than assumed.

## What changed to get this result, and what did not

Two earlier Xonotic runs reported "not reached". Neither was a lie about the
game; both were the probe being wrong in a way worth recording.

**The first was a stale measurement.** Every reading came back at correlation
exactly 1.000 — the signature of comparing a frame with itself. The game was
sitting on a Join prompt with the world paused.

**The second was a guessed threshold.** The probe required a displacement to
exceed two percent of the frame width, and it rejected exactly the data above:
28 columns on a 1028-column profile is 2.7%, and its mirror at 25 columns is
2.4%, so the sign test failed on a response that is visibly clean.

The threshold is now **measured rather than chosen**. The no-input control runs
first, under identical timing, and reports what the scene does when left alone.
Everything else is judged against that, in this game, in this run.

An absolute constant could not do this job: the right number depends on what is
in the scene, not on the resolution. Water, a skybox and a flickering light all
move on their own, and how much they move is a property of the game.

What was **not** changed: the confidence floor was lowered to 0.5, and that
deserves scrutiny, because loosening a threshold until a result passes is the
failure mode this project's own ratchet exists to catch.

The justification is that the original bar had the relationship backwards.
Confidence falls as the turn grows — a larger rotation shares less of its
profile with the frame before it — so a high bar rejects the readings carrying
the most signal. The no-input control returns 0.998 to 1.000. **High confidence
is the signature of nothing happening.** The floor now says only that the
correlation found something, and the displacement relative to the baseline does
the discriminating.

## Setup

| | |
| --- | --- |
| Game | Xonotic 0.8.6, map Dance, Capture the Flag |
| Engine | DarkPlaces |
| Window mode | Windowed 1280×720 |
| State | Live gameplay after joining, free look, cursor captured |
| Route | `ScreenCrop`, which reads the display and so sees what a person sees |

Started with `+cl_allow_uidtracking 0 +cl_allow_uid2name 0`, which answers the
first-run prompt about sharing a nickname with a statistics service with the
privacy-preserving values rather than by clicking a consent dialog.

Reaching gameplay needed one click on `Join`. That is a setup step an operator
performs; the probe cannot verify that the camera is free, because "no
displacement" looks identical whether the input failed or the camera was locked.

## Evidence

`E:\afk-agent\evidence\q1\xonotic-before.png` and `xonotic-after.png` — a 600
unit turn, captured either side. The world rotates, the weapon and head-up
display do not, and the minimap rotates with the player's facing. Kept outside
this public repository.

## Question 1 status

**Two of five.** Breathedge on Unreal Engine 4, Xonotic on DarkPlaces.

Still open by the matrix's own criterion, which asks for four of five across
engines.
