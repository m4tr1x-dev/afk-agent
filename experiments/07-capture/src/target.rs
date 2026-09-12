//! A window of our own, to capture.
//!
//! The border comparison needs a target that is **visible, still and ours**,
//! and none of the three is negotiable.
//!
//! *Visible*, because a screen reading reads whatever is on screen: a window
//! behind another window gives a perfectly clean reading of the wrong thing,
//! twice, and every difference comes out at zero. The first run of that
//! comparison did exactly this and reported that no border was drawn around a
//! window it had never seen.
//!
//! *Still*, because the null control is "did the interior change". A window
//! that repaints during the comparison makes every reading inconclusive, which
//! is what happened on the second attempt.
//!
//! *Ours*, because the alternative is capturing whatever the maintainer left
//! open, and a measurement that reads somebody's browser is not one to run
//! unattended.
//!
//! Asking the window manager to bring somebody else's window forward does not
//! solve it: `SetForegroundWindow` from a background process is refused by
//! design, which the probe observed rather than assumed.
//!
//! So the probe makes its own: a small topmost window filled with one flat
//! colour, which is also what makes a border unmistakable when one is drawn.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, HBRUSH, InvalidateRect,
    PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    HWND_TOPMOST, MSG, PostMessageW, PostQuitMessage, RegisterClassW, SW_SHOW, SWP_NOMOVE,
    SWP_NOSIZE, SetTimer, SetWindowPos, ShowWindow, TranslateMessage, WM_CLOSE, WM_DESTROY,
    WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

/// A window the probe owns, running its own message loop.
pub(crate) struct Target {
    handle: HWND,
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Target {
    /// The flat colour the window is filled with.
    ///
    /// Mid-grey rather than black or white: a border drawn in either the light
    /// or the dark accent colour differs from it clearly, and a window that
    /// failed to paint at all reads as black rather than as this.
    const FILL: COLORREF = COLORREF(0x0060_6060);

    /// Size in pixels. Small enough to be quick to read, large enough that the
    /// interior dwarfs the edge band the comparison looks at.
    const SIDE: i32 = 640;

    /// Create the window and wait for it to appear.
    ///
    /// # Errors
    ///
    /// Returns the step that failed.
    pub(crate) fn open() -> Result<Self, String> {
        let (ready, appeared) = channel::<Result<isize, String>>();
        let (stop, halt) = channel::<()>();

        // The window lives on its own thread with its own message loop,
        // because a window whose messages nobody pumps is a window the
        // compositor may not treat as live.
        let thread = spawn(move || {
            let handle = match create() {
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
                    // SAFETY: `handle` was created on this thread and is live.
                    unsafe {
                        let _ = DestroyWindow(handle);
                    };
                    break;
                }
                // The window filter is deliberately `None`. `WM_QUIT` is posted
                // to the *thread* rather than to a window, so a loop that
                // filters by window handle never sees it and never returns
                // zero — which is a hang on exit, and was one.
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
            .map_err(|_| "the window thread did not report back".to_owned())??;
        let handle = HWND(handle as *mut core::ffi::c_void);

        // Give the compositor a moment to actually show it. Reading straight
        // away measures a window mid-appearance.
        sleep(Duration::from_millis(500));

        Ok(Self {
            handle,
            stop,
            thread: Some(thread),
        })
    }

    pub(crate) const fn handle(&self) -> HWND {
        self.handle
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        // A posted message is what unblocks `GetMessage`; changing the window
        // position is not, because the compositor may coalesce it away. This
        // was a hang on exit before the message was posted instead.
        //
        // SAFETY: `handle` is live until the thread destroys it.
        unsafe {
            let _ = PostMessageW(Some(self.handle), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Register the class and create the window. Runs on the window's own thread.
fn create() -> Result<HWND, String> {
    // SAFETY: passing null asks for this module's handle, which is valid.
    let instance = unsafe { GetModuleHandleW(PCWSTR::null()) }
        .map_err(|error| format!("GetModuleHandleW: {error}"))?;

    // SAFETY: the colour is a plain value; the brush is owned by the class for
    // the life of the process, which is why it is not deleted.
    let brush: HBRUSH = unsafe { CreateSolidBrush(Target::FILL) };

    let class = WNDCLASSW {
        lpfnWndProc: Some(procedure),
        hInstance: instance.into(),
        lpszClassName: w!("AfkAgentCaptureTarget"),
        hbrBackground: brush,
        ..WNDCLASSW::default()
    };
    // SAFETY: `class` is fully initialised and outlives the call. Registering
    // the same class twice returns zero, which is not fatal here because the
    // probe creates one window per run.
    unsafe { RegisterClassW(&raw const class) };

    // SAFETY: the class was just registered and every pointer is valid.
    let handle = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST,
            w!("AfkAgentCaptureTarget"),
            w!("afk-agent capture target"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            Target::SIDE,
            Target::SIDE,
            None,
            None,
            Some(instance.into()),
            None,
        )
    }
    .map_err(|error| format!("CreateWindowExW: {error}"))?;

    // SAFETY: `handle` is the window just created.
    unsafe {
        // Roughly sixty times a second, so the compositor has something to
        // deliver. Without it a sweep of eight hundred frames never finishes.
        SetTimer(Some(handle), 1, 16, None);
        let _ = ShowWindow(handle, SW_SHOW);
        let _ = SetWindowPos(
            handle,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        );
    }
    Ok(handle)
}

/// Which of the two near-identical greys the next paint uses.
///
/// The window has to *change*, or the compositor delivers almost nothing: it
/// publishes on change rather than on a clock, and a still window yields one
/// frame in eight seconds. A sweep that needs eight hundred frames needs the
/// target to be alive.
///
/// Two greys one unit apart, because the change has to be real to the
/// compositor without being large enough to matter to anything measuring
/// colours in the frame.
static PHASE: AtomicBool = AtomicBool::new(false);

/// The window procedure.
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_DESTROY => {
            // SAFETY: called from the window's own thread.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        WM_TIMER => {
            // SAFETY: `window` is live and the rectangle argument is optional.
            unsafe {
                let _ = InvalidateRect(Some(window), None, true);
            };
            LRESULT(0)
        }
        WM_PAINT => {
            let phase = PHASE.fetch_xor(true, Ordering::Relaxed);
            let shade = if phase { 0x60 } else { 0x61 };
            let mut paint = PAINTSTRUCT::default();
            // SAFETY: `paint` is a valid writable structure and EndPaint is
            // called on every path below.
            unsafe {
                let dc = BeginPaint(window, &raw mut paint);
                let brush = CreateSolidBrush(COLORREF(shade << 16 | shade << 8 | shade));
                FillRect(dc, &raw const paint.rcPaint, brush);
                let _ = DeleteObject(brush.into());
                let _ = EndPaint(window, &raw const paint);
            }
            LRESULT(0)
        }
        // SAFETY: forwarding the message the window manager delivered.
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}
