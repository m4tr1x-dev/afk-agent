//! Which capture route works on which window mode, and how each one fails.
//!
//! Two questions, and the second is the one that cannot be reasoned out.
//!
//! **Which route to ship** is `ADR-0008`, and Windows Graphics Capture is the
//! expected answer. An experiment that only confirmed that would be worth
//! little.
//!
//! **How each route fails** is `FR-PERC-009`, which asks the system to tell an
//! unsupported window mode apart from a transient failure. That requires
//! running each route against modes it cannot handle and writing down what came
//! back — and what comes back is rarely an error. `PrintWindow` on a game
//! returns black. A stale surface returns the same correct frame for ever. Both
//! succeed, and a comparison that records only success and failure would call
//! them working.
//!
//! So every cell records six readings, four of them about failure, and the
//! verdict has five values rather than two.
//!
//! ```text
//! capture-probe --window "AssaultCube" --seconds 10
//! capture-probe --window "Notepad" --route wgc --seconds 5
//! ```
//!
//! No input is synthesised here and none can be: `tools/check_call_sites.py`
//! rule A confines those spellings to `afk-input` and `afk-guardian`, and it
//! scans this directory too. The roster's `synthesis_allowed` therefore does
//! not gate this probe — capturing a window is not acting on it, and a title
//! whose terms forbid automation may still be looked at.

// The probe reduces frames to statistics and talks to a platform whose
// interfaces are 32-bit signed where this code counts in usize. Writing each
// conversion as a checked one would bury the measurement in ceremony.
// `experiments/` is explicitly not held to product standard.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
#![cfg_attr(test, allow(clippy::expect_used))]

mod border;
mod gdi;
mod measure;
mod target;
mod wgc;

use std::fmt::Write as _;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::HWND;

use gdi::Gdi;
use measure::Tally;

/// How long each route is measured for, by default.
///
/// Long enough that a stale surface is unmistakable — a route delivering one
/// frame and then repeating it looks fine over a single second.
const DEFAULT_SECONDS: u64 = 10;

/// The pace the reflex loop would ask for, from `15-performance-budgets.md`.
const TARGET_HZ: u64 = 30;

fn main() -> ExitCode {
    let mut needle = String::new();
    let mut seconds = DEFAULT_SECONDS;
    let mut only: Option<String> = None;
    let mut arm = 0_u64;
    let mut border_only = false;
    let mut front = false;
    let mut own = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--window" => needle = args.next().unwrap_or_default(),
            "--seconds" => {
                seconds = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(DEFAULT_SECONDS);
            }
            "--route" => only = args.next(),
            "--arm" => arm = args.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            "--border" => border_only = true,
            "--front" => front = true,
            "--own-target" => own = true,
            "--list" => {
                for title in visible_windows() {
                    println!("{title}");
                }
                return ExitCode::SUCCESS;
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}\n\n{USAGE}");
                return ExitCode::from(2);
            }
        }
    }

    // The border comparison can supply its own target, which is the only way
    // to get a window that is visible, still and not somebody's browser.
    if own {
        let target = match target::Target::open() {
            Ok(target) => target,
            Err(error) => {
                eprintln!("capture-probe: {error}");
                return ExitCode::from(1);
            }
        };
        let outcome = border::compare(target.handle());
        drop(target);
        return match outcome {
            Ok(report) => {
                print!("{report}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("capture-probe: {error}");
                ExitCode::from(1)
            }
        };
    }

    if needle.is_empty() {
        eprintln!("--window is required\n\n{USAGE}");
        return ExitCode::from(2);
    }

    if arm > 0 {
        eprintln!("bring the target forward; starting in {arm}s");
        sleep(Duration::from_secs(arm));
    }

    match run(&needle, seconds, only.as_deref(), border_only, front) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("capture-probe: {error}");
            ExitCode::from(1)
        }
    }
}

const USAGE: &str = "\
capture-probe --window <title substring> [--seconds N] [--route NAME] [--arm N]

  --window   Part of the target window's title. First visible match wins.
  --seconds  How long to measure each route. Default 10, which is long enough
             for a stale surface to be unmistakable.
  --route    Measure one route only: printwindow, bitblt, screencrop or wgc.
  --arm      Wait this many seconds before starting, to bring a game forward.
  --list     Print every visible window title and exit. A probe that cannot
             find its target should be able to say what it can see.
  --front    Bring the target window to the front before measuring. Off by
             default: a probe that rearranges the desktop without being asked
             is a probe nobody runs twice.
  --own-target
             Answer question 7 against a window the probe creates itself: flat
             grey, topmost, still. The only way to get a target that is
             visible, unchanging and not somebody else's browser.
  --border   Skip the route comparison and answer question 7 instead: capture
             the window twice, with the capture border required and
             suppressed, and difference the outermost pixels. A setter that
             returns success is not evidence that anything changed.
";

fn run(
    needle: &str,
    seconds: u64,
    only: Option<&str>,
    border_only: bool,
    front: bool,
) -> Result<String, String> {
    let window = find_window(needle)
        .ok_or_else(|| format!("no visible window with a title containing {needle:?}"))?;
    let title = window_title(window);

    let mut out = String::new();
    let _ = writeln!(out, "window   {title}");

    if front {
        gdi::bring_to_front(window);
        sleep(Duration::from_millis(600));
    }

    if border_only {
        let _ = writeln!(out);
        out.push_str(&border::compare(window)?);
        return Ok(out);
    }

    let _ = writeln!(
        out,
        "measured {seconds}s per route, asking for {TARGET_HZ} Hz"
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{:<12} {:>6} {:>8} {:>8} {:>7} {:>7} {:>7}  verdict",
        "route", "fps", "p50 ms", "p95 ms", "black", "flat", "repeat"
    );

    let mut rows = Vec::new();
    for route in [Gdi::PrintWindow, Gdi::BitBlt, Gdi::ScreenCrop] {
        if let Some(only) = only
            && !route.name().eq_ignore_ascii_case(only)
        {
            continue;
        }
        rows.push((
            route.name().to_owned(),
            measure_gdi(window, route, seconds),
            None,
        ));
    }

    if only.is_none_or(|only| only.eq_ignore_ascii_case("wgc")) {
        let (tally, note) = measure_wgc(window, seconds);
        rows.push(("WGC".to_owned(), tally, Some(note)));
    }

    for (name, tally, note) in &rows {
        let _ = writeln!(
            out,
            "{:<12} {:>6.1} {:>8.2} {:>8.2} {:>6.0}% {:>6.0}% {:>6.0}%  {}",
            name,
            tally.fps(),
            tally.latency_ms(50.0),
            tally.latency_ms(95.0),
            tally.share(tally.black),
            tally.share(tally.featureless),
            tally.share(tally.repeated),
            tally.verdict(),
        );
        if let Some(note) = note {
            let _ = writeln!(out, "               {note}");
        }
        if let Some(error) = &tally.first_error {
            let _ = writeln!(
                out,
                "               {} error(s), first: {error}",
                tally.errors
            );
        }
    }

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "frames are {}x{} where a route produced any",
        rows.iter().map(|r| r.1.width).max().unwrap_or(0),
        rows.iter().map(|r| r.1.height).max().unwrap_or(0),
    );
    Ok(out)
}

/// Measure one window-manager route.
fn measure_gdi(window: HWND, route: Gdi, seconds: u64) -> Tally {
    let mut tally = Tally::default();
    let interval = Duration::from_micros(1_000_000 / TARGET_HZ);
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let started = Instant::now();
    let mut previous = None;

    while Instant::now() < deadline {
        let step = Instant::now();
        match gdi::capture(window, route) {
            Ok(frame) => previous = Some(tally.record(&frame, previous)),
            Err(error) => tally.record_error(error),
        }
        // Pace to the target rate rather than spinning. A route measured flat
        // out reports a number no reflex loop would ever ask for.
        if let Some(remaining) = interval.checked_sub(step.elapsed()) {
            sleep(remaining);
        }
    }
    tally.wall = started.elapsed();
    tally
}

/// Measure the compositor route, and report what it accepted.
fn measure_wgc(window: HWND, seconds: u64) -> (Tally, String) {
    let mut tally = Tally::default();
    let (mut session, requested) = match wgc::Wgc::start(window) {
        Ok(pair) => pair,
        Err(error) => {
            tally.record_error(error);
            return (tally, "the session did not start".to_owned());
        }
    };

    let note = format!(
        "cursor suppression {}; capture border suppression {}",
        describe(&requested.cursor_disabled),
        describe(&requested.border_disabled),
    );

    let deadline = Instant::now() + Duration::from_secs(seconds);
    let started = Instant::now();
    let mut previous = None;
    while Instant::now() < deadline {
        match session.next() {
            Ok(Some(frame)) => previous = Some(tally.record(&frame, previous)),
            // Nothing pending. The compositor delivers on change, so an idle
            // window legitimately produces nothing, and counting that as a
            // failure would report every static window as a broken route.
            Ok(None) => sleep(Duration::from_millis(2)),
            Err(error) => {
                tally.record_error(error);
                break;
            }
        }
    }
    tally.wall = started.elapsed();
    (tally, note)
}

fn describe(outcome: &Result<(), String>) -> String {
    match outcome {
        Ok(()) => "accepted".to_owned(),
        Err(error) => format!("REFUSED ({error})"),
    }
}

/// Find the first visible window whose title contains `needle`.
fn find_window(needle: &str) -> Option<HWND> {
    use std::sync::Mutex;

    use windows::Win32::Foundation::LPARAM;
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible};
    use windows::core::BOOL;

    /// Keep enumerating.
    const CARRY_ON: BOOL = BOOL(1);

    static FOUND: Mutex<Option<isize>> = Mutex::new(None);
    static NEEDLE: Mutex<String> = Mutex::new(String::new());

    unsafe extern "system" fn visit(window: HWND, _: LPARAM) -> BOOL {
        // SAFETY: `window` is supplied by EnumWindows and is live for the
        // duration of the callback.
        if !unsafe { IsWindowVisible(window) }.as_bool() {
            return CARRY_ON;
        }
        let title = window_title(window).to_lowercase();
        let Ok(needle) = NEEDLE.lock() else {
            return CARRY_ON;
        };
        if title.contains(needle.as_str())
            && let Ok(mut found) = FOUND.lock()
            && found.is_none()
        {
            *found = Some(window.0 as isize);
        }
        CARRY_ON
    }

    if let Ok(mut slot) = NEEDLE.lock() {
        *slot = needle.to_lowercase();
    }
    if let Ok(mut slot) = FOUND.lock() {
        *slot = None;
    }
    // SAFETY: `visit` matches the expected callback signature and does not
    // unwind.
    let _ = unsafe { EnumWindows(Some(visit), LPARAM(0)) };

    FOUND
        .lock()
        .ok()
        .and_then(|found| *found)
        .map(|raw| HWND(raw as *mut core::ffi::c_void))
}

/// Every visible window that has a title.
fn visible_windows() -> Vec<String> {
    use std::sync::Mutex;

    use windows::Win32::Foundation::LPARAM;
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible};
    use windows::core::BOOL;

    static TITLES: Mutex<Vec<String>> = Mutex::new(Vec::new());

    unsafe extern "system" fn visit(window: HWND, _: LPARAM) -> BOOL {
        // SAFETY: `window` is supplied by EnumWindows and is live for the call.
        if unsafe { IsWindowVisible(window) }.as_bool() {
            let title = window_title(window);
            if !title.is_empty()
                && let Ok(mut titles) = TITLES.lock()
            {
                titles.push(title);
            }
        }
        BOOL(1)
    }

    if let Ok(mut titles) = TITLES.lock() {
        titles.clear();
    }
    // SAFETY: `visit` matches the callback signature and does not unwind.
    let _ = unsafe { EnumWindows(Some(visit), LPARAM(0)) };
    TITLES
        .lock()
        .map(|titles| titles.clone())
        .unwrap_or_default()
}

fn window_title(window: HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;

    let mut buffer = [0_u16; 512];
    // SAFETY: `buffer` is a valid writable slice of the length passed.
    let length = unsafe { GetWindowTextW(window, &mut buffer) };
    if length <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buffer[..length as usize])
}
