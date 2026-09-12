//! Grab a window's pixels, by whichever route works.
//!
//! Prototype, and deliberately so. `ADR-0008` — the capture interface — is
//! unwritten and blocked on "a prototype against several window modes", which
//! is this. What it learns about which route works for which kind of window is
//! the evidence that record needs.
//!
//! Three routes:
//!
//! 1. **`PrintWindow` with full content.** Asks the window to render itself
//!    into a bitmap. Works for composited windows whose content the desktop
//!    manager already holds, which covers most modern games in borderless
//!    windowed mode.
//! 2. **`BitBlt` from the window's device context.** The older route. It copies
//!    what is on screen, so it fails when the window is occluded, and it
//!    commonly returns black for content drawn by the graphics device rather
//!    than by the drawing interface.
//! 3. **`ScreenCrop`.** Reads the display itself, cropped to where the window
//!    sits. It is the control rather than a candidate: it sees exactly what a
//!    person sees, so a turn that is visible on screen is visible to it, which
//!    is what separates an input that never arrived from a capture that does
//!    not reflect the rendered scene. It also captures anything overlapping the
//!    window, which is why the product cannot use it.
//!
//! Neither is what the product ships. The specification requires window-scoped
//! capture through the platform's dedicated interface, with no full-resolution
//! frame read back to main memory per frame — this reads back every frame,
//! which is fine for a probe taking three of them and would be a defect in the
//! perception pipeline.
//!
//! The route that worked is reported, because "capture succeeded" without
//! saying how would throw away the finding this prototype exists to produce.

use core::ffi::c_void;

/// A window handle.
pub(crate) type Hwnd = *mut c_void;

type Hdc = *mut c_void;
type HBitmap = *mut c_void;
type HGdiObj = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colours_used: u32,
    colours_important: u32,
}

#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colours: [u32; 3],
}

const SRCCOPY: u32 = 0x00CC_0020;
const DIB_RGB_COLORS: u32 = 0;
const BI_RGB: u32 = 0;
/// Render the whole window, including content the graphics device drew.
const PW_RENDERFULLCONTENT: u32 = 0x0000_0002;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetClientRect(window: Hwnd, rect: *mut Rect) -> i32;
    fn GetDC(window: Hwnd) -> Hdc;
    fn ReleaseDC(window: Hwnd, dc: Hdc) -> i32;
    fn PrintWindow(window: Hwnd, dc: Hdc, flags: u32) -> i32;
    fn IsWindow(window: Hwnd) -> i32;
    fn FindWindowA(class: *const u8, title: *const u8) -> Hwnd;
    fn GetWindowTextA(window: Hwnd, text: *mut u8, count: i32) -> i32;
    fn EnumWindows(callback: extern "system" fn(Hwnd, isize) -> i32, param: isize) -> i32;
    fn IsWindowVisible(window: Hwnd) -> i32;
    fn GetWindowRect(window: Hwnd, rect: *mut Rect) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(dc: Hdc) -> Hdc;
    fn CreateCompatibleBitmap(dc: Hdc, width: i32, height: i32) -> HBitmap;
    fn SelectObject(dc: Hdc, object: HGdiObj) -> HGdiObj;
    fn BitBlt(
        dest: Hdc,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        source: Hdc,
        source_x: i32,
        source_y: i32,
        rop: u32,
    ) -> i32;
    fn GetDIBits(
        dc: Hdc,
        bitmap: HBitmap,
        start: u32,
        lines: u32,
        bits: *mut c_void,
        info: *mut BitmapInfo,
        usage: u32,
    ) -> i32;
    fn DeleteObject(object: HGdiObj) -> i32;
    fn DeleteDC(dc: Hdc) -> i32;
}

/// Which route produced the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Route {
    /// The window rendered itself, including graphics-device content.
    PrintWindow,
    /// Copied from the window's own device context.
    BitBlt,
    /// Copied from the screen, cropped to where the window sits.
    ///
    /// The control that separates "the input never arrived" from "the capture
    /// does not reflect the rendered scene". It reads what the display actually
    /// shows, so a turn that is visible to a person is visible to it — at the
    /// cost of capturing whatever occludes the window, which is why it is a
    /// diagnostic rather than something the product could use.
    ScreenCrop,
}

impl core::fmt::Display for Route {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::PrintWindow => "PrintWindow",
            Self::BitBlt => "BitBlt",
            Self::ScreenCrop => "ScreenCrop",
        })
    }
}

/// One captured frame, already reduced to grayscale.
pub(crate) struct Frame {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) gray: Vec<u8>,
    pub(crate) route: Route,
}

/// Why a capture produced nothing usable.
///
/// `FR-PERC-009` requires an unsupported window mode and a transient failure to
/// be distinguishable, "because reporting 'capture failed' for both is the
/// difference between a five-second fix and an hour of confusion". The
/// distinction starts here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CaptureError {
    /// The handle is not a window. It closed, or it never existed.
    NoWindow,
    /// The window has no client area to capture.
    EmptyClientArea,
    /// The drawing interface refused to give up a device context or a bitmap.
    GdiRefused,
    /// Both routes produced a frame, and every one was a flat colour.
    ///
    /// The characteristic signature of a window whose content the graphics
    /// device owns and the drawing interface cannot see. It is a window-mode
    /// problem rather than a transient one, and it is the finding `ADR-0008`
    /// needs.
    Featureless,
}

impl core::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NoWindow => "the handle is not a window",
            Self::EmptyClientArea => "the window has no client area",
            Self::GdiRefused => "the drawing interface refused a device context or bitmap",
            Self::Featureless => {
                "every route produced a flat frame; the graphics device owns this window's content"
            }
        })
    }
}

/// Capture a window by a named route, with no fallback.
///
/// Exposed separately because "produced a frame with content" and "produced a
/// frame that changes" are different questions, and only the caller knows which
/// it is asking. A route can return a perfectly detailed image that is the same
/// image every time, which is the failure that reads as "the camera did not
/// turn".
pub(crate) fn capture_via(window: Hwnd, route: Route) -> Result<Frame, CaptureError> {
    let (width, height) = client_size(window)?;
    match grab(window, width, height, route) {
        Ok(frame) if is_flat(&frame.gray) => Err(CaptureError::Featureless),
        other => other,
    }
}

/// The client area's dimensions, or why there are none.
fn client_size(window: Hwnd) -> Result<(i32, i32), CaptureError> {
    // SAFETY: `IsWindow` accepts any value, including an invalid handle, and
    // answers whether it currently names a window. That is the whole reason to
    // call it before anything that would require a valid one.
    if unsafe { IsWindow(window) } == 0 {
        return Err(CaptureError::NoWindow);
    }
    let mut rect = Rect::default();
    // SAFETY: `window` names a live window, checked immediately above, and
    // `rect` is a live, correctly sized structure the call writes into.
    if unsafe { GetClientRect(window, &raw mut rect) } == 0 {
        return Err(CaptureError::EmptyClientArea);
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return Err(CaptureError::EmptyClientArea);
    }
    Ok((width, height))
}

/// Every route, in the order they are tried.
pub(crate) const ROUTES: [Route; 3] = [Route::PrintWindow, Route::BitBlt, Route::ScreenCrop];

/// Is this frame a single flat colour?
///
/// A black frame is the most common first capture failure, and two black frames
/// correlate perfectly at every displacement. Detecting it here means the probe
/// reports a capture problem rather than "the camera did not turn", which would
/// be wrong in the most misleading direction available.
fn is_flat(gray: &[u8]) -> bool {
    let Some(&first) = gray.first() else {
        return true;
    };
    // Allow for a little dithering rather than requiring exact equality.
    gray.iter().all(|&v| v.abs_diff(first) <= 2)
}

fn grab_from_screen(window: Hwnd) -> Result<Frame, CaptureError> {
    let mut rect = Rect::default();
    // SAFETY: `window` is live, checked by the caller, and `rect` is a live,
    // correctly sized structure the call writes into.
    if unsafe { GetWindowRect(window, &raw mut rect) } == 0 {
        return Err(CaptureError::EmptyClientArea);
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return Err(CaptureError::EmptyClientArea);
    }

    // SAFETY: a null window handle asks for the screen's device context, which
    // is the documented way to read what the display is showing. It is released
    // on every path below.
    let screen_dc = unsafe { GetDC(core::ptr::null_mut()) };
    if screen_dc.is_null() {
        return Err(CaptureError::GdiRefused);
    }

    let result = screen_into(screen_dc, rect.left, rect.top, width, height);

    // SAFETY: `screen_dc` came from `GetDC(null)` and has not been released.
    unsafe { ReleaseDC(core::ptr::null_mut(), screen_dc) };
    result
}

fn screen_into(
    screen_dc: Hdc,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<Frame, CaptureError> {
    // SAFETY: `screen_dc` is a live device context.
    let memory_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if memory_dc.is_null() {
        return Err(CaptureError::GdiRefused);
    }
    // SAFETY: as above, and the dimensions are positive.
    let bitmap = unsafe { CreateCompatibleBitmap(screen_dc, width, height) };
    if bitmap.is_null() {
        // SAFETY: nothing has been selected into `memory_dc`.
        unsafe { DeleteDC(memory_dc) };
        return Err(CaptureError::GdiRefused);
    }

    // SAFETY: `memory_dc` is live and `bitmap` is compatible with it. The
    // previous selection is restored before either is deleted.
    let previous = unsafe { SelectObject(memory_dc, bitmap) };
    // SAFETY: both device contexts are live and the rectangle fits the bitmap,
    // which was created at exactly these dimensions.
    let copied = unsafe { BitBlt(memory_dc, 0, 0, width, height, screen_dc, x, y, SRCCOPY) != 0 };

    let frame = if copied {
        read_pixels(memory_dc, bitmap, width, height, Route::ScreenCrop)
    } else {
        Err(CaptureError::GdiRefused)
    };

    // SAFETY: `previous` is what `SelectObject` returned, and restoring it is
    // what makes the deletions below sound.
    unsafe {
        SelectObject(memory_dc, previous);
        DeleteObject(bitmap);
        DeleteDC(memory_dc);
    }
    frame
}

fn grab(window: Hwnd, width: i32, height: i32, route: Route) -> Result<Frame, CaptureError> {
    if route == Route::ScreenCrop {
        return grab_from_screen(window);
    }
    // SAFETY: `window` is live. `GetDC` returns null on failure, which is
    // checked, and the handle is released on every path below.
    let window_dc = unsafe { GetDC(window) };
    if window_dc.is_null() {
        return Err(CaptureError::GdiRefused);
    }

    let result = grab_into(window, window_dc, width, height, route);

    // SAFETY: `window_dc` came from `GetDC` for `window` and has not been
    // released, which is the exact contract `ReleaseDC` requires.
    unsafe { ReleaseDC(window, window_dc) };
    result
}

fn grab_into(
    window: Hwnd,
    window_dc: Hdc,
    width: i32,
    height: i32,
    route: Route,
) -> Result<Frame, CaptureError> {
    // SAFETY: `window_dc` is a live device context, which is the only
    // precondition for creating one compatible with it.
    let memory_dc = unsafe { CreateCompatibleDC(window_dc) };
    if memory_dc.is_null() {
        return Err(CaptureError::GdiRefused);
    }

    // SAFETY: as above, and the dimensions are positive, checked by the caller.
    let bitmap = unsafe { CreateCompatibleBitmap(window_dc, width, height) };
    if bitmap.is_null() {
        // SAFETY: `memory_dc` came from `CreateCompatibleDC` and nothing has
        // been selected into it, so deleting it is sound.
        unsafe { DeleteDC(memory_dc) };
        return Err(CaptureError::GdiRefused);
    }

    let outcome = render(window, window_dc, memory_dc, bitmap, width, height, route);

    // SAFETY: both handles were created above and are not selected into any
    // device context at this point, because `render` restores the original
    // selection before returning.
    unsafe {
        DeleteObject(bitmap);
        DeleteDC(memory_dc);
    }
    outcome
}

fn render(
    window: Hwnd,
    window_dc: Hdc,
    memory_dc: Hdc,
    bitmap: HBitmap,
    width: i32,
    height: i32,
    route: Route,
) -> Result<Frame, CaptureError> {
    // SAFETY: `memory_dc` is a live device context and `bitmap` a live bitmap
    // compatible with it. The previously selected object is restored below,
    // which is what makes deleting both afterwards sound.
    let previous = unsafe { SelectObject(memory_dc, bitmap) };

    let drawn = match route {
        // SAFETY: `window` is live and `memory_dc` has a compatible bitmap
        // selected. `PrintWindow` writes into that bitmap and nothing else.
        Route::PrintWindow => unsafe { PrintWindow(window, memory_dc, PW_RENDERFULLCONTENT) != 0 },
        // SAFETY: both device contexts are live, the rectangle lies inside the
        // bitmap created at exactly these dimensions, and `SRCCOPY` reads from
        // the source and writes to the destination only.
        Route::BitBlt => unsafe {
            BitBlt(memory_dc, 0, 0, width, height, window_dc, 0, 0, SRCCOPY) != 0
        },
        // Handled before this function is reached; it does not use the
        // window's own device context at all.
        Route::ScreenCrop => unreachable!("the screen route is served by grab_from_screen"),
    };

    let frame = if drawn {
        read_pixels(memory_dc, bitmap, width, height, route)
    } else {
        Err(CaptureError::GdiRefused)
    };

    // SAFETY: `previous` is whatever `SelectObject` returned, which is the
    // object to restore. Restoring it before deletion is what makes the
    // deletion in the caller sound.
    unsafe { SelectObject(memory_dc, previous) };
    frame
}

fn read_pixels(
    memory_dc: Hdc,
    bitmap: HBitmap,
    width: i32,
    height: i32,
    route: Route,
) -> Result<Frame, CaptureError> {
    let w = width as usize;
    let h = height as usize;
    let mut bgra = vec![0_u8; w * h * 4];

    let mut info = BitmapInfo {
        header: BitmapInfoHeader {
            size: core::mem::size_of::<BitmapInfoHeader>() as u32,
            width,
            // Negative height asks for a top-down image, so row zero is the top
            // of the window rather than the bottom. Getting this wrong flips
            // the frame vertically, which a horizontal correlation would not
            // notice — it would simply report a worse match, and the failure
            // would look like the camera not turning.
            height: -height,
            planes: 1,
            bit_count: 32,
            compression: BI_RGB,
            ..BitmapInfoHeader::default()
        },
        colours: [0; 3],
    };

    // SAFETY: `bitmap` is selected out of no device context at this point but
    // remains compatible with `memory_dc`; `bgra` is large enough for
    // `width * height` 32-bit pixels, which is what the header requests; and
    // `info` is a live, correctly sized structure.
    let lines = unsafe {
        GetDIBits(
            memory_dc,
            bitmap,
            0,
            height as u32,
            bgra.as_mut_ptr().cast::<c_void>(),
            &raw mut info,
            DIB_RGB_COLORS,
        )
    };
    if lines == 0 {
        return Err(CaptureError::GdiRefused);
    }

    // Rec. 601 luma. The correlation only needs a consistent scalar per pixel,
    // and a perceptual weighting keeps a coloured interface from dominating the
    // profile the way a plain channel average would.
    let mut gray = Vec::with_capacity(w * h);
    for &[blue, green, red, _alpha] in bgra.as_chunks::<4>().0 {
        let luma = 0.114 * f32::from(blue) + 0.587 * f32::from(green) + 0.299 * f32::from(red);
        gray.push(luma as u8);
    }

    Ok(Frame {
        width: w,
        height: h,
        gray,
        route,
    })
}

/// Find a visible top-level window whose title contains `needle`.
///
/// Case-sensitive, first match wins. A probe run names its target explicitly,
/// so matching more cleverly would only make it harder to tell which window was
/// chosen.
pub(crate) fn find_window(needle: &str) -> Option<Hwnd> {
    struct Search {
        needle: String,
        found: Hwnd,
    }

    extern "system" fn visit(window: Hwnd, param: isize) -> i32 {
        // SAFETY: `param` is the pointer passed to `EnumWindows` below, which
        // points at a `Search` that outlives the enumeration because
        // `EnumWindows` does not return until it finishes.
        let search = unsafe { &mut *(param as *mut Search) };

        // SAFETY: `window` is supplied by the enumeration and is live for the
        // duration of the callback.
        if unsafe { IsWindowVisible(window) } == 0 {
            return 1;
        }

        let mut buffer = [0_u8; 512];
        // SAFETY: `buffer` is a live array and its length is passed exactly, so
        // the call cannot write past it.
        let len = unsafe { GetWindowTextA(window, buffer.as_mut_ptr(), buffer.len() as i32) };
        if len <= 0 {
            return 1;
        }
        let title = String::from_utf8_lossy(&buffer[..len as usize]);
        if title.contains(&search.needle) {
            search.found = window;
            return 0;
        }
        1
    }

    let mut search = Search {
        needle: needle.to_owned(),
        found: core::ptr::null_mut(),
    };
    // SAFETY: `visit` matches the required signature, and the pointer handed to
    // it refers to `search`, which outlives the call because `EnumWindows` is
    // synchronous.
    unsafe { EnumWindows(visit, (&raw mut search) as isize) };

    if search.found.is_null() {
        None
    } else {
        Some(search.found)
    }
}

/// Suppress the unused warning for a route the probe reaches only by name.
#[allow(dead_code)]
const _UNUSED: unsafe extern "system" fn(*const u8, *const u8) -> Hwnd = FindWindowA;

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.gray == other.gray
    }
}

impl core::fmt::Debug for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Frame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("route", &self.route)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    #[test]
    fn an_invalid_handle_is_reported_as_no_window() {
        // COVERS: FR-PERC-009
        //
        // The requirement asks for an unsupported window mode and a transient
        // failure to be distinguishable. A handle that is not a window is
        // neither, and saying so is where that distinction starts.
        let bogus = core::ptr::without_provenance_mut(0xdead_0000);
        assert_eq!(
            capture_via(bogus, Route::PrintWindow),
            Err(CaptureError::NoWindow)
        );
    }

    #[test]
    fn a_flat_frame_is_recognised() {
        // COVERS: FR-PERC-009
        assert!(is_flat(&[0; 64]));
        assert!(is_flat(&[128; 64]));
        // Dithering is not a flat frame, but it is not content either.
        assert!(is_flat(&[10, 11, 12, 10, 11, 12]));
        assert!(!is_flat(&[0, 255, 0, 255]));
        assert!(is_flat(&[]));
    }

    #[test]
    fn a_capture_error_explains_itself() {
        // COVERS: FR-PERC-009
        assert_eq!(
            CaptureError::Featureless.to_string(),
            "every route produced a flat frame; the graphics device owns this window's content"
        );
        assert_eq!(Route::PrintWindow.to_string(), "PrintWindow");
    }
}
