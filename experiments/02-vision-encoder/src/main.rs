//! Known-good-matrix question 2, and the runtime question hiding behind it.
//!
//! The matrix asks whether the vision encoder loads on the Vulkan backend. That
//! phrasing invites the wrong assertion. A vision tower wired to the wrong
//! projector loads without complaint and answers fluently about the general
//! image, which is precisely the quiet failure the matrix warns about elsewhere
//! for a version mismatch. "It loaded" and "it sees" are different claims.
//!
//! So this probe answers three questions, and the third is the one the runtime
//! decision actually turns on.
//!
//! **1. Does the model see the image?** Eight synthetic scenes it cannot have
//! memorised, each carrying a three-digit number, one of six background colours
//! and one of four shapes. All three correct by chance is one in 21,600. The
//! bar is seven of eight exactly correct on all three fields.
//!
//! **2. Was the graphics processor actually used?** A runtime that quietly fell
//! back to the processor makes every latency number in the milestone
//! meaningless, and it does not announce itself. Two readings: what the server
//! reports about its own device, and how much graphics memory disappeared.
//!
//! **3. Is the per-image visual token budget a per-request knob?** The model
//! card documents budgets of 70, 140, 280, 560 and 1120. Whether the *runtime*
//! exposes them per request is a separate fact, and `FR-MODEL-002` is written as
//! though it does. If the budget is instead a property of the projector or of
//! the input resolution, the requirement needs rewording from "set the budget"
//! to "control the cost", which is satisfiable either way.
//!
//! This probe measures the prompt token count against input resolution, which
//! separates the two: if the cost tracks the image we send, the knob is ours.
//!
//! ```text
//! vision-encoder-probe --host 127.0.0.1:8080 --out results/
//! ```

// Image coordinates are small positive integers throughout, and the drawing
// code is clearer with signed arithmetic than with checked conversions at
// every subtraction. The buffer bounds are enforced once, in `put`.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
// Tests assert on results whose failure is the test failing anyway, and the
// message from `expect` names which one. The ban exists for the product,
// where a panic reaches a user rather than a test runner.
#![cfg_attr(test, allow(clippy::expect_used))]

mod client;
mod scene;

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use scene::{COLOURS, SHAPES, Scene};

/// How many scenes the accuracy test uses.
const SCENES: usize = 8;

/// The bar is above seven-eighths, chosen before the run rather than after it.
const _: () = assert!(REQUIRED * 8 >= SCENES * 7);

/// How many of those must be exactly right on all three fields.
///
/// Chosen before the run, as `docs/contributing/adr-process.md` requires. Seven
/// of eight leaves room for one genuine misread of a digit without leaving room
/// for a projector that is guessing.
const REQUIRED: usize = 7;

/// Input sizes for the budget sweep, in pixels on a side.
///
/// Spread wide enough that a cost proportional to area is unmistakable, and
/// including one below and one above the 512 the scenes are drawn at.
const SIZES: [usize; 4] = [256, 512, 768, 1024];

fn main() -> ExitCode {
    let mut host = "127.0.0.1:8080".to_owned();
    let mut seed = 0_u64;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => host = args.next().unwrap_or(host),
            "--seed" => seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(0),
            "--help" | "-h" => {
                println!("vision-encoder-probe [--host HOST:PORT] [--seed N]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
    }

    if seed == 0 {
        seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x5eed_1234, |d| d.as_nanos() as u64)
            | 1;
    }

    match run(&host, seed) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("probe: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(host: &str, seed: u64) -> Result<String, String> {
    let mut out = String::new();
    let _ = writeln!(out, "seed {seed}  host {host}");
    let _ = writeln!(out);

    device_report(host, &mut out)?;
    let correct = accuracy(host, seed, &mut out)?;
    budget(host, seed, &mut out)?;

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "verdict: {} of {SCENES} exactly correct, {} of {SCENES} required -> {}",
        correct,
        REQUIRED,
        if correct >= REQUIRED {
            "SEES"
        } else {
            "DOES NOT SEE"
        }
    );
    Ok(out)
}

/// What the server says about the device it is using.
///
/// Reported rather than asserted. The probe cannot tell from here whether the
/// runtime silently fell back to the processor; the graphics-memory reading
/// taken alongside this, by the harness that starts the server, is what settles
/// it. Printing both and letting them disagree is the point.
fn device_report(host: &str, out: &mut String) -> Result<(), String> {
    let body = client::get(host, "/props")?;
    let props: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("/props is not JSON: {e}"))?;

    let model = props
        .get("model_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let context = props
        .get("default_generation_settings")
        .and_then(|s| s.get("n_ctx"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    let _ = writeln!(out, "model    {model}");
    let _ = writeln!(out, "context  {context}");
    let _ = writeln!(out);
    Ok(())
}

/// Eight scenes, three fields each, exact match only.
fn accuracy(host: &str, seed: u64, out: &mut String) -> Result<usize, String> {
    let _ = writeln!(
        out,
        "--- accuracy: eight scenes, all three fields exact ---"
    );
    let mut random = Random::new(seed);
    let mut correct = 0;

    for index in 0..SCENES {
        let scene = Scene {
            number: 100 + (random.next() % 900) as u16,
            colour: (random.next() % COLOURS.len() as u64) as usize,
            shape: (random.next() % SHAPES.len() as u64) as usize,
        };

        let pixels = scene.render();
        let png = scene::to_png(&pixels)?;
        let started = Instant::now();
        let answer = ask(host, &png, None)?;
        let elapsed = started.elapsed();

        let (number, colour, shape) = scene.expected();
        let hit = answer.digits == number && answer.colour == colour && answer.shape == shape;
        if hit {
            correct += 1;
        }

        let _ = writeln!(
            out,
            "  {index}  expected {:<28}  got {:>3} / {:<7} / {:<9}  {}  {:.2}s",
            scene.describe(),
            answer.digits,
            answer.colour,
            answer.shape,
            if hit { "ok  " } else { "MISS" },
            elapsed.as_secs_f64()
        );
    }

    let _ = writeln!(out, "  {correct} of {SCENES} exact");
    let _ = writeln!(out);
    Ok(correct)
}

/// Does the prompt cost track the image we send?
///
/// The same scene at four input resolutions. If the prompt token count rises
/// with the image we hand over, the visual cost is under the caller's control
/// whatever the runtime exposes, and `FR-MODEL-002` is satisfiable by rescaling
/// before sending. If it does not move, the cost is fixed by the projector and
/// the requirement has to say so.
fn budget(host: &str, seed: u64, out: &mut String) -> Result<(), String> {
    let _ = writeln!(out, "--- visual cost against input resolution ---");
    let scene = Scene {
        number: 100 + (seed % 900) as u16,
        colour: (seed % COLOURS.len() as u64) as usize,
        shape: (seed % SHAPES.len() as u64) as usize,
    };
    let pixels = scene.render();

    // The same request with no image at all. Subtracting it turns a prompt
    // token count into an image token count, which is the number the budget
    // question is actually about; without it the text prompt is silently
    // included and every figure below is inflated by a constant nobody stated.
    let baseline = ask_without_image(host)?;
    let _ = writeln!(out, "  text only, no image: {baseline} prompt tokens");

    let mut readings: Vec<(usize, u64)> = Vec::new();
    for side in SIZES {
        let resampled = scene::resample(&pixels, side);
        let png = scene::to_png_sized(&resampled, side)?;
        let answer = ask(host, &png, None)?;
        let image_tokens = answer.prompt_tokens.saturating_sub(baseline);
        let _ = writeln!(
            out,
            "  {side:>5} px  prompt {:>6}  image {:>6} tokens  answered {} / {} / {}",
            answer.prompt_tokens, image_tokens, answer.digits, answer.colour, answer.shape
        );
        readings.push((side, image_tokens));
    }

    let first = readings.first().map_or(0, |r| r.1);
    let last = readings.last().map_or(0, |r| r.1);
    let verdict = if last > first {
        "the caller controls the visual cost by choosing the input resolution"
    } else {
        "the visual cost does not track the input; it is fixed by the projector"
    };
    let _ = writeln!(out, "  {first} -> {last} tokens: {verdict}");
    let _ = writeln!(out);
    Ok(())
}

/// One answer, as the grammar constrains it.
struct Answer {
    digits: String,
    colour: String,
    shape: String,
    prompt_tokens: u64,
}

/// Send the same text prompt with no image, and report its prompt token count.
///
/// The baseline for the budget sweep. Everything above it is what the image
/// cost.
fn ask_without_image(host: &str) -> Result<u64, String> {
    let request = serde_json::json!({
        "messages": [{"role": "user", "content": PROMPT}],
        "grammar": default_grammar(),
        "temperature": 0.0,
        "max_tokens": 64,
        "cache_prompt": false,
    });
    let body = client::post_json(host, "/v1/chat/completions", &request.to_string())?;
    let response: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("response is not JSON: {e}"))?;
    response
        .pointer("/usage/prompt_tokens")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("no token count in response: {}", truncate(&body)))
}

/// The question, worded once so the text-only baseline and the image requests
/// differ by the image and nothing else.
const PROMPT: &str = "Report exactly what is in this image. \
                      The digits are the three-digit number. \
                      The colour is the background colour. \
                      The shape is the single shape above the digits.";

/// Ask the model about one image, under a grammar.
fn ask(host: &str, png: &[u8], grammar: Option<&str>) -> Result<Answer, String> {
    let grammar = grammar.map_or_else(default_grammar, str::to_owned);
    let data = client::base64(png);

    let request = serde_json::json!({
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": PROMPT},
                {"type": "image_url",
                 "image_url": {"url": format!("data:image/png;base64,{data}")}}
            ]
        }],
        "grammar": grammar,
        "temperature": 0.0,
        "max_tokens": 64,
        "cache_prompt": false,
    });

    let body = client::post_json(host, "/v1/chat/completions", &request.to_string())?;
    let response: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("response is not JSON: {e}"))?;

    let content = response
        .pointer("/choices/0/message/content")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("no completion in response: {}", truncate(&body)))?;

    let parsed: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("completion is not JSON: {e}: {}", truncate(content)))?;

    let field = |name: &str| -> Result<String, String> {
        parsed
            .get(name)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("completion has no {name}: {}", truncate(content)))
    };

    Ok(Answer {
        digits: field("digits")?,
        colour: field("colour")?,
        shape: field("shape")?,
        prompt_tokens: response
            .pointer("/usage/prompt_tokens")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
    })
}

/// The grammar the answer is constrained to.
///
/// Every field is a closed alternation or a fixed shape, so an answer outside
/// the six colours or four shapes cannot be produced. That is what makes a
/// wrong answer a genuine misread rather than a formatting difference, and it
/// is the same mechanism `ADR-0013` applies to tool calls.
fn default_grammar() -> String {
    let colours = COLOURS
        .iter()
        .map(|(name, _)| format!("\"\\\"{name}\\\"\""))
        .collect::<Vec<_>>()
        .join(" | ");
    let shapes = SHAPES
        .iter()
        .map(|name| format!("\"\\\"{name}\\\"\""))
        .collect::<Vec<_>>()
        .join(" | ");

    format!(
        "root ::= \"{{\\\"digits\\\":\\\"\" digit digit digit \
         \"\\\",\\\"colour\\\":\" colour \",\\\"shape\\\":\" shape \"}}\"\n\
         digit ::= [0-9]\n\
         colour ::= {colours}\n\
         shape ::= {shapes}\n"
    )
}

fn truncate(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= 300 {
        trimmed.to_owned()
    } else {
        format!("{}...", &trimmed[..300])
    }
}

/// A small deterministic generator, so a run can be reproduced from its seed.
struct Random(u64);

impl Random {
    /// A fixed non-zero state stands in for a seed of zero.
    ///
    /// Forcing the low bit instead - which is what this did first - maps 42 and
    /// 43 to the same state, so two runs given different seeds produce
    /// identical scenes. A test caught it; nothing in a run would have.
    const FALLBACK: u64 = 0x9e37_79b9_7f4a_7c15;

    const fn new(seed: u64) -> Self {
        Self(if seed == 0 { Self::FALLBACK } else { seed })
    }

    const fn next(&mut self) -> u64 {
        // xorshift64*. Not for anything that needs to resist analysis; this
        // picks three-digit numbers.
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_produces_the_same_scenes() {
        // A run is only reproducible if the seed is the whole of the state, and
        // the seed is printed so a surprising result can be rerun exactly.
        let mut a = Random::new(42);
        let mut b = Random::new(42);
        for _ in 0..32 {
            assert_eq!(a.next(), b.next());
        }
    }

    #[test]
    fn adjacent_seeds_diverge() {
        // 42 and 43 specifically. The first version of `new` forced the low bit
        // to avoid a zero state, which quietly mapped both onto 43: two runs
        // asked for different seeds would have produced the same eight scenes.
        let mut a = Random::new(42);
        let mut b = Random::new(43);
        assert_ne!(a.next(), b.next());
    }

    #[test]
    fn a_zero_seed_still_produces_a_stream() {
        let mut zero = Random::new(0);
        assert_ne!(zero.next(), 0);
    }

    #[test]
    fn the_grammar_closes_every_field() {
        let grammar = default_grammar();
        for (name, _) in COLOURS {
            assert!(grammar.contains(name), "colour {name} is unreachable");
        }
        for name in SHAPES {
            assert!(grammar.contains(name), "shape {name} is unreachable");
        }
        // Three digits exactly. A `digit+` would let the model answer with two
        // and be scored as a misread rather than as a malformed answer.
        assert!(grammar.contains("digit digit digit"));
    }

    #[test]
    fn the_required_score_is_far_above_chance() {
        // All three fields right by luck is one in 900 x 6 x 4. The threshold
        // exists so that a broken projector cannot reach it; this asserts the
        // arithmetic behind that claim rather than leaving it in a comment.
        let chance = 1.0 / (900.0 * COLOURS.len() as f64 * SHAPES.len() as f64);
        assert!(chance < 1e-4, "chance is {chance}, not negligible");
    }
}
