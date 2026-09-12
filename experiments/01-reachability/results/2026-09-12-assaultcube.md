# AssaultCube 1.3.0.2, 2026-09-12 — reached, and the cleanest response yet

Third engine. Cube, which is a different lineage from both Unreal Engine 4 and
DarkPlaces.

```text
capture route: ScreenCrop
profile width: 1036 columns; movement threshold 5 columns

control, no input      lag     0  confidence 1.000  -> quiet
sweep    10 units    lag    11  confidence 0.992  -> moved
sweep    20 units    lag    22  confidence 0.979  -> moved
sweep    40 units    lag    42  confidence 0.963  -> moved
sweep    80 units    lag    86  confidence 0.950  -> moved
fit                    1.072 columns per unit, R2 1.000
sign                   +40 ->    42, -40 ->   -41  -> opposite
return to origin       lag     0  confidence 1.000  -> returned
control, absolute move lag   244  confidence 0.623  -> ABSOLUTE ALSO TURNS

verdict: REACHED, and linear - one ratio describes the response
```

**R² of 1.000.** The response is as close to a straight line through the origin
as this method can resolve: 11, 22, 42, 86 columns for 10, 20, 40 and 80 mouse
units.

A second run gave 1.134 columns per unit at R² 0.999, so the ratio repeats to
about six percent between runs.

## An absolute reposition turns the camera here

`06-action-and-input.md` said a reposition produces "no delta at all, or one
enormous delta that the game clamps or discards as implausible".

Cube does neither. It **applies** the delta: a lag of roughly 245 columns, a
quarter of the frame width, from a single reposition, reproduced across two
runs at 244 and 247.

That is a third outcome, and it is the one that matters most, because it is the
one that **looks like it worked**. A reposition a game ignores costs an action.
A reposition a game applies costs control of the camera, and the agent has no
way to tell which kind of game it is pointed at.

The page is amended, and the amendment strengthens `FR-ACT-004` rather than
weakening it. Xonotic produced no turn at all from the same control, so both of
the original outcomes and the new one are now measured rather than supposed.

## Two corrections to the probe, both found here

**The absolute control depended on cursor history.** Its first reading was
244 columns, its second zero — not because the game changed, but because the
cursor was already where the control moves it, so the second invocation
requested no displacement at all. I briefly wrote the zero down as the
interesting result and concluded the first reading had been an artefact. It was
the other way round.

The control now parks the cursor in one corner before moving it across, so it
requests a displacement of its own rather than whatever the last run left
behind. With that, the readings are 244 and 247 across two runs.

**The liveness precondition rejected a working target.** It waited for the scene
to move on its own, and AssaultCube does not: nothing in it animates while the
player stands still. The check refused to run against a game that was working
perfectly.

A stale capture and a genuinely motionless scene produce the same bytes twice,
so no amount of waiting separates them. The precondition now emits a small turn
of its own and asks whether the frame changed. That tests capture and input
together, which is what actually needs to be true, and it does not care whether
the scene has weather.

## Setup

| | |
| --- | --- |
| Game | AssaultCube 1.3.0.2, default map, no bots |
| Engine | Cube |
| Window mode | Windowed 1280×720 (`-t -w1280 -h720`) |
| State | Live gameplay, free look, cursor captured |
| Route | `ScreenCrop` |

Reaching gameplay needed the welcome menu dismissed, and **the click only
registered after the pointer was moved there first**. A click delivered at a
coordinate the pointer had not visited did nothing.

That is `FR-ACT-005` observed in the wild, on the first game that exercised it:

> Interfaces that activate on hover discard a click that arrives in the same
> frame as the pointer.

The requirement calls motion synthesis functional rather than cosmetic. This is
what that means in practice, and it was found by accident rather than by test.

## Calibration, and a cross-check that passed

The wrap test turns until the view comes back round:

```text
one revolution: 3960 mouse units (match confidence 1.000)
calibration:    0.0909 degrees per mouse unit
```

That is the figure `FR-ACT-004` needs, and it needs nothing else: no field of
view, no knowledge of the player's sensitivity setting, no assumption about how
far away the scenery is. Keep turning and watch for the frame to match where it
started.

**The two measurements agree, and they were taken by different means.** The
sweep gave 1.072 columns per unit on a profile spanning 80% of a 1286-pixel
window. If that profile covers 0.8 × FOV, then

    degrees per unit = 1.072 x 0.8 x FOV / 1036

Setting that equal to the wrap test's 0.0909 gives **FOV ≈ 110 degrees**, which
is an ordinary widescreen-corrected field of view for this game.

Neither measurement was derived from the other. The sweep correlates two frames
a few columns apart; the wrap counts a full revolution and never looks at a
displacement at all. Agreeing to a plausible field of view is the kind of check
that catches a method fooling itself, and this one passed.

## Question 1 status

**Three of five.** Unreal Engine 4, DarkPlaces, Cube.

| Game | Engine | Columns per unit | R² | Absolute reposition |
| --- | --- | --- | --- | --- |
| Breathedge | Unreal Engine 4 | not resolved | — | no turn |
| Xonotic | DarkPlaces | 0.892 | 0.957 | no turn |
| AssaultCube | Cube | 1.072 | 1.000 | **turns, ~245 columns** |

The criterion is four of five across engines, so the question stays open.
