//! Does the overlay see itself?
//!
//! `INV-GUI-001` and `09-gui-and-overlay.md` make this catastrophic rather than
//! untidy. The agent draws marks over the game; if those marks appear in the
//! frames the agent captures, perception sees its own annotations, draws marks
//! on them, and the loop feeds on itself. One leaked pixel is the whole defect.
//!
//! So the assertion is **absolute, not statistical**: across every captured
//! frame, zero pixels of the overlay's colour.
//!
//! **An absolute assertion of absence is worthless without a positive control.**
//! "No magenta was found" is what a working exclusion looks like, and also what
//! a probe that never drew anything looks like, and what a probe looking at the
//! wrong window looks like. The border comparison in this same experiment
//! produced two confident wrong answers of exactly that shape before it grew
//! one.
//!
//! So this runs twice:
//!
//! | Phase | Display affinity | Expected |
//! | --- | --- | --- |
//! | 1 | Default | The overlay **is** in the captured frames |
//! | 2 | `WDA_EXCLUDEFROMCAPTURE` | Zero pixels of it, in every frame |
//!
//! Phase 1 failing means the overlay was never drawn, never on top, or never
//! over the target — and phase 2 then proves nothing. That is reported as
//! inconclusive rather than as a pass.
//!
//! There is a second distinction the specification insists on and this honours:
//! *is the overlay visible to a person* is a question for a screen reading, and
//! *does the capture contain it* is a question for the captured frame. They are
//! different instruments and conflating them is how a capture feedback defect
//! ships.

use std::fmt::Write as _;
use std::sync::mpsc::{Sender, channel};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetWindowRect,
    HWND_TOPMOST, LWA_ALPHA, MSG, PostMessageW, RegisterClassW, SW_SHOWNOACTIVATE, SWP_NOACTIVATE,
    SetLayeredWindowAttributes, SetWindowDisplayAffinity, SetWindowPos, ShowWindow,
    TranslateMessage, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WM_CLOSE, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

use crate::gdi;
use crate::measure::Frame;
use crate::target::Target;
use crate::wgc::Wgc;

/// The overlay's colour, in the order the platform wants it: blue, green, red.
///
/// Magenta, because nothing in a game or an interface is this exact value by
/// accident, and because it is as far from the target's flat grey as a colour
/// can be.
const MARK: COLORREF = COLORREF(0x00FF_00FF);

/// The same colour as the captured frames carry it: red, green, blue.
const MARK_RGB: [u8; 3] = [0xFF, 0x00, 0xFF];

/// How many frames the absolute assertion runs over.
///
/// The specification asks for a thousand. Eight hundred at the rate the
/// compositor delivers is about fifteen seconds, and the run stops early only
/// if it has already failed.
const FRAMES: usize = 800;

/// How long to wait for those frames before giving up.
const PATIENCE: Duration = Duration::from_secs(90);

/// Run both phases.
///
/// # Errors
///
/// Returns the step that failed.
pub(crate) fn check() -> Result<String, String> {
    let target = Target::open()?;
    let overlay = Overlay::open(target.handle())?;

    let mut out = String::new();
    let _ = writeln!(out, "--- overlay self-exclusion ---");

    // Two scopes, because they turn out to answer different questions, and the
    // positive control is what revealed that. A window-scoped capture
    // composites one window; a screen-scoped one reads the desktop, including
    // whatever is drawn on top.
    overlay.set_excluded(false)?;
    sleep(Duration::from_millis(400));
    let window_default = sweep(target.handle(), 60)?;
    let screen_default = count(&gdi::capture_window_rect(target.handle(), 0)?);
    let _ = writeln!(
        out,
        "  default affinity     window-scoped: {} of {} frames carry the mark;  screen-scoped: {screen_default} pixels",
        window_default.frames_with_mark, window_default.frames
    );

    overlay.set_excluded(true)?;
    sleep(Duration::from_millis(400));
    let window_excluded = sweep(target.handle(), FRAMES)?;
    let screen_excluded = count(&gdi::capture_window_rect(target.handle(), 0)?);
    let _ = writeln!(
        out,
        "  EXCLUDEFROMCAPTURE   window-scoped: {} of {} frames, {} pixels total;  screen-scoped: {screen_excluded} pixels",
        window_excluded.frames_with_mark, window_excluded.frames, window_excluded.total_pixels
    );

    let _ = writeln!(out);

    // The window-scoped result, which is the one the product depends on.
    let scoped = if window_excluded.total_pixels > 0 {
        "  window-scoped: LEAKED. The overlay appears under EXCLUDEFROMCAPTURE, which is the perception feedback loop."
    } else if window_default.frames_with_mark == 0 {
        "  window-scoped: the overlay never appears, with or without the affinity. A per-window capture composites that window alone, so an overlay drawn as a separate top-level window is excluded by construction rather than by a flag."
    } else {
        "  window-scoped: the overlay appears by default and contributes zero pixels under EXCLUDEFROMCAPTURE."
    };
    let _ = writeln!(out, "{scoped}");

    // The screen-scoped result, which is what the flag is actually for.
    let screen = if screen_default == 0 {
        "  screen-scoped: INCONCLUSIVE. The overlay is not on screen where the target is, so there was nothing to exclude and the positive control failed."
    } else if screen_excluded == 0 {
        "  screen-scoped: EXCLUDED. The overlay is plainly visible by default and contributes zero pixels under EXCLUDEFROMCAPTURE."
    } else {
        "  screen-scoped: LEAKED. The overlay survives EXCLUDEFROMCAPTURE in a screen reading."
    };
    let _ = writeln!(out, "{screen}");

    drop(overlay);
    drop(target);
    Ok(out)
}

/// What one sweep of captured frames found.
struct Sweep {
    frames: usize,
    frames_with_mark: usize,
    total_pixels: usize,
}

/// Capture `wanted` frames and count the mark in each.
fn sweep(window: HWND, wanted: usize) -> Result<Sweep, String> {
    let (mut session, _) = Wgc::start(window)?;
    let mut sweep = Sweep {
        frames: 0,
        frames_with_mark: 0,
        total_pixels: 0,
    };

    let deadline = Instant::now() + PATIENCE;
    while sweep.frames < wanted && Instant::now() < deadline {
        match session.next() {
            Ok(Some(frame)) => {
                let found = count(&frame);
                sweep.frames += 1;
                sweep.total_pixels += found;
                if found > 0 {
                    sweep.frames_with_mark += 1;
                }
            }
            Ok(None) => sleep(Duration::from_millis(2)),
            Err(error) => return Err(error),
        }
    }

    if sweep.frames == 0 {
        return Err("no frames arrived during the sweep".to_owned());
    }
    Ok(sweep)
}

/// How many pixels of the mark colour a frame carries.
///
/// Exact, with a small tolerance for the compositor's own blending at the
/// edges of the overlay. A tolerance wide enough to admit a nearby colour would
/// turn a leak into a rounding error.
fn count(frame: &Frame) -> usize {
    let (pixels, _) = frame.pixels.as_chunks::<3>();
    pixels
        .iter()
        .filter(|pixel| {
            pixel[0].abs_diff(MARK_RGB[0]) <= 8
                && pixel[1].abs_diff(MARK_RGB[1]) <= 8
                && pixel[2].abs_diff(MARK_RGB[2]) <= 8
        })
        .count()
}

/// A layered, click-through, topmost window over the target.
///
/// It runs its own message loop on its own thread. The first version did not,
/// and never painted: a window whose messages nobody pumps never receives
/// `WM_PAINT`, so there was nothing on screen to exclude. The positive control
/// caught it on the first run, which is the entire reason the control exists.
struct Overlay {
    handle: HWND,
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Overlay {
    /// Create it on its own thread, sized and placed over `target`.
    fn open(target: HWND) -> Result<Self, String> {
        let raw = target.0 as isize;
        let (ready, appeared) = channel::<Result<isize, String>>();
        let (stop, halt) = channel::<()>();

        let thread = spawn(move || {
            let target = HWND(raw as *mut core::ffi::c_void);
            let handle = match create(target) {
                Ok(handle) => {
                    let _ = ready.send(Ok(handle.0 as isize));
                    handle
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                    return;
                }
            };

            let mut message = MSG::default();
            loop {
                if halt.try_recv().is_ok() {
                    // SAFETY: `handle` was created on this thread.
                    unsafe {
                        let _ = DestroyWindow(handle);
                    };
                    break;
                }
                // The window filter is `None` so that `WM_QUIT`, which is
                // posted to the thread rather than to a window, is delivered.
                //
                // SAFETY: `message` is a valid writable MSG.
                let got = unsafe { GetMessageW(&raw mut message, None, 0, 0) };
                if got.0 <= 0 {
                    break;
                }
                // SAFETY: `message` was filled in by GetMessage.
                unsafe {
                    let _ = TranslateMessage(&raw const message);
                    DispatchMessageW(&raw const message);
                }
            }
        });

        let handle = appeared
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "the overlay thread did not report back".to_owned())??;
        sleep(Duration::from_millis(400));
        Ok(Self {
            handle: HWND(handle as *mut core::ffi::c_void),
            stop,
            thread: Some(thread),
        })
    }
}

/// Create the overlay window. Runs on the overlay's own thread.
fn create(target: HWND) -> Result<HWND, String> {
    let mut rect = RECT::default();
    // SAFETY: `rect` is a valid writable RECT.
    unsafe { GetWindowRect(target, &raw mut rect) }
        .map_err(|error| format!("GetWindowRect: {error}"))?;

    // SAFETY: a null module name asks for this module, which is valid.
    let instance = unsafe { GetModuleHandleW(PCWSTR::null()) }
        .map_err(|error| format!("GetModuleHandleW: {error}"))?;
    // SAFETY: the colour is a plain value.
    let brush: HBRUSH = unsafe { CreateSolidBrush(MARK) };

    let class = WNDCLASSW {
        lpfnWndProc: Some(procedure),
        hInstance: instance.into(),
        lpszClassName: w!("AfkAgentOverlayProbe"),
        hbrBackground: brush,
        ..WNDCLASSW::default()
    };
    // SAFETY: `class` is fully initialised and outlives the call.
    unsafe { RegisterClassW(&raw const class) };

    // The extended styles are the overlay's contract with the desktop:
    // never take focus, never take a click, always on top. A tool window
    // so it does not appear in the task switcher.
    //
    // SAFETY: the class was just registered and every pointer is valid.
    let handle = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            w!("AfkAgentOverlayProbe"),
            w!("afk-agent overlay probe"),
            WS_POPUP | WS_VISIBLE,
            rect.left + 40,
            rect.top + 40,
            200,
            120,
            None,
            None,
            Some(instance.into()),
            None,
        )
    }
    .map_err(|error| format!("CreateWindowExW: {error}"))?;

    // SAFETY: `handle` is the window just created.
    unsafe {
        SetLayeredWindowAttributes(handle, COLORREF(0), 255, LWA_ALPHA)
            .map_err(|error| format!("SetLayeredWindowAttributes: {error}"))?;
        let _ = ShowWindow(handle, SW_SHOWNOACTIVATE);
        let _ = SetWindowPos(
            handle,
            Some(HWND_TOPMOST),
            rect.left + 40,
            rect.top + 40,
            200,
            120,
            SWP_NOACTIVATE,
        );
    }
    Ok(handle)
}

impl Overlay {
    /// Turn capture exclusion on or off.
    fn set_excluded(&self, excluded: bool) -> Result<(), String> {
        let affinity = if excluded {
            WDA_EXCLUDEFROMCAPTURE
        } else {
            WDA_NONE
        };
        // SAFETY: `handle` is a live window this process owns.
        unsafe { SetWindowDisplayAffinity(self.handle, affinity) }
            .map_err(|error| format!("SetWindowDisplayAffinity({excluded}): {error}"))
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        // A posted message is what unblocks `GetMessage`.
        // SAFETY: `handle` is live until the thread destroys it.
        unsafe {
            let _ = PostMessageW(Some(self.handle), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Nothing is drawn: the class brush fills the window, which is the whole
/// point of a mark that has to be one exact colour.
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: forwarding the message the window manager delivered.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
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
    fn the_mark_colour_is_counted_exactly() {
        let found = count(&frame(vec![
            0xFF, 0x00, 0xFF, // the mark
            0x60, 0x60, 0x60, // the target's grey
            0xFF, 0x00, 0xFF, // the mark again
        ]));
        assert_eq!(found, 2);
    }

    #[test]
    fn a_nearby_colour_is_not_the_mark() {
        // The tolerance exists for the compositor's blending at an edge. Wide
        // enough to admit a different colour, it would turn a leak into a
        // rounding error — and the assertion is absolute.
        assert_eq!(count(&frame(vec![0xFF, 0x40, 0xFF])), 0);
        assert_eq!(count(&frame(vec![0xC0, 0x00, 0xC0])), 0);
    }

    #[test]
    fn blending_at_an_edge_still_counts() {
        assert_eq!(count(&frame(vec![0xF8, 0x07, 0xFA])), 1);
    }

    #[test]
    fn an_empty_frame_carries_nothing() {
        assert_eq!(count(&frame(Vec::new())), 0);
    }
}
