//! Known-good-matrix question 7, answered with the right instrument.
//!
//! `SetIsBorderRequired(false)` returning success is not an answer. A setter
//! that accepts a value and changes nothing is the commonest shape of platform
//! defect there is.
//!
//! **The first version of this comparison was also not an answer**, and it is
//! worth saying why, because the mistake is the one this project keeps having
//! to relearn. It captured the window twice through the compositor, once with
//! the border required and once without, and differenced the two frames. Both
//! readings came back at 0.00%, and it printed "no suppression".
//!
//! That conclusion was unsupported. Zero difference is equally consistent with
//! two different worlds: there is no border in either capture, or the border is
//! never drawn into a capture at all. **The instrument could not see the thing
//! it was reporting on.**
//!
//! The capture border is drawn by the compositor *around the window on screen*,
//! for the person sitting there. So the instrument that can see it is the one
//! that reads the screen — and the specification already draws this distinction
//! for the overlay: whether something is visible on screen is a question for a
//! screenshot, and whether a capture contains it is a question for the captured
//! frame. Conflating the two is how a capture feedback defect ships.
//!
//! Three phases, each a screen reading of the window's rectangle plus a margin:
//!
//! | Phase | What is running | What it establishes |
//! | --- | --- | --- |
//! | 1 | Nothing | The baseline: the window unadorned |
//! | 2 | A capture with the border **required** | Whether a border is drawn at all |
//! | 3 | A capture with the border **suppressed** | Whether suppression works |
//!
//! Phase 2 is the positive control, and it is what the first version lacked.
//! If phase 2 does not differ from phase 1, no border was ever drawn and phase
//! 3 proves nothing either way — which is a result, stated as one, rather than
//! a verdict the readings cannot carry.

use std::fmt::Write as _;
use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::HWND;

use crate::gdi;
use crate::measure::Frame;
use crate::wgc::Wgc;

/// How far outside the window rectangle to read.
///
/// The border is drawn around the window, so a reading confined to the window
/// would miss it. Twelve pixels is comfortably more than any border drawn.
const MARGIN: i32 = 12;

/// How thick a band at each edge counts as "where a border would be".
const BAND: usize = 16;

/// How long to let the compositor settle after a session starts.
///
/// The border is not drawn the instant `StartCapture` returns, and reading too
/// early reports its absence.
const SETTLE: Duration = Duration::from_millis(900);

/// Above this share of differing pixels, a change is real rather than noise.
const REAL: f64 = 0.5;

/// Run the three-phase comparison.
///
/// # Errors
///
/// Returns the step that failed.
pub(crate) fn compare(window: HWND) -> Result<String, String> {
    let mut out = String::new();
    let _ = writeln!(out, "--- capture border, unpackaged, question 7 ---");

    // A screen reading reads whatever is on screen, which is not necessarily
    // the target. Without this, a window behind another window yields a clean
    // reading of the wrong thing, twice, and every difference is zero — which
    // is what the first run of this comparison reported.
    if !gdi::is_unobscured(window) {
        return Err("the target window is covered, so a screen reading would be of something else. Bring it to the front, or use --own-target.".to_owned());
    }

    let baseline = gdi::capture_window_rect(window, MARGIN)?;
    let _ = writeln!(
        out,
        "  reading      {}x{} of screen, {MARGIN} px outside the window",
        baseline.width, baseline.height
    );

    let bordered = phase(window, true, &baseline, &mut out, "border required ")?;
    let suppressed = phase(window, false, &baseline, &mut out, "border suppressed")?;

    // The stability control, and the reason it is a fourth reading rather than
    // a looser threshold. The bordered phase moved the interior by 1.41% on the
    // first run that got this far, which under a simple rule disqualified a
    // result that was otherwise unambiguous — and widening the threshold to
    // admit it would have been fitting the test to the answer.
    //
    // Instead: read the window again with nothing capturing it. If that matches
    // the baseline, the window is still on its own, and movement during a phase
    // was caused by that phase rather than by the window.
    let settled = gdi::capture_window_rect(window, MARGIN)?;
    let drift = if settled.width == baseline.width && settled.height == baseline.height {
        difference(&baseline, &settled, Region::Interior)
    } else {
        f64::INFINITY
    };
    let _ = writeln!(
        out,
        "  nothing capturing   interior {drift:>6.2}%  (stability)"
    );

    let _ = writeln!(out);
    let verdict = if drift > REAL {
        "INCONCLUSIVE: the window is not still on its own, so nothing measured \
         here can be attributed to a capture session. Retry against a still window."
    } else if bordered.edge <= REAL {
        "INCONCLUSIVE: no border was drawn even when one was required, so this \
         run cannot say whether suppression works. Either this window is not \
         eligible for a border, or this build does not draw one."
    } else if suppressed.edge <= REAL && suppressed.interior <= REAL {
        "SUPPRESSED: a border appears when required, nothing appears when \
         suppressed, and the window is stable on its own. From an unpackaged \
         process."
    } else {
        "NOT SUPPRESSED: the border appears whether or not it is suppressed. \
         Every captured frame carries one, and perception must crop it."
    };
    let _ = writeln!(out, "  {verdict}");
    Ok(out)
}

/// What one phase measured against the baseline.
struct Reading {
    edge: f64,
    interior: f64,
}

/// Start a capture, let the compositor settle, and read the screen.
fn phase(
    window: HWND,
    border: bool,
    baseline: &Frame,
    out: &mut String,
    label: &str,
) -> Result<Reading, String> {
    let (mut session, requested) = if border {
        Wgc::start_bordered(window)?
    } else {
        Wgc::start(window)?
    };
    if let Err(error) = &requested.border_disabled {
        return Err(format!(
            "SetIsBorderRequired({border}) was refused: {error}"
        ));
    }

    // Frames are pulled while waiting, because a session whose frames are never
    // collected can be throttled, and a throttled session may not be adorned
    // the same way.
    let deadline = Instant::now() + SETTLE;
    while Instant::now() < deadline {
        match session.next() {
            Ok(_) => sleep(Duration::from_millis(10)),
            Err(error) => return Err(error),
        }
    }

    let screen = gdi::capture_window_rect(window, MARGIN)?;
    drop(session);

    if screen.width != baseline.width || screen.height != baseline.height {
        return Err(format!(
            "the window changed size during the '{}' phase",
            label.trim()
        ));
    }

    let edge = difference(baseline, &screen, Region::Edge);
    let interior = difference(baseline, &screen, Region::Interior);
    let _ = writeln!(
        out,
        "  {label}  edge {edge:>6.2}%   interior {interior:>6.2}%  (control)"
    );
    Ok(Reading { edge, interior })
}

/// Which part of the reading to difference.
#[derive(Clone, Copy)]
enum Region {
    /// The outermost [`BAND`] rows and columns, where a border would be.
    Edge,
    /// Everything inside them. The null control: the window's own content.
    Interior,
}

/// The share of pixels that differ, as a percentage.
fn difference(left: &Frame, right: &Frame, region: Region) -> f64 {
    let width = left.width;
    let height = left.height;
    let mut differing = 0_usize;
    let mut counted = 0_usize;

    for y in 0..height {
        for x in 0..width {
            let on_edge = x < BAND || y < BAND || x + BAND >= width || y + BAND >= height;
            if on_edge != matches!(region, Region::Edge) {
                continue;
            }
            counted += 1;
            let offset = (y * width + x) * 3;
            if offset + 2 >= left.pixels.len() || offset + 2 >= right.pixels.len() {
                continue;
            }
            // Eight levels of tolerance. Two screen readings of the same still
            // window are not always identical, and treating one unit as a
            // border would report a border everywhere.
            let apart = (0..3).any(|channel| {
                left.pixels[offset + channel].abs_diff(right.pixels[offset + channel]) > 8
            });
            if apart {
                differing += 1;
            }
        }
    }

    if counted == 0 {
        0.0
    } else {
        differing as f64 * 100.0 / counted as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: usize, height: usize, value: u8) -> Frame {
        Frame {
            pixels: vec![value; width * height * 3],
            width,
            height,
            elapsed: Duration::ZERO,
        }
    }

    #[test]
    fn an_unchanged_reading_differs_nowhere() {
        let a = solid(64, 64, 40);
        let b = solid(64, 64, 40);
        assert!(difference(&a, &b, Region::Edge) < 1e-9);
        assert!(difference(&a, &b, Region::Interior) < 1e-9);
    }

    #[test]
    fn a_change_confined_to_the_edge_leaves_the_interior_alone() {
        // This is the shape a border makes, and the shape the verdict keys on.
        let a = solid(64, 64, 40);
        let mut b = solid(64, 64, 40);
        for y in 0..64 {
            for x in 0..64 {
                if x < BAND || y < BAND || x + BAND >= 64 || y + BAND >= 64 {
                    let offset = (y * 64 + x) * 3;
                    b.pixels[offset] = 250;
                }
            }
        }
        assert!(difference(&a, &b, Region::Edge) > 99.0);
        assert!(difference(&a, &b, Region::Interior) < 1e-9);
    }

    #[test]
    fn a_repaint_moves_the_interior_too() {
        // The null control doing its job. A window that repainted between two
        // readings changes everywhere, and an edge difference then proves
        // nothing about a border.
        let a = solid(64, 64, 40);
        let b = solid(64, 64, 200);
        assert!(difference(&a, &b, Region::Edge) > 99.0);
        assert!(difference(&a, &b, Region::Interior) > 99.0);
    }

    #[test]
    fn small_noise_is_not_a_border() {
        let a = solid(64, 64, 40);
        let mut b = solid(64, 64, 40);
        b.pixels.fill(45);
        assert!(difference(&a, &b, Region::Edge) < 1e-9);
    }
}
