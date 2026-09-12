//! The only component in the system that synthesises input.
//!
//! `FR-ACT-008` states that as a requirement rather than a convention, and the
//! rationale says why: "A second call site is a second place the release
//! guarantees can be violated, and it is found by a user rather than by a
//! test."
//!
//! Four mechanisms keep it true, and none of them is this comment. The crate
//! declares its own extern block instead of depending on a bindings crate, so
//! Cargo's feature unification cannot hand the input API to anything else;
//! `clippy.toml` forbids the bindings-crate paths with no exemption anywhere;
//! `deny.toml` bans the dependency that would make them reachable; and
//! `tools/check_call_sites.py` catches whatever those three do not yet know
//! about.
//!
//! # Scope
//!
//! This is the minimum the reachability probe needs, not the executor. It
//! carries pointer movement, the foreground guard and the release path.
//! Keyboard synthesis, sustained actions, the denied-key list and the
//! per-subgoal ownership in `FR-ACT-007` arrive with the executor in M3.
//!
//! The order is deliberate. Known-good-matrix question 1 asks whether the
//! public input API reaches a real game, and it gates the project. Answering it
//! with a probe that had its own `SendInput` would answer a question about code
//! the product does not ship.
//!
//! # The guarantee that shapes everything
//!
//! `INV-ACT-001`: no input is delivered while the target window is not the
//! foreground window. It is checked at the point of delivery rather than the
//! point of decision, because a check performed when the action was chosen —
//! hundreds of milliseconds earlier — proves nothing about the moment the event
//! is sent.

mod sys;

pub use sys::Hwnd;

/// Why a synthesis request was refused.
///
/// A refusal is information rather than a failure. `FR-ACT-006` requires a
/// rejected action to be reported back rather than silently dropped, because a
/// model with no way to learn that something cannot happen asks for it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refused {
    /// The target window was not in the foreground at the moment of delivery.
    ///
    /// `INV-ACT-001`. The primary control on blast radius: it is what stops the
    /// agent acting on a window that is not the game.
    NotForeground,
    /// The operating system did not accept the event.
    ///
    /// Usually means a window running at higher integrity holds the foreground,
    /// which a user-space process cannot and should not reach.
    Rejected,
    /// The requested movement exceeded the per-event bound.
    ///
    /// `FR-ACT-004`: games reading raw input reject or clamp a single
    /// implausible movement, so a large turn is a sequence of small ones. The
    /// bound is enforced here rather than trusted to the caller.
    TooLarge,
}

impl core::fmt::Display for Refused {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let text = match self {
            Self::NotForeground => "the target window was not in the foreground",
            Self::Rejected => "the operating system did not accept the event",
            Self::TooLarge => "the movement exceeded the per-event bound",
        };
        f.write_str(text)
    }
}

impl core::error::Error for Refused {}

/// Largest relative movement permitted in one event, in mouse units.
///
/// A placeholder. `06-action-and-input.md` open question 4 says the per-tick
/// magnitude cap "is a guess until measured against real games", and measuring
/// it is part of what the reachability probe does: a game that clamps shows a
/// displacement that stops growing, and where it stops is this number.
///
/// Until then the value errs small. Too small costs a slower turn; too large
/// risks the input being discarded as implausible, which would look exactly
/// like the input not arriving at all — and that is the question being asked.
pub const MAX_RELATIVE_STEP: i32 = 400;

/// Synthesises input for one target window.
///
/// Holds no lock that anything else can take. `FR-ACT-001` requires release to
/// be possible at any moment, including from a panic path, and a release that
/// can block is a release that can fail to happen.
#[derive(Debug)]
pub struct Synthesiser {
    target: Hwnd,
}

impl Synthesiser {
    /// Bind to a target window.
    ///
    /// Every event this synthesiser sends is gated on that window still being
    /// in the foreground.
    #[must_use]
    pub fn bind(target: Hwnd) -> Self {
        Self { target }
    }

    /// The window this synthesiser is bound to.
    #[must_use]
    pub fn target(&self) -> Hwnd {
        self.target
    }

    /// Is the target in the foreground right now?
    ///
    /// Exposed so a caller can pause before doing work whose result would be
    /// discarded. It is not a substitute for the check inside `move_relative`:
    /// the foreground can change between the two, which is exactly why
    /// `INV-ACT-001` is enforced at delivery.
    #[must_use]
    pub fn target_is_foreground(&self) -> bool {
        !self.target.is_null() && core::ptr::eq(sys::foreground_window(), self.target)
    }

    /// Move the pointer by a relative delta.
    ///
    /// This is the path that matters for a game. `FR-ACT-004`: games controlling
    /// a camera read relative movement deltas from the raw input stream and
    /// typically capture the cursor, so a reposition produces no delta at all
    /// and the cursor's absolute position is meaningless.
    ///
    /// # Errors
    ///
    /// [`Refused::NotForeground`] when the target is not in front,
    /// [`Refused::TooLarge`] when either component exceeds
    /// [`MAX_RELATIVE_STEP`], and [`Refused::Rejected`] when the system declines
    /// the event.
    pub fn move_relative(&self, dx: i32, dy: i32) -> Result<(), Refused> {
        if dx.abs() > MAX_RELATIVE_STEP || dy.abs() > MAX_RELATIVE_STEP {
            return Err(Refused::TooLarge);
        }
        self.deliver(sys::MouseInput {
            dx,
            dy,
            mouse_data: 0,
            flags: sys::MOUSEEVENTF_MOVE,
            time: 0,
            extra_info: 0,
        })
    }

    /// Move the pointer to an absolute position on the virtual desktop.
    ///
    /// Present for one reason: it is the null control in the reachability
    /// experiment. `FR-ACT-004` predicts this produces no camera turn in a game
    /// reading raw input, and if it turns out to work as well as relative
    /// movement then that requirement's rationale is weaker than the
    /// specification claims and the page needs amending.
    ///
    /// Coordinates are normalised to `0..=65535` across the virtual desktop, as
    /// the platform requires.
    ///
    /// # Errors
    ///
    /// As [`Synthesiser::move_relative`], except that [`Refused::TooLarge`] does
    /// not apply.
    pub fn move_absolute(&self, x: u16, y: u16) -> Result<(), Refused> {
        self.deliver(sys::MouseInput {
            dx: i32::from(x),
            dy: i32::from(y),
            mouse_data: 0,
            flags: sys::MOUSEEVENTF_MOVE | sys::MOUSEEVENTF_ABSOLUTE | sys::MOUSEEVENTF_VIRTUALDESK,
            time: 0,
            extra_info: 0,
        })
    }

    /// Release everything this synthesiser is holding.
    ///
    /// `FR-ACT-001`, and the most important behaviour in the input layer: "An
    /// agent that stops while holding a movement key leaves the character
    /// walking, and the user cannot undo that from outside the game."
    ///
    /// Idempotent, takes no locks, and deliberately does **not** check the
    /// foreground guard. Releasing is always permitted: a release that the
    /// guard could refuse would leave input held precisely when the situation
    /// has already gone wrong.
    ///
    /// Nothing is held yet — this synthesiser carries pointer movement only,
    /// which is instantaneous. The method exists now so that every caller is
    /// written against it from the start, rather than having it retrofitted
    /// once there is something to release.
    pub fn release_all(&self) {
        // Nothing held. When the executor adds key and button state in M3, this
        // walks the held set, releases modifiers last, and clears it.
    }

    fn deliver(&self, mouse: sys::MouseInput) -> Result<(), Refused> {
        // INV-ACT-001. Checked here, immediately before the event is sent,
        // rather than anywhere earlier: the foreground can change between a
        // decision and its delivery, and only the last check counts.
        if !self.target_is_foreground() {
            return Err(Refused::NotForeground);
        }

        let event = sys::Input {
            kind: sys::INPUT_MOUSE,
            value: sys::InputUnion { mouse },
        };
        if sys::send(event) {
            Ok(())
        } else {
            Err(Refused::Rejected)
        }
    }
}

impl Drop for Synthesiser {
    fn drop(&mut self) {
        // INV-SAFE-002: after any stop, panic, focus loss or crash, no input
        // remains held. A panic unwinding past a synthesiser has to release
        // what it held, which is why the release profile unwinds rather than
        // aborts.
        self.release_all();
    }
}

#[cfg(test)]
mod tests {
    // Narrow scope, and only here: inside a test a panic is the reporting
    // mechanism. The coding standards forbid crate-wide allows.
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    /// A handle that is never any window's, used to exercise the guard.
    fn never_foreground() -> Hwnd {
        core::ptr::without_provenance_mut(0xdead_0000)
    }

    #[test]
    fn a_synthesiser_bound_to_a_foreign_window_delivers_nothing() {
        // COVERS: INV-ACT-001
        //
        // The primary control on blast radius. A synthesiser whose target is
        // not in front must refuse, and refuse for that reason rather than by
        // failing somewhere deeper.
        let synth = Synthesiser::bind(never_foreground());
        assert!(!synth.target_is_foreground());
        assert_eq!(synth.move_relative(10, 0), Err(Refused::NotForeground));
        assert_eq!(synth.move_absolute(100, 100), Err(Refused::NotForeground));
    }

    #[test]
    fn a_null_target_is_never_the_foreground() {
        // COVERS: INV-ACT-001
        //
        // `GetForegroundWindow` returns null when no window has focus — a
        // locked screen, a session switch. Comparing null to null would make
        // the guard pass at exactly the moment it must not.
        let synth = Synthesiser::bind(core::ptr::null_mut());
        assert!(!synth.target_is_foreground());
        assert_eq!(synth.move_relative(1, 1), Err(Refused::NotForeground));
    }

    #[test]
    fn an_implausible_movement_is_refused_before_the_guard() {
        // COVERS: FR-ACT-004
        //
        // Ordering matters. The bound is a property of the request, not of the
        // moment, so a caller learns its delta is too large whether or not the
        // window happened to be in front.
        let synth = Synthesiser::bind(never_foreground());
        assert_eq!(
            synth.move_relative(MAX_RELATIVE_STEP + 1, 0),
            Err(Refused::TooLarge)
        );
        assert_eq!(
            synth.move_relative(0, -(MAX_RELATIVE_STEP + 1)),
            Err(Refused::TooLarge)
        );
    }

    #[test]
    fn a_movement_at_the_bound_is_permitted() {
        // COVERS: FR-ACT-004
        let synth = Synthesiser::bind(never_foreground());
        // Refused by the guard rather than by the bound, which is the point.
        assert_eq!(
            synth.move_relative(MAX_RELATIVE_STEP, MAX_RELATIVE_STEP),
            Err(Refused::NotForeground)
        );
    }

    #[test]
    fn release_is_idempotent_and_needs_no_foreground() {
        // COVERS: FR-ACT-001
        //
        // "Release is idempotent, ordered so that modifier keys are released
        // last, and takes no locks that anything else can hold." A release the
        // foreground guard could refuse would leave input held exactly when
        // something has already gone wrong.
        let synth = Synthesiser::bind(never_foreground());
        assert!(!synth.target_is_foreground());
        synth.release_all();
        synth.release_all();
    }

    #[test]
    fn the_platform_structures_match_the_platform() {
        // COVERS: FR-ACT-008
        //
        // A hand-declared layout that disagrees with the operating system's
        // does not crash: the call is rejected, zero is returned, and the agent
        // silently stops being able to act. The compile-time assertions in
        // `sys` pin it; this records why they are there.
        assert_eq!(core::mem::size_of::<sys::Input>(), 40);
        assert_eq!(core::mem::align_of::<sys::Input>(), 8);
    }

    #[test]
    fn a_refusal_explains_itself() {
        // COVERS: FR-ACT-006
        //
        // A rejected action is reported back rather than dropped, so the reason
        // has to survive as something a caller can render.
        assert_eq!(
            Refused::NotForeground.to_string(),
            "the target window was not in the foreground"
        );
        assert_eq!(
            Refused::TooLarge.to_string(),
            "the movement exceeded the per-event bound"
        );
    }
}
