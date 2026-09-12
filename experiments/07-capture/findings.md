# Question 7 — can the capture border be suppressed from an unpackaged application?

**Status: yes, reproduced three times with a positive control, a null control
and a stability control.** The route comparison `ADR-0008` needs came out of the
same harness, and so did the first entries in the failure-signature table
`FR-PERC-009` has been waiting for.

Run on 2026-09-12 on the reference hardware, Windows 11 Pro build 26200.

## The answer, and the three controls that make it one

```text
--- capture border, unpackaged, question 7 ---
  reading      664x664 of screen, 12 px outside the window
  border required   edge   4.54%   interior   1.41%  (control)
  border suppressed  edge   0.00%   interior   0.00%  (control)
  nothing capturing   interior   0.00%  (stability)

  SUPPRESSED
```

Identical across three consecutive runs, to two decimal places.

| Reading | What it rules out |
| --- | --- |
| The border **required** phase moves the edge | That no border is ever drawn, which would make the suppressed phase meaningless |
| The border **suppressed** phase moves nothing | That the setter is accepted and ignored |
| **Nothing capturing** moves nothing | That the window changes on its own, which would make every other reading unattributable |

`SetIsBorderRequired(false)` returning success was never the answer.
A setter that accepts a value and changes nothing is the commonest shape of
platform defect there is, and this probe's first two attempts both produced a
confident verdict from an instrument that could not see the thing.

## Two wrong answers on the way, both instructive

**The first comparison could not see a border at all.** It captured the window
twice through the compositor, once with the border required and once without,
and differenced the two frames. Both came back 0.00%, and it printed
"NO SUPPRESSION".

That conclusion was unsupported. Zero difference is equally consistent with two
different worlds: there is no border in either capture, or **the border is never
drawn into a capture at all**. The border is drawn by the compositor around the
window *on screen*, for the person sitting there — so the instrument that can
see it is the one that reads the screen.

The specification already makes this distinction, for the overlay: whether
something is visible on screen is a question for a screenshot, and whether a
capture contains it is a question for the captured frame. Conflating them is how
a capture feedback defect ships. This probe conflated them on its first try.

**The second comparison read the wrong window.** Once it read the screen, it
read whatever was on screen at the target's coordinates — which, for a window
behind another window, is a clean reading of something else, twice, with every
difference at zero.

The occlusion check that fixed it fired on its first run, against the very
window the previous attempt had silently measured.

Asking the window manager to bring somebody else's window forward does not
solve it either: `SetForegroundWindow` from a background process is refused by
design, which the probe observed rather than assumed.

So the probe makes its own target: a flat grey topmost window on its own message
loop. Visible, still, and not somebody's browser.

**The interior moved 1.41% in the bordered phase**, which under the original
rule disqualified an otherwise unambiguous result. Widening the threshold to
admit it would have been fitting the test to the answer. Instead the comparison
gained a fourth reading — the window with nothing capturing it — which
establishes that the window is still on its own, so movement during a phase is
attributable to that phase. The 1.41% is the border reaching inside the
16-pixel edge band, and it is recorded rather than explained away.

## The route comparison

Four routes, each measured for eight seconds at a requested 30 Hz.

### A windowed OpenGL game — AssaultCube 1.3.0.2 at 1280x720

| Route | fps | p50 ms | p95 ms | Verdict |
| --- | --- | --- | --- | --- |
| `PrintWindow` | 29.7 | 12.92 | 16.76 | usable |
| `BitBlt` | 29.8 | 3.56 | 4.19 | usable |
| `ScreenCrop` | 29.7 | 10.33 | 12.96 | usable |
| **Windows Graphics Capture** | **55.1** | **2.27** | **2.61** | **usable** |

The compositor route is four to six times cheaper per frame than anything else,
and it is the only one that delivered **above** the requested rate — 55 frames a
second, because it delivers on the game's own presentation rather than on a
clock the probe set.

At 2.27 ms it takes 7% of a 33 ms reflex tick. The nearest alternative takes 11%
and the worst takes 39%.

### A compositor-drawn application window

The same four routes against an ordinary modern application window, at
2560x1392:

| Route | fps | p50 ms | Verdict |
| --- | --- | --- | --- |
| `PrintWindow` | 27.5 | 34.80 | **stale surface** |
| `BitBlt` | 29.7 | 15.70 | **black frames** |
| `ScreenCrop` | 28.1 | 34.53 | usable |
| Windows Graphics Capture | 0.1 | 10.56 | **too few frames** |

**This row is the finding `FR-PERC-009` needs**, because three of the four
failures are silent. Every call succeeded. Nothing returned an error.

| Signature | What it means | What it is not |
| --- | --- | --- |
| 100% black frames | The route cannot see content drawn through a swap chain | A window that is not rendering |
| 100% repeated frames | A surface that succeeds for ever and never updates | A still scene |
| Under five frames in eight seconds | The compositor delivers on change and nothing changed | A broken route |

The last one is why the verdict vocabulary has five values rather than two, and
why `too few frames` exists at all: the first version of this harness read a
single frame from a static window and called the route **usable**. One sample
supports neither verdict.

## What this settles and what it does not

**Settled.** The capture border can be suppressed from an unpackaged process on
this build. Every captured frame does **not** carry a border, so the perception
layer does not need a crop step, and `09-gui-and-overlay.md` open question 6
loses the branch it was worried about.

Cursor suppression is accepted on every run as well, which `FR-PERC-006` needs:
the pointer in a captured frame is the agent's own, and perception that treats
it as an element will click on itself.

**Not settled.** This is one window mode on one build.

- **Borderless fullscreen and exclusive fullscreen are not measured.** Both are
  where capture routes historically differ most, and neither was reachable
  without a person at the keyboard to change the game's video settings.
- **Desktop duplication cropped to the window** is the third backend `ADR-0008`
  lists and it is not implemented here. The two routes measured already separate
  by a factor of five, and duplication's known cost is that it captures the
  whole display, which this project does not want.
- **Multiple displays and mixed scaling** are untested. `INV-GND-002` exists
  because a wrong transform lands clicks near the target, and "near" reads as a
  bad model rather than as bad arithmetic.
- **Graphics memory per route** is not measured, which the plan asked for. The
  probe reads frames back into main memory, so its own footprint would dominate.

## Method

```bash
cargo build --release -p capture-probe

# The route comparison, against whatever window matches.
capture-probe --window "AssaultCube" --seconds 8

# Question 7, against a window the probe creates: flat grey, topmost, still.
capture-probe --own-target

# What the probe can see, for when it cannot find a target.
capture-probe --list
```

No input is synthesised and none can be: `tools/check_call_sites.py` rule A
confines those spellings to `afk-input` and `afk-guardian`, and it scans this
directory too. The roster's `synthesis_allowed` therefore does not gate this
probe — capturing a window is not acting on it, and a title whose terms forbid
automation may still be looked at.

Full transcripts in `results/`.
