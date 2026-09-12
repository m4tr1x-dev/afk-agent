//! The operating system's input surface, and the whole of it.
//!
//! Declared here by hand rather than pulled from a bindings crate, and that is
//! a deliberate architectural choice rather than an aversion to dependencies.
//!
//! Cargo unifies features across the entire dependency graph. If any crate in
//! the workspace enabled a bindings crate's keyboard-and-mouse feature, every
//! crate that depends on it could call `SendInput`, and a dependency ban would
//! not notice — the ban sees crates, not features. Declaring the extern locally
//! sidesteps unification completely.
//!
//! It has a second effect worth as much. Because this crate never names the
//! paths that `clippy.toml` forbids, it needs no `#[allow]` for them, so there
//! is no exemption anywhere in the workspace for a future contributor to copy
//! into a crate that should not have one.

// This module is the platform boundary. Every unsafe block below carries a
// SAFETY comment naming the invariant that makes it sound, which the workspace
// lints enforce rather than leave to review.

use core::ffi::c_void;

/// An opaque window handle.
pub type Hwnd = *mut c_void;

pub(crate) const INPUT_MOUSE: u32 = 0;

pub(crate) const MOUSEEVENTF_MOVE: u32 = 0x0001;
pub(crate) const MOUSEEVENTF_ABSOLUTE: u32 = 0x8000;
pub(crate) const MOUSEEVENTF_VIRTUALDESK: u32 = 0x4000;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct MouseInput {
    pub(crate) dx: i32,
    pub(crate) dy: i32,
    pub(crate) mouse_data: u32,
    pub(crate) flags: u32,
    pub(crate) time: u32,
    pub(crate) extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct KeyboardInput {
    pub(crate) vk: u16,
    pub(crate) scan: u16,
    pub(crate) flags: u32,
    pub(crate) time: u32,
    pub(crate) extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union InputUnion {
    pub(crate) mouse: MouseInput,
    pub(crate) keyboard: KeyboardInput,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Input {
    pub(crate) kind: u32,
    pub(crate) value: InputUnion,
}

// A hand-declared repr(C) structure that disagrees with the platform's is the
// kind of defect that does not crash: `SendInput` rejects the call, returns
// zero, and the agent silently stops being able to act. Pin the layout here so
// a mistake is a compile error instead.
const _: () = {
    assert!(core::mem::size_of::<Input>() == 40);
    assert!(core::mem::align_of::<Input>() == 8);
    assert!(core::mem::size_of::<MouseInput>() == 32);
};

// Both functions live in user32. Naming the library here rather than relying
// on a `.lib` on the link line is what keeps this crate's platform surface
// visible in one place and free of a build script.
#[link(name = "user32")]
unsafe extern "system" {
    /// Synthesise input events.
    ///
    /// Returns the number of events successfully inserted into the input
    /// stream. A return below `count` means the call was blocked, which happens
    /// when a more privileged window holds the foreground.
    pub(crate) fn SendInput(count: u32, inputs: *const Input, size: i32) -> u32;

    /// The window the user is currently working with.
    pub(crate) fn GetForegroundWindow() -> Hwnd;
}

/// Send one input event, returning whether the system accepted it.
///
/// The single point at which this project touches the operating system's input
/// stream. Everything above it is bookkeeping.
pub(crate) fn send(event: Input) -> bool {
    let size = i32::try_from(core::mem::size_of::<Input>()).unwrap_or(0);
    // SAFETY: `event` is a live, correctly initialised `Input` whose layout is
    // pinned by the assertions above, `size` is its exact size in bytes, and
    // the pointer is valid for the duration of the call because `event` outlives
    // it. `SendInput` reads `count * size` bytes and writes nothing.
    let sent = unsafe { SendInput(1, &raw const event, size) };
    sent == 1
}

/// The window the user is currently working with, or null.
pub(crate) fn foreground_window() -> Hwnd {
    // SAFETY: `GetForegroundWindow` takes no arguments, reads no memory the
    // caller owns, and returns either a valid handle or null. It is safe to
    // call from any thread.
    unsafe { GetForegroundWindow() }
}
