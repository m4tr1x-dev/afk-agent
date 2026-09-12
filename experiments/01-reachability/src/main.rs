//! Known-good-matrix question 1: does synthesised relative mouse movement
//! reach a real game?
//!
//! This gates the project. The roadmap is blunt about it: "If this fails there
//! is no project." Far better to learn that from a small probe than from three
//! months of building on the assumption.
//!
//! Deliberately outside the product architecture, and archived when the
//! question is resolved. It does use the real [`afk_input`], because the
//! question is whether **the public input API the product ships** reaches a
//! game, and a probe with its own synthesis would answer a different question.
//!
//! # Running it
//!
//! ```text
//! reachability-probe --window "Xonotic" --arm 8
//! ```
//!
//! The arming delay exists because the foreground guard is real: the probe
//! refuses to send anything unless its target is in front, so there has to be
//! time to switch to the game after starting it from a shell.
//!
//! # What it reports
//!
//! Four conjunct tests and three null controls, each printed with its verdict,
//! and a machine-readable summary written to `results/`. No single measurement
//! is treated as an answer, because a scene moves on its own: an explosion
//! displaces pixels too.

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

mod capture;
mod correlate;
mod run;

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut window = String::new();
    let mut arm_seconds = 8_u64;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--window" => window = args.next().unwrap_or_default(),
            "--arm" => {
                arm_seconds = args.next().and_then(|v| v.parse().ok()).unwrap_or(8);
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

    if window.is_empty() {
        eprintln!("--window is required\n\n{USAGE}");
        return ExitCode::from(2);
    }

    match run::execute(&window, arm_seconds) {
        Ok(report) => {
            println!("\n{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("\nthe probe could not reach a verdict: {error}");
            eprintln!(
                "That is not the same as the answer being no. A capture that \
                 never produced a usable frame says nothing about whether input \
                 arrived."
            );
            ExitCode::from(1)
        }
    }
}

const USAGE: &str = "\
reachability-probe --window <title substring> [--arm <seconds>]

  --window   Part of the target window's title. First visible match wins.
  --arm      Seconds to wait before starting, so you can bring the game
             forward. Default 8.

The probe refuses to send input unless the target is in the foreground, so the
arming delay is not a convenience.
";
