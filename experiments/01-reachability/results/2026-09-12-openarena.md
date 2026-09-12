# OpenArena 0.8.8, 2026-09-12 — reached, and the cross-check earns its keep

Fourth engine: id Tech 3.

```text
capture route: ScreenCrop
profile width: 1028 columns; movement threshold 5 columns

control, no input      lag     0  confidence 1.000  -> quiet
sweep    10 units    lag    13  confidence 0.978  -> moved
sweep    20 units    lag    25  confidence 0.953  -> moved
sweep    40 units    lag    50  confidence 0.922  -> moved
sweep    80 units    lag    83  confidence 0.857  -> moved
fit                    1.091 columns per unit, R2 0.975
sign                   +40 ->    50, -40 ->   -50  -> opposite
return to origin       lag     0  confidence 0.999  -> returned
control, absolute move lag  -257  confidence 0.727  -> ABSOLUTE ALSO TURNS

verdict: REACHED, and linear - one ratio describes the response
```

Wrap test: **3240 mouse units for one revolution, 0.1111 degrees per unit**, at
a match confidence of 0.950.

Started straight into a map from the command line — `+map oa_dm1` with
`r_fullscreen 0` — so there was no menu to navigate and no click to place. The
only roster title so far that needed no manual setup at all.

## The cross-check disagreed, and it was right to

For AssaultCube the two independent measurements implied a field of view of
110 degrees, which is ordinary. Here the same arithmetic gives **131 degrees**,
which is not.

| Fit over | Slope, columns per unit | R² | Implied field of view |
| --- | --- | --- | --- |
| All four samples | 1.091 | 0.975 | 130.9° |
| First three | 1.252 | **1.000** | 114.0° |

Dropping the 80-unit sample takes R² to 1.000 and the implied field of view to
114 degrees, which is an ordinary widescreen-corrected value for this game.

So the disagreement is not the game behaving strangely. **The 80-unit sample is
where the correlation starts to lose the match**, and including it drags the
slope down and inflates the implied field of view. The same check applied to
AssaultCube moves the answer by one degree, because its largest sample sits on
the line.

That is the cross-check doing exactly what a cross-check is for: it was tight
where the fit was clean and loose where it was not, and it located the loose
sample without being told which one to suspect.

## A number the specification has been waiting for

`06-action-and-input.md` open question 4 asks for the per-tick magnitude cap and
says it "is a guess until measured against real games". This does not answer
that question, but it answers a neighbouring one that matters as much.

**Frame correlation stops being a usable verification signal somewhere between
40 and 80 mouse units on this game** — around 50 columns of displacement on a
1028-column profile, which is roughly five percent of the frame.

That is a limit of the measurement rather than of the game, and it bounds
something the product needs: how far the view may move between two frames and
still be checkable by correlating them. A `look` step larger than that is not
wrong, but its effect cannot be verified by this method.

## An absolute reposition turns the camera here too

Lag of −257 columns at confidence 0.727, in the same direction as the reposition
and about a quarter of the frame.

That makes two engines of the four that **apply** the delta rather than
discarding or clamping it — Cube and id Tech 3 — against two that ignore it,
Unreal Engine 4 and DarkPlaces. The third outcome added to
`06-action-and-input.md` for AssaultCube is not an outlier. It is half the
sample.

## Setup

| | |
| --- | --- |
| Game | OpenArena 0.8.8, map `oa_dm1` |
| Engine | id Tech 3 |
| Window mode | Windowed 1280×720, from the command line |
| State | Live gameplay from launch, free look, cursor captured |
| Route | `ScreenCrop` |

## Question 1 status

**Four engines tested, four reached, no failures.**

| Game | Engine | Columns per unit | R² | Degrees per unit | Absolute reposition |
| --- | --- | --- | --- | --- | --- |
| Breathedge | Unreal Engine 4 | not resolved | — | not run | no turn |
| Xonotic | DarkPlaces | 0.892 | 0.957 | not run | no turn |
| AssaultCube | Cube | 1.072 | 1.000 | 0.0909 | turns, ~245 columns |
| OpenArena | id Tech 3 | 1.091 | 0.975 | 0.1111 | turns, ~257 columns |

The matrix asks for "five games of different engines" and the pass criterion is
four of five. **Four of four is a stronger result than four of five**, but the
denominator is not the one the protocol names, and saying the criterion is met
would be reading it generously.

What is settled: synthesised relative movement through the public input API
turns the camera on every engine tried, across two commercial-lineage and two
community engines, with a measured and repeatable response on three of them.
