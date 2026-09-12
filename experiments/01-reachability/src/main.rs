//! Known-good-matrix question 1: does synthesised relative mouse movement
//! reach a real game?
//!
//! This gates the project. The roadmap is blunt about it: "If this fails there
//! is no project." It is far better to learn that from a small probe than from
//! three months of building on the assumption.
//!
//! Deliberately outside the product architecture. It answers one question and
//! is deleted or archived when the question is resolved, and holding it to
//! product standard would slow down the thing it exists to make fast.

// Pedantic numeric-cast lints, allowed in this crate and nowhere else.
//
// The probe reduces frames to pixel statistics, and the casts between pixel
// counts, indices and floats are the arithmetic it exists to do. Writing each
// as a checked conversion would bury the method in ceremony for no safety
// gain: an out-of-range column index is caught by the bounds check, not by a
// cast.
//
// `experiments/` is explicitly not held to product standard, per
// docs/contributing/repository-layout.md, and this does not leave it. `crates/`
// carries the workspace policy unaltered.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

// The correlation method is complete and tested. Capture and input synthesis
// land next; until they do, nothing outside the tests drives it.
#[allow(dead_code)]
mod correlate;

fn main() {
    println!("reachability probe: not yet wired to capture or input");
}
