//! The experiment: four conjunct tests and three null controls.
//!
//! A measured displacement on its own proves nothing. Scenes move by
//! themselves — an explosion, water, a day-night cycle — and a probe that
//! reported "the image shifted, so the camera turned" would hand the project's
//! gating question an answer it had not earned.
//!
//! So a positive result requires all four of these to hold, and all three
//! controls to come back clean.

use std::fmt::Write as _;
use std::thread::sleep;
use std::time::Duration;

use afk_input::{MAX_RELATIVE_STEP, Refused, Synthesiser};

use crate::capture::{self, CaptureError, Route};
use crate::correlate::{Profile, displacement};

/// Frames to discard after emitting, so the game has rendered the result.
const SETTLE_FRAMES: u32 = 3;

/// Milliseconds between synthesised events, roughly a reflex tick.
const STEP_INTERVAL_MS: u64 = 16;

/// Total displacements swept, in mouse units, for the linearity test.
const SWEEP: [i32; 4] = [100, 200, 400, 800];

/// Correlation below this is not a measurement, whatever lag it names.
const MIN_CONFIDENCE: f64 = 0.80;

/// Fraction of the profile width a displacement must exceed to count as a turn.
const MIN_TURN_FRACTION: f64 = 0.02;

/// What went wrong before any verdict could be reached.
#[derive(Debug)]
pub(crate) enum ProbeError {
    WindowNotFound(String),
    Capture(CaptureError),
    NeverForeground,
    Refused(Refused),
    Degenerate,
    /// Consecutive captures of a live scene were pixel-identical.
    ///
    /// The failure this probe is most likely to get wrong, so it is checked
    /// before anything else runs. Identical frames correlate at 1.000 with zero
    /// displacement, which reads as "the camera did not turn" — and that is the
    /// project's gating question answered wrongly, in the most confident
    /// possible tone.
    ///
    /// Two causes, and neither is an answer to question 1: the capture route
    /// returns a stale or static image, or the window is showing something that
    /// does not move, such as a menu.
    StaticCapture {
        frames: u32,
    },
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowNotFound(title) => {
                write!(f, "no visible window with a title containing {title:?}")
            }
            Self::Capture(error) => write!(f, "capture: {error}"),
            Self::NeverForeground => f.write_str(
                "the target never came to the foreground, so nothing was sent. \
                 INV-ACT-001 is doing its job; bring the game forward during the \
                 arming delay",
            ),
            Self::Refused(refused) => write!(f, "input refused: {refused}"),
            Self::Degenerate => f.write_str(
                "the frames carried too little texture to correlate. A flat scene \
                 cannot answer this question; point the camera at something",
            ),
            Self::StaticCapture { frames } => write!(
                f,
                "{frames} consecutive captures were pixel-identical, so nothing \
                 measured afterwards would mean anything. Either the capture \
                 route returns a stale image for this window, or the window is \
                 showing something static such as a menu. That is an ADR-0008 \
                 finding, not an answer to question 1"
            ),
        }
    }
}

impl std::error::Error for ProbeError {}

/// One emit-and-measure cycle.
struct Measurement {
    /// Total mouse units requested.
    requested: i32,
    /// Columns the scene moved.
    lag: i32,
    /// Correlation at that lag.
    confidence: f64,
    /// Profile width, so the lag can be read as a fraction.
    width: usize,
}

impl Measurement {
    fn is_turn(&self) -> bool {
        self.confidence >= MIN_CONFIDENCE
            && f64::from(self.lag.abs()) > (self.width as f64) * MIN_TURN_FRACTION
    }
}

/// Emit `total` mouse units horizontally, in steps the input layer accepts.
///
/// `FR-ACT-004`: a camera turn is a sequence of small relative movements, never
/// one large one. Games reading raw input clamp or discard a single implausible
/// delta, so a probe that sent the whole amount at once would be measuring the
/// game's sanity check rather than whether input arrives.
fn emit_relative(synth: &Synthesiser, total: i32) -> Result<(), ProbeError> {
    let step = MAX_RELATIVE_STEP.min(40);
    let steps = (total.abs() + step - 1) / step;
    let direction = total.signum();
    for index in 0..steps {
        let remaining = total.abs() - index * step;
        let magnitude = step.min(remaining);
        synth
            .move_relative(direction * magnitude, 0)
            .map_err(ProbeError::Refused)?;
        sleep(Duration::from_millis(STEP_INTERVAL_MS));
    }
    Ok(())
}

fn settle() {
    sleep(Duration::from_millis(
        STEP_INTERVAL_MS * u64::from(SETTLE_FRAMES),
    ));
}

fn profile(window: capture::Hwnd) -> Result<(Profile, Route), ProbeError> {
    let frame = capture::capture(window).map_err(ProbeError::Capture)?;
    let profile =
        Profile::from_gray(&frame.gray, frame.width, frame.height).ok_or(ProbeError::Degenerate)?;
    if profile.is_featureless() {
        return Err(ProbeError::Degenerate);
    }
    Ok((profile, frame.route))
}

/// Emit a displacement and measure what the scene did.
fn cycle(
    synth: &Synthesiser,
    window: capture::Hwnd,
    total: i32,
) -> Result<Measurement, ProbeError> {
    let (before, _) = profile(window)?;
    if total != 0 {
        emit_relative(synth, total)?;
    } else {
        // The no-input control waits exactly as long as a real emission would,
        // so that anything the scene does on its own has the same chance to
        // show up. A shorter wait would make the control easier to pass than
        // the test, which would make it worthless.
        sleep(Duration::from_millis(
            STEP_INTERVAL_MS * u64::from((SWEEP[0].unsigned_abs() / 40).max(1)),
        ));
    }
    settle();
    let (after, _) = profile(window)?;

    let found = displacement(&before, &after).ok_or(ProbeError::Degenerate)?;
    Ok(Measurement {
        requested: total,
        lag: found.lag,
        confidence: found.confidence,
        width: before.len(),
    })
}

/// Run the experiment and return a human-readable report.
pub(crate) fn execute(title: &str, arm_seconds: u64) -> Result<String, ProbeError> {
    let window =
        capture::find_window(title).ok_or_else(|| ProbeError::WindowNotFound(title.to_owned()))?;

    println!("found window for {title:?}");
    println!("bring it to the foreground; starting in {arm_seconds}s");
    sleep(Duration::from_secs(arm_seconds));

    let synth = Synthesiser::bind(window.cast());
    if !synth.target_is_foreground() {
        return Err(ProbeError::NeverForeground);
    }

    // Establish that capture is live before attributing anything to input.
    //
    // This check exists because of a run that produced a confident, complete
    // and entirely wrong verdict: every measurement came back at confidence
    // 1.000 with zero displacement, which is the signature of identical frames
    // rather than of a camera that did not move.
    println!("checking whether any capture route is live:");
    let route = liveness(window)?;
    println!("capture route: {route}");

    let mut report = String::new();
    let _ = writeln!(report, "window: {title:?}");
    let _ = writeln!(report, "capture route: {route}");
    let _ = writeln!(report);

    // --- Null control 1: no input ------------------------------------------
    //
    // If this produces a signal, the detector is measuring animation and every
    // positive result below is worthless.
    let quiet = cycle(&synth, window, 0)?;
    let control_clean = !quiet.is_turn();
    let _ = writeln!(
        report,
        "control, no input      lag {:>5}  confidence {:.3}  -> {}",
        quiet.lag,
        quiet.confidence,
        verdict(control_clean, "quiet", "SCENE MOVES ON ITS OWN")
    );

    // --- Test 2: monotonicity and linearity --------------------------------
    let mut sweep = Vec::new();
    for total in SWEEP {
        let measured = cycle(&synth, window, total)?;
        let _ = writeln!(
            report,
            "sweep {:>5} units    lag {:>5}  confidence {:.3}  -> {}",
            measured.requested,
            measured.lag,
            measured.confidence,
            verdict(measured.is_turn(), "moved", "no movement")
        );
        sweep.push(measured);
        // Return the view so each sweep step starts from the same place.
        emit_relative(&synth, -total)?;
        settle();
    }
    let monotonic = sweep
        .windows(2)
        .all(|pair| pair[1].lag.abs() >= pair[0].lag.abs());

    let signs_oppose = sign_test(&synth, window, &mut report)?;

    // --- Test 3: return to origin ------------------------------------------
    //
    // The single strongest test here. It proves both that input arrived and
    // that the mapping is stable, and a scene moving on its own has no reason
    // to come back.
    let (origin, _) = profile(window)?;
    emit_relative(&synth, SWEEP[2])?;
    settle();
    emit_relative(&synth, -SWEEP[2])?;
    settle();
    let (returned, _) = profile(window)?;
    let round_trip = displacement(&origin, &returned).ok_or(ProbeError::Degenerate)?;
    let returns = round_trip.lag.abs() <= 2 && round_trip.confidence >= 0.90;
    let _ = writeln!(
        report,
        "return to origin       lag {:>5}  confidence {:.3}  -> {}",
        round_trip.lag,
        round_trip.confidence,
        verdict(returns, "returned", "DRIFTED")
    );

    let absolute_quiet = absolute_control(&synth, window, &mut report)?;

    let reached = sweep.iter().any(Measurement::is_turn);
    let verdict_line = match (reached, control_clean, signs_oppose, returns, monotonic) {
        (true, true, true, true, true) => "REACHED, and linear",
        (true, true, true, true, false) => {
            "REACHED, but not monotonic - acceleration or a clamp; calibration \
             needs a table rather than a ratio"
        }
        (true, true, _, _, _) => {
            "INCONCLUSIVE - movement detected but the sign or return test failed"
        }
        (true, false, _, _, _) => {
            "INCONCLUSIVE - the scene moves on its own; the control is not clean"
        }
        (false, _, _, _, _) => "NOT REACHED by relative movement",
    };

    let _ = writeln!(report);
    let _ = writeln!(report, "verdict: {verdict_line}");
    if !absolute_quiet {
        let _ = writeln!(
            report,
            "note: absolute movement also turned the view. FR-ACT-004's rationale \
             is weaker than 06-action-and-input.md claims for this game, and the \
             page needs amending."
        );
    }

    synth.release_all();
    Ok(report)
}

/// Frames sampled when checking that the scene is live.
const LIVENESS_FRAMES: u32 = 12;

/// Confirm that some capture route produces frames that actually change.
///
/// Returns the live route. Fails rather than proceeding, because every test
/// below correlates two frames, and two identical frames correlate perfectly at
/// zero displacement — which reads exactly like "the input never arrived".
///
/// Each route is checked on its own. "Produced a frame with content" and
/// "produced a frame that changes" are different questions, and a route can
/// return a perfectly detailed image that is the same image every time. The
/// per-route result is the table `ADR-0008` needs.
fn liveness(window: capture::Hwnd) -> Result<Route, ProbeError> {
    let mut last_error = None;

    for route in capture::ROUTES {
        match route_is_live(window, route) {
            Ok(true) => {
                println!("  {route}: live");
                return Ok(route);
            }
            Ok(false) => println!("  {route}: static across {LIVENESS_FRAMES} frames"),
            Err(error) => {
                println!("  {route}: {error}");
                last_error = Some(error);
            }
        }
    }

    if let Some(error) = last_error {
        return Err(ProbeError::Capture(error));
    }
    Err(ProbeError::StaticCapture {
        frames: LIVENESS_FRAMES,
    })
}

fn route_is_live(window: capture::Hwnd, route: Route) -> Result<bool, capture::CaptureError> {
    let mut previous = capture::capture_via(window, route)?.gray;
    for _ in 1..LIVENESS_FRAMES {
        sleep(Duration::from_millis(80));
        let frame = capture::capture_via(window, route)?;
        if frame.gray != previous {
            return Ok(true);
        }
        previous = frame.gray;
    }
    Ok(false)
}

/// Test one: reversing the input reverses the displacement.
///
/// A scene changing on its own has no reason to correlate with the sign of
/// what was sent, which is what makes this the cheapest way to tell a camera
/// turn from an explosion.
fn sign_test(
    synth: &Synthesiser,
    window: capture::Hwnd,
    report: &mut String,
) -> Result<bool, ProbeError> {
    let amount = SWEEP[2];

    let right = cycle(synth, window, amount)?;
    emit_relative(synth, -amount)?;
    settle();

    let left = cycle(synth, window, -amount)?;
    emit_relative(synth, amount)?;
    settle();

    let opposed = right.is_turn() && left.is_turn() && right.lag.signum() != left.lag.signum();
    let _ = writeln!(
        report,
        "sign                   +{amount} -> {:>5}, -{amount} -> {:>5}  -> {}",
        right.lag,
        left.lag,
        verdict(opposed, "opposite", "SAME SIGN")
    );
    Ok(opposed)
}

/// Null control three: an absolute reposition.
///
/// `FR-ACT-004` predicts this produces no turn in a game reading raw input. If
/// it works as well as relative movement, that requirement's rationale is
/// weaker than `06-action-and-input.md` claims and the page needs amending —
/// a result worth being able to get.
fn absolute_control(
    synth: &Synthesiser,
    window: capture::Hwnd,
    report: &mut String,
) -> Result<bool, ProbeError> {
    let (before, _) = profile(window)?;
    // A refusal here is not a failure of the control: it is the foreground
    // guard, and it means the same thing as no movement.
    let _ = synth.move_absolute(20_000, 32_000);
    settle();
    let (after, _) = profile(window)?;
    let moved = displacement(&before, &after).ok_or(ProbeError::Degenerate)?;
    let quiet = moved.lag.abs() <= 2;
    let _ = writeln!(
        report,
        "control, absolute move lag {:>5}  confidence {:.3}  -> {}",
        moved.lag,
        moved.confidence,
        verdict(quiet, "no turn, as specified", "ABSOLUTE ALSO TURNS")
    );
    Ok(quiet)
}

fn verdict(ok: bool, yes: &'static str, no: &'static str) -> &'static str {
    if ok { yes } else { no }
}
