//! The corpus recorder: ground truth from a person's own clicks.
//!
//! `20-evaluation-harness.md` is blunt about why this exists: *"a benchmark
//! labelled by the system under test measures agreement rather than accuracy."*
//! If the perception pipeline proposes the boxes and the model describes the
//! targets, the grounding benchmark measures the system agreeing with itself.
//!
//! The way out is a person. When somebody clicks at a point and the screen
//! changes within 200 milliseconds, that point is **by definition** inside a
//! real interactive element — with no model, no detector, and no circle. The
//! frame immediately before the click is the input; the point is the label; the
//! change is the evidence that the element was interactive at all.
//!
//! **This is not a new capability.** `FR-SAFE-002` already requires observing
//! the user's real input and telling it from synthesised input, and `ADR-0025`
//! excludes hooking the *game* and injecting into the *game's* process.
//! Watching our own process's raw input stream is neither.
//!
//! **Nothing is synthesised here**, and `tools/check_call_sites.py` rule A
//! enforces that: the synthesis spellings may appear only in `afk-input` and
//! `afk-guardian`, and it scans this directory. Raw input is observation, which
//! is a different surface and deliberately not on that list.
//!
//! ## What it writes
//!
//! ```text
//! frame-0000.png           the frame immediately before the click
//! frame-0000.labels.json   the click point, the timestamp, and whether the
//!                          screen changed within the window
//! ```
//!
//! Frames land outside the repository by default. Some roster titles are
//! commercial, the repository is public, and `20-evaluation-harness.md` leaves
//! publication as an open question — so the recorder does not decide it.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::{
    GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT,
    RIDEV_INPUTSINK, RegisterRawInputDevices,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, HWND_MESSAGE, MSG, PM_REMOVE,
    PeekMessageW, RI_MOUSE_LEFT_BUTTON_DOWN, RegisterClassW, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_INPUT, WNDCLASSW,
};
use windows::core::{PCWSTR, w};

use crate::measure::Frame;
use crate::wgc::Wgc;

/// How long after a click to look for a change.
///
/// The harness page's number. Long enough for a menu to respond, short enough
/// that an unrelated animation is unlikely to be mistaken for a response.
const RESPONSE: Duration = Duration::from_millis(200);

/// How many frames to keep, so the one *before* a click is available.
///
/// A click is noticed after it happens, so the useful frame is already in the
/// past. Eight frames at the rate the compositor delivers is a comfortable
/// tenth of a second.
const HISTORY: usize = 8;

/// How much of the frame must differ to count as a response.
///
/// A share rather than a count, because frames differ in size between titles.
/// Two tenths of one percent is about 1500 pixels of a 720p frame: smaller than
/// a button, larger than a cursor.
const RESPONDED: f64 = 0.2;

/// Record until the click budget is reached.
///
/// # Errors
///
/// Returns the step that failed.
pub(crate) fn record(
    window: HWND,
    directory: &str,
    wanted: usize,
    minutes: u64,
) -> Result<String, String> {
    std::fs::create_dir_all(directory).map_err(|error| format!("{directory}: {error}"))?;
    let sink = MessageSink::open()?;

    let (mut session, _) = Wgc::start(window)?;
    let mut history: VecDeque<Frame> = VecDeque::with_capacity(HISTORY);

    let mut out = String::new();
    let _ = writeln!(out, "--- corpus recorder ---");
    let _ = writeln!(out, "  writing to   {directory}");
    let _ = writeln!(out, "  stopping at  {wanted} clicks or {minutes} minutes");
    let _ = writeln!(
        out,
        "  play normally; every click that changes the screen is a label"
    );
    let _ = writeln!(out);

    let deadline = Instant::now() + Duration::from_secs(minutes * 60);
    let mut kept = 0;
    let mut discarded = 0;

    while kept < wanted && Instant::now() < deadline {
        match session.next() {
            Ok(Some(frame)) => {
                if history.len() == HISTORY {
                    history.pop_front();
                }
                history.push_back(frame);
            }
            Ok(None) => sleep(Duration::from_millis(2)),
            Err(error) => return Err(error),
        }

        let Some(click) = sink.take_click() else {
            continue;
        };

        // The frame before the click, not the one after it.
        let Some(before) = history.back() else {
            continue;
        };

        let mut point = POINT {
            x: click.0,
            y: click.1,
        };
        // SAFETY: `point` is a valid writable POINT and `window` is live.
        let inside = unsafe { ScreenToClient(window, &raw mut point) }.as_bool();
        if !inside
            || point.x < 0
            || point.y < 0
            || point.x as usize >= before.width
            || point.y as usize >= before.height
        {
            discarded += 1;
            continue;
        }

        let before = clone_frame(before);
        let responded = wait_for_change(&mut session, &before)?;

        if responded {
            write_pair(directory, kept, &before, point, responded)?;
            kept += 1;
            if kept % 10 == 0 {
                let _ = writeln!(out, "  {kept} labels, {discarded} clicks discarded");
            }
        } else {
            // A click the screen ignored. Counted rather than dropped
            // silently: the ratio is itself a finding about how much of a
            // session is spent clicking on nothing.
            discarded += 1;
        }
    }

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {kept} labels written, {discarded} clicks discarded as no-ops or outside the window"
    );
    if kept == 0 {
        let _ = writeln!(
            out,
            "  NOTHING RECORDED: no click produced a screen change inside the target window. \
             Check that the game is the window named, and that it is the one being clicked."
        );
    }
    Ok(out)
}

/// Watch for a change within the response window.
fn wait_for_change(session: &mut Wgc, before: &Frame) -> Result<bool, String> {
    let deadline = Instant::now() + RESPONSE;
    while Instant::now() < deadline {
        match session.next() {
            Ok(Some(frame)) => {
                if changed(before, &frame) >= RESPONDED {
                    return Ok(true);
                }
            }
            Ok(None) => sleep(Duration::from_millis(2)),
            Err(error) => return Err(error),
        }
    }
    Ok(false)
}

/// The share of pixels that differ, as a percentage.
fn changed(left: &Frame, right: &Frame) -> f64 {
    if left.pixels.len() != right.pixels.len() || left.pixels.is_empty() {
        return 100.0;
    }
    let (left_pixels, _) = left.pixels.as_chunks::<3>();
    let (right_pixels, _) = right.pixels.as_chunks::<3>();
    let differing = left_pixels
        .iter()
        .zip(right_pixels)
        .filter(|(a, b)| {
            a[0].abs_diff(b[0]) > 12 || a[1].abs_diff(b[1]) > 12 || a[2].abs_diff(b[2]) > 12
        })
        .count();
    differing as f64 * 300.0 / left.pixels.len() as f64
}

fn clone_frame(frame: &Frame) -> Frame {
    Frame {
        pixels: frame.pixels.clone(),
        width: frame.width,
        height: frame.height,
        elapsed: frame.elapsed,
    }
}

/// Write one frame and its label.
fn write_pair(
    directory: &str,
    index: usize,
    frame: &Frame,
    point: POINT,
    responded: bool,
) -> Result<(), String> {
    let stem = format!("{directory}/frame-{index:04}");
    crate::write_png(&format!("{stem}.png"), frame)?;

    let at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());

    // Hand-written rather than serialised. The shape is four fields and a
    // schema version, and this crate is a time-boxed probe.
    let label = format!(
        "{{\n  \"schema\": 1,\n  \"click\": {{ \"x\": {}, \"y\": {} }},\n  \
         \"frame\": {{ \"width\": {}, \"height\": {} }},\n  \
         \"responded_within_ms\": {},\n  \"responded\": {},\n  \"at_unix_ms\": {}\n}}\n",
        point.x,
        point.y,
        frame.width,
        frame.height,
        RESPONSE.as_millis(),
        responded,
        at
    );
    std::fs::write(format!("{stem}.labels.json"), label)
        .map_err(|error| format!("{stem}.labels.json: {error}"))
}

/// A message-only window that receives the user's own raw mouse input.
struct MessageSink {
    handle: HWND,
}

/// Where the window procedure leaves a click for the loop to collect.
///
/// A static because a window procedure has no other channel, and the recorder
/// owns exactly one sink.
static PENDING: std::sync::Mutex<Option<(i32, i32)>> = std::sync::Mutex::new(None);

impl MessageSink {
    fn open() -> Result<Self, String> {
        // SAFETY: a null module name asks for this module, which is valid.
        let instance = unsafe { GetModuleHandleW(PCWSTR::null()) }
            .map_err(|error| format!("GetModuleHandleW: {error}"))?;

        let class = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: w!("AfkAgentRecorderSink"),
            ..WNDCLASSW::default()
        };
        // SAFETY: `class` is fully initialised and outlives the call.
        unsafe { RegisterClassW(&raw const class) };

        // A message-only window: no pixels, no presence on screen, and it
        // still receives WM_INPUT.
        //
        // SAFETY: the class was just registered and every pointer is valid.
        let handle = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("AfkAgentRecorderSink"),
                w!("afk-agent recorder"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(instance.into()),
                None,
            )
        }
        .map_err(|error| format!("CreateWindowExW: {error}"))?;

        // RIDEV_INPUTSINK: deliver input even when this window is not in the
        // foreground, which it never will be — the game is.
        let device = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x02,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: handle,
        };
        // SAFETY: the slice is one valid descriptor and the size matches.
        unsafe {
            RegisterRawInputDevices(
                &[device],
                u32::try_from(size_of::<RAWINPUTDEVICE>()).unwrap_or(24),
            )
        }
        .map_err(|error| format!("RegisterRawInputDevices: {error}"))?;

        Ok(Self { handle })
    }

    /// Pump messages and return a click if one arrived.
    fn take_click(&self) -> Option<(i32, i32)> {
        let mut message = MSG::default();
        // SAFETY: `message` is a valid writable MSG; PeekMessage does not block.
        while unsafe { PeekMessageW(&raw mut message, Some(self.handle), 0, 0, PM_REMOVE) }
            .as_bool()
        {
            // SAFETY: `message` was filled in by PeekMessage.
            unsafe {
                let _ = TranslateMessage(&raw const message);
                DispatchMessageW(&raw const message);
            }
        }
        PENDING.lock().ok().and_then(|mut slot| slot.take())
    }
}

/// Collect a left-button press and record where the pointer was.
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    if message == WM_INPUT {
        let mut raw = RAWINPUT::default();
        let mut size = u32::try_from(size_of::<RAWINPUT>()).unwrap_or(48);
        let header = u32::try_from(size_of::<RAWINPUTHEADER>()).unwrap_or(24);
        // SAFETY: `raw` is a valid writable RAWINPUT and `size` its length.
        let copied = unsafe {
            GetRawInputData(
                HRAWINPUT(lparam.0 as *mut core::ffi::c_void),
                RID_INPUT,
                Some((&raw mut raw).cast()),
                &raw mut size,
                header,
            )
        };
        if copied != u32::MAX {
            // SAFETY: the header says this is a mouse record, so the mouse
            // arm of the union is the live one.
            let flags = unsafe { raw.data.mouse.Anonymous.Anonymous.usButtonFlags };
            if u32::from(flags) & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
                let mut point = POINT::default();
                // SAFETY: `point` is a valid writable POINT.
                if unsafe { GetCursorPos(&raw mut point) }.is_ok()
                    && let Ok(mut slot) = PENDING.lock()
                {
                    *slot = Some((point.x, point.y));
                }
            }
        }
    }
    // SAFETY: forwarding the message the window manager delivered.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// Where the recorder writes by default.
///
/// Outside the repository. Some roster titles are commercial, the repository is
/// public, and whether the corpus can be published is an open question on
/// `20-evaluation-harness.md` — so the recorder does not decide it.
pub(crate) const DEFAULT_DIRECTORY: &str = "E:/afk-agent/corpus";

/// Confirm a directory is not inside the repository.
///
/// # Errors
///
/// Returns a message naming the problem.
pub(crate) fn refuse_repository_path(directory: &str) -> Result<(), String> {
    let here = std::env::current_dir().unwrap_or_default();

    // Two comparisons, because neither alone is enough, and this guard has been
    // wrong twice.
    //
    // A relative path has to be joined to the working directory first: a
    // destination that does not exist yet cannot be canonicalised, and a
    // relative path never starts with an absolute one. `--record ./corpus`
    // passed the guard on that alone, and wrote into the repository.
    //
    // Canonicalising both is the other half, for a path reached by a symbolic
    // link or containing `..` — but on Windows it returns the verbatim form,
    // which never starts with a non-verbatim one. So the raw comparison and the
    // canonical comparison are both made, and either one is enough to refuse.
    let target = if Path::new(directory).is_absolute() {
        Path::new(directory).to_path_buf()
    } else {
        here.join(directory)
    };

    let inside_raw = target.starts_with(&here);
    let inside_canonical = match (target.canonicalize(), here.canonicalize()) {
        (Ok(target), Ok(here)) => target.starts_with(here),
        _ => false,
    };

    if inside_raw || inside_canonical {
        return Err(format!(
            "{directory} is inside the repository. Frames of commercial titles must not be \
             committed to a public repository, and whether the corpus can be published at all \
             is an open question. Write somewhere else, such as {DEFAULT_DIRECTORY}."
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(pixels: Vec<u8>) -> Frame {
        Frame {
            width: pixels.len() / 3,
            height: 1,
            pixels,
            elapsed: Duration::ZERO,
        }
    }

    #[test]
    fn an_unchanged_frame_scores_zero() {
        let a = frame(vec![10, 20, 30, 40, 50, 60]);
        let b = frame(vec![10, 20, 30, 40, 50, 60]);
        assert!(changed(&a, &b) < 1e-9);
    }

    #[test]
    fn a_fully_changed_frame_scores_a_hundred() {
        let a = frame(vec![0, 0, 0, 0, 0, 0]);
        let b = frame(vec![255, 255, 255, 255, 255, 255]);
        assert!((changed(&a, &b) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn small_noise_is_not_a_response() {
        // Compression and dithering move a pixel by a few units between frames
        // of a still scene. Counting that as a response would label every
        // click as a hit, which is the failure that would make the corpus
        // useless while looking like a success.
        let a = frame(vec![100, 100, 100, 100, 100, 100]);
        let b = frame(vec![108, 92, 100, 100, 111, 100]);
        assert!(changed(&a, &b) < 1e-9);
    }

    #[test]
    fn frames_of_different_sizes_count_as_wholly_changed() {
        // A resize mid-click. Reporting it as a response is wrong, but so is
        // silently comparing mismatched buffers, and the caller discards it.
        let a = frame(vec![0; 6]);
        let b = frame(vec![0; 9]);
        assert!((changed(&a, &b) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn a_path_inside_the_working_tree_is_refused() {
        // Frames of commercial titles must not reach a public repository, and
        // whether the corpus can be published at all is still open. The guard
        // compares against the working directory, so the test asks about that
        // rather than assuming where the test runner starts.
        let here = std::env::current_dir().expect("a working directory");
        assert!(refuse_repository_path(&here.display().to_string()).is_err());
        assert!(refuse_repository_path(&here.join("src").display().to_string()).is_err());
    }

    #[test]
    fn a_relative_path_that_does_not_exist_yet_is_still_refused() {
        // `--record ./corpus` passed the guard before this: the directory does
        // not exist, so canonicalising failed, and a relative path never starts
        // with an absolute one. It would have written into the repository.
        assert!(refuse_repository_path("./corpus").is_err());
        assert!(refuse_repository_path("corpus/frames").is_err());
    }

    #[test]
    fn a_path_outside_it_is_allowed() {
        let outside = std::env::temp_dir().join("afk-agent-corpus-test");
        assert!(refuse_repository_path(&outside.display().to_string()).is_ok());
    }
}
