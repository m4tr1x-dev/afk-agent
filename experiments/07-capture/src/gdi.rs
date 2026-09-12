//! The two routes that need nothing but the window manager.
//!
//! `PrintWindow` asks the window to render itself into a bitmap; `BitBlt`
//! copies whatever is currently on screen where the window is. They are the
//! oldest capture mechanisms Windows has, they work without a graphics device,
//! and they are the baseline every newer route has to beat.
//!
//! They are also the two most likely to fail interestingly. `PrintWindow`
//! returns black for a window that renders through a swap chain rather than
//! through the window manager, which is every game; `BitBlt` returns whatever
//! is in front, including another window. Both failures are silent — the call
//! succeeds and the bitmap is wrong — which is exactly the class of signature
//! `FR-PERC-009` needs written down.

use std::time::Instant;

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, ClientToScreen, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC,
    ReleaseDC, SRCCOPY, SelectObject, StretchBlt,
};
// PrintWindow lives under the printing interfaces rather than beside the other
// window calls, which is a historical accident and the reason this import looks
// wrong at first glance.
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetClientRect, GetDesktopWindow, GetWindowRect, PW_RENDERFULLCONTENT,
    SW_RESTORE, SetForegroundWindow, ShowWindow, WindowFromPoint,
};

use crate::measure::Frame;

/// Which of the window-manager routes to use.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gdi {
    /// Ask the window to render itself, including content drawn by a compositor.
    PrintWindow,
    /// Copy the window's own device context.
    BitBlt,
    /// Copy the screen region the window occupies.
    ///
    /// The control route. It reads what a person would see, so a frame that is
    /// correct here and wrong elsewhere separates "the window is not rendering"
    /// from "this route cannot see it".
    ScreenCrop,
}

impl Gdi {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::PrintWindow => "PrintWindow",
            Self::BitBlt => "BitBlt",
            Self::ScreenCrop => "ScreenCrop",
        }
    }
}

/// Capture one frame.
///
/// # Errors
///
/// Returns a message naming the step that failed, rather than a code. A route
/// comparison is read by a person deciding which route to ship.
pub(crate) fn capture(window: HWND, route: Gdi) -> Result<Frame, String> {
    let started = Instant::now();
    let (width, height) = client_size(window)?;
    if width <= 0 || height <= 0 {
        return Err(format!("client area is {width}x{height}"));
    }

    // SAFETY: `window` is a window handle the caller obtained from the window
    // manager and has not closed. Every handle created below is released on
    // every path, including the error paths, by the guards at the end.
    let pixels = unsafe { grab(window, width, height, route) }?;

    Ok(Frame {
        pixels,
        width: width as usize,
        height: height as usize,
        elapsed: started.elapsed(),
    })
}

/// Whether the window's own centre is actually the topmost thing there.
///
/// A screen reading reads whatever is on screen, which is not necessarily the
/// target: a window behind another window yields a perfectly clean reading of
/// the wrong thing, twice, and every difference comes out at zero. The first
/// run of the border comparison did exactly that and reported "no border was
/// drawn" about a window it had never seen.
///
/// One call and no pixels: ask the window manager what is at the centre point,
/// walk up to its root, and compare.
#[must_use]
pub(crate) fn is_unobscured(window: HWND) -> bool {
    let mut rect = RECT::default();
    // SAFETY: `rect` is a valid writable RECT.
    if unsafe { GetWindowRect(window, &raw mut rect) }.is_err() {
        return false;
    }
    let centre = POINT {
        x: i32::midpoint(rect.left, rect.right),
        y: i32::midpoint(rect.top, rect.bottom),
    };
    // SAFETY: a point is passed by value and the returned handle is only
    // compared, never dereferenced.
    let at_point = unsafe { WindowFromPoint(centre) };
    if at_point.is_invalid() {
        return false;
    }
    // SAFETY: `at_point` is a live window handle from the window manager.
    let root = unsafe { GetAncestor(at_point, GA_ROOT) };
    root == window
}

/// Bring a window to the front.
///
/// Opt-in, because a probe that rearranges the desktop without being asked is
/// a probe nobody runs twice. This is a window-manager call rather than input
/// synthesis — `FR-ACT-008` is untouched and this file contains no input path.
pub(crate) fn bring_to_front(window: HWND) {
    // SAFETY: `window` is a live window handle; both calls only reorder.
    unsafe {
        let _ = ShowWindow(window, SW_RESTORE);
        let _ = SetForegroundWindow(window);
    }
}

/// Capture the screen where the window is, including its frame and a margin.
///
/// The client-area routes above cannot see the capture border: it is drawn
/// around the window rather than inside its client area, and it may not be
/// drawn into a captured frame at all. Reading the screen is the only
/// instrument that sees what a person sees.
///
/// # Errors
///
/// Returns the step that failed.
pub(crate) fn capture_window_rect(window: HWND, margin: i32) -> Result<Frame, String> {
    let started = Instant::now();
    let mut rect = RECT::default();
    // SAFETY: `rect` is a valid writable RECT.
    unsafe { GetWindowRect(window, &raw mut rect) }
        .map_err(|error| format!("GetWindowRect: {error}"))?;

    let left = rect.left - margin;
    let top = rect.top - margin;
    let width = (rect.right - rect.left) + margin * 2;
    let height = (rect.bottom - rect.top) + margin * 2;
    if width <= 0 || height <= 0 {
        return Err(format!("window rectangle is {width}x{height}"));
    }

    // SAFETY: the desktop device context is released on every path below.
    let source = unsafe { GetDC(Some(GetDesktopWindow())) };
    if source.is_invalid() {
        return Err("GetDC on the desktop returned nothing".to_owned());
    }
    // SAFETY: `source` is live for the duration of the call.
    let pixels = unsafe { crop(source, left, top, width, height) };
    // SAFETY: `source` came from GetDC on the desktop and has not been released.
    unsafe { ReleaseDC(Some(GetDesktopWindow()), source) };

    Ok(Frame {
        pixels: pixels?,
        width: width as usize,
        height: height as usize,
        elapsed: started.elapsed(),
    })
}

/// # Safety
///
/// `source` must be a live device context for the desktop.
unsafe fn crop(
    source: HDC,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Result<Vec<u8>, String> {
    // SAFETY: `source` is live.
    let memory = unsafe { CreateCompatibleDC(Some(source)) };
    if memory.is_invalid() {
        return Err("CreateCompatibleDC failed".to_owned());
    }
    // SAFETY: as above.
    let bitmap = unsafe { CreateCompatibleBitmap(source, width, height) };
    if bitmap.is_invalid() {
        // SAFETY: `memory` is live and owned here.
        unsafe {
            let _ = DeleteDC(memory);
        };
        return Err("CreateCompatibleBitmap failed".to_owned());
    }
    // SAFETY: both handles are live; the selection is restored below.
    let previous = unsafe { SelectObject(memory, bitmap.into()) };

    // SAFETY: both device contexts are live and the destination bitmap matches.
    let ok = unsafe {
        StretchBlt(
            memory,
            0,
            0,
            width,
            height,
            Some(source),
            left,
            top,
            width,
            height,
            SRCCOPY,
        )
    };
    let result = if ok.as_bool() {
        // SAFETY: `bitmap` is live and matches the dimensions passed.
        unsafe { read_pixels(memory, bitmap, width, height) }
    } else {
        Err("screen crop refused".to_owned())
    };

    // SAFETY: restoring the selection before deleting, and both handles were
    // created here.
    unsafe {
        SelectObject(memory, previous);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory);
    }
    result
}

fn client_size(window: HWND) -> Result<(i32, i32), String> {
    let mut rect = RECT::default();
    // SAFETY: `rect` is a valid writable RECT for the duration of the call.
    unsafe { GetClientRect(window, &raw mut rect) }
        .map_err(|error| format!("GetClientRect: {error}"))?;
    Ok((rect.right - rect.left, rect.bottom - rect.top))
}

/// # Safety
///
/// `window` must be a live window handle.
unsafe fn grab(window: HWND, width: i32, height: i32, route: Gdi) -> Result<Vec<u8>, String> {
    // SAFETY: the desktop window handle is always valid; GetDC returns null on
    // failure, which is checked.
    let source = unsafe {
        if route == Gdi::ScreenCrop {
            GetDC(Some(GetDesktopWindow()))
        } else {
            GetDC(Some(window))
        }
    };
    if source.is_invalid() {
        return Err("GetDC returned nothing".to_owned());
    }

    // SAFETY: `source` is the device context obtained just above and is
    // released below on every path.
    let result = unsafe { grab_into(window, source, width, height, route) };

    // SAFETY: `source` came from GetDC on the same window and has not been
    // released. Releasing it on the error path as well is why this is not a `?`.
    unsafe {
        ReleaseDC(
            Some(if route == Gdi::ScreenCrop {
                GetDesktopWindow()
            } else {
                window
            }),
            source,
        )
    };
    result
}

/// # Safety
///
/// `source` must be a device context obtained for `window` or the desktop.
unsafe fn grab_into(
    window: HWND,
    source: HDC,
    width: i32,
    height: i32,
    route: Gdi,
) -> Result<Vec<u8>, String> {
    // SAFETY: `source` is a live device context.
    let memory = unsafe { CreateCompatibleDC(Some(source)) };
    if memory.is_invalid() {
        return Err("CreateCompatibleDC failed".to_owned());
    }
    // SAFETY: as above.
    let bitmap = unsafe { CreateCompatibleBitmap(source, width, height) };
    if bitmap.is_invalid() {
        // SAFETY: `memory` is live and owned here.
        unsafe {
            let _ = DeleteDC(memory);
        };
        return Err("CreateCompatibleBitmap failed".to_owned());
    }

    // SAFETY: both handles are live; the previous selection is restored below.
    let previous = unsafe { SelectObject(memory, bitmap.into()) };
    // SAFETY: every handle is live and `bitmap` is selected into `memory`,
    // which is what `render` requires.
    let result = unsafe { render(window, source, memory, bitmap, width, height, route) };

    // SAFETY: restoring the original selection before deleting is required, and
    // both deletions are of handles created here.
    unsafe {
        SelectObject(memory, previous);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(memory);
    }
    result
}

/// # Safety
///
/// All handles must be live and `bitmap` selected into `memory`.
#[allow(clippy::too_many_arguments)]
unsafe fn render(
    window: HWND,
    source: HDC,
    memory: HDC,
    bitmap: HBITMAP,
    width: i32,
    height: i32,
    route: Gdi,
) -> Result<Vec<u8>, String> {
    match route {
        Gdi::PrintWindow => {
            // PW_RENDERFULLCONTENT is what makes this work for a window drawn
            // by the compositor rather than by the window manager. Without it,
            // the result is reliably black and reliably misleading.
            //
            // SAFETY: both handles are live.
            let ok =
                unsafe { PrintWindow(window, memory, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)) };
            if !ok.as_bool() {
                return Err("PrintWindow refused".to_owned());
            }
        }
        Gdi::BitBlt => {
            // SAFETY: both device contexts are live and the rectangle is inside
            // the bitmap created for it.
            let ok = unsafe {
                StretchBlt(
                    memory,
                    0,
                    0,
                    width,
                    height,
                    Some(source),
                    0,
                    0,
                    width,
                    height,
                    SRCCOPY,
                )
            };
            if !ok.as_bool() {
                return Err("BitBlt refused".to_owned());
            }
        }
        Gdi::ScreenCrop => {
            let mut origin = windows::Win32::Foundation::POINT::default();
            // SAFETY: `origin` is a valid writable POINT.
            if !unsafe { ClientToScreen(window, &raw mut origin) }.as_bool() {
                return Err("ClientToScreen refused".to_owned());
            }
            // SAFETY: as above; the source rectangle is on the desktop context.
            let ok = unsafe {
                StretchBlt(
                    memory,
                    0,
                    0,
                    width,
                    height,
                    Some(source),
                    origin.x,
                    origin.y,
                    width,
                    height,
                    SRCCOPY,
                )
            };
            if !ok.as_bool() {
                return Err("screen BitBlt refused".to_owned());
            }
        }
    }

    // SAFETY: `bitmap` is live and its dimensions match the header below.
    unsafe { read_pixels(memory, bitmap, width, height) }
}

/// # Safety
///
/// `bitmap` must be a live bitmap of exactly `width` by `height`.
unsafe fn read_pixels(
    memory: HDC,
    bitmap: HBITMAP,
    width: i32,
    height: i32,
) -> Result<Vec<u8>, String> {
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>())
                .map_err(|_| "header size does not fit".to_owned())?,
            biWidth: width,
            // Negative height asks for a top-down buffer. Positive gives
            // bottom-up, and every consumer here assumes the first row is the
            // top one.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    };

    let count = (width as usize) * (height as usize);
    let mut raw = vec![0_u8; count * 4];
    // SAFETY: `raw` is large enough for `height` rows of `width` 32-bit pixels,
    // which is what the header above declares.
    let copied = unsafe {
        GetDIBits(
            memory,
            bitmap,
            0,
            u32::try_from(height).map_err(|_| "height does not fit".to_owned())?,
            Some(raw.as_mut_ptr().cast()),
            &raw mut info,
            DIB_RGB_COLORS,
        )
    };
    if copied == 0 {
        return Err("GetDIBits copied nothing".to_owned());
    }

    // Blue, green, red, unused — to red, green, blue.
    let mut pixels = Vec::with_capacity(count * 3);
    let (quads, _) = raw.as_chunks::<4>();
    for chunk in quads {
        pixels.push(chunk[2]);
        pixels.push(chunk[1]);
        pixels.push(chunk[0]);
    }
    Ok(pixels)
}
