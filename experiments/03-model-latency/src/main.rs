//! Known-good-matrix question 3: what does a tactical tick cost?
//!
//! The matrix is blunt about why this cannot be measured on an idle machine:
//! *"The figure without a game in memory is not the figure that matters."*
//! The agent and the game share one graphics device and the game was there
//! first, so a latency taken with 24 GiB free is a latency nobody will ever see.
//!
//! Five numbers in the specification are placeholders waiting on this, and
//! `15-performance-budgets.md` marks every one of them **Unvalidated**:
//!
//! | Figure | Placeholder |
//! | --- | --- |
//! | Tactical call | 150–400 ms |
//! | Deliberative call | 3–15 s |
//! | `NFR-MODEL-001` | follows from the tactical figure |
//! | Tactical cadence | 1–4 Hz, "an intention" |
//! | Weights plus cache per variant | absent entirely |
//!
//! This probe measures the first, on a real captured game frame, with the game
//! running — and reports the same measurement without the game as the control,
//! because the interesting quantity is the *difference* and there is no
//! difference without a baseline.
//!
//! ```text
//! model-latency-probe --host 127.0.0.1:8080 --frame E:/afk-agent/frames/frame-0000.png
//! ```
//!
//! **A prediction, written before the run**, as the process asks. On this card
//! through Vulkan with a 12B-class model at a small visual budget, decoding is
//! quick but the vision tower runs on every request and prefill dominates. The
//! honest expectation is 250–600 ms and that the 4 Hz upper bound in
//! `05-reasoning-loop.md` is out of reach. If that is what comes back, the page
//! to change is the cadence, not the measurement.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#![cfg_attr(test, allow(clippy::expect_used))]

mod client;

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::Instant;

/// How many requests per condition.
///
/// Fifty, because `20-evaluation-harness.md` asks for a median and a 95th
/// percentile, and a 95th percentile from ten samples is the largest of ten.
const REQUESTS: usize = 50;

/// The visual budgets to sweep, as image widths in pixels.
///
/// The runtime prices an image by its resolution rather than by a per-request
/// parameter — measured in `experiments/02-vision-encoder/` — so this is the
/// dial, and these are the sizes that produced 83, 123 and 443 image tokens.
const WIDTHS: [usize; 3] = [256, 512, 1024];

fn main() -> ExitCode {
    let mut host = "127.0.0.1:8080".to_owned();
    let mut frame = String::new();
    let mut label = "unlabelled".to_owned();
    let mut requests = REQUESTS;
    let mut prefix = 0_usize;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => host = args.next().unwrap_or(host),
            "--frame" => frame = args.next().unwrap_or_default(),
            "--label" => label = args.next().unwrap_or(label),
            "--requests" => requests = args.next().and_then(|v| v.parse().ok()).unwrap_or(REQUESTS),
            "--prefix" => prefix = args.next().and_then(|v| v.parse().ok()).unwrap_or(0),
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

    if frame.is_empty() {
        eprintln!("--frame is required\n\n{USAGE}");
        return ExitCode::from(2);
    }

    match run(&host, &frame, &label, requests, prefix) {
        Ok(report) => {
            print!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("model-latency-probe: {error}");
            ExitCode::from(1)
        }
    }
}

const USAGE: &str = "\
model-latency-probe --frame PATH [--host HOST:PORT] [--label TEXT] [--requests N]

  --frame     A PNG captured from a real game, written by capture-probe --save.
              A synthetic image measures a different pipeline.
  --label     What condition this run is, for the transcript: 'game resident'
              or 'no game'. The difference between the two is the finding.
  --requests  Requests per visual budget. Default 50, because a 95th
              percentile from ten samples is the largest of ten.
  --prefix N  Pad the prompt with N stable sentences before the question, to
              the size the product's context actually is. Without it the
              prompt is forty tokens and the prefix cache appears to buy
              nothing, which says more about the probe than about the cache.
";

fn run(
    host: &str,
    frame: &str,
    label: &str,
    requests: usize,
    prefix: usize,
) -> Result<String, String> {
    // A stable preamble of the size the product's prompt actually is.
    //
    // Without this the measurement is misleading in a way that matters: the
    // probe's own prompt is about forty tokens, so there is nothing for the
    // prefix cache to hit and it appears to buy nothing. The product's blocks 1
    // to 5 are hundreds or thousands of tokens and are byte-identical between
    // calls by FR-CTX-002 — which is the requirement's entire purpose.
    //
    // The filler is deterministic and identical between requests, which is what
    // makes it cacheable. Real content would be too; this only has to be the
    // right size.
    let mut preamble = String::new();
    for index in 0..prefix {
        let _ = write!(
            preamble,
            "rule {index}: the agent does not act outside the target window. "
        );
    }
    let image = load(frame)?;
    let mut out = String::new();
    let _ = writeln!(out, "condition   {label}");
    let _ = writeln!(
        out,
        "frame       {frame}  ({}x{})",
        image.width, image.height
    );
    let _ = writeln!(out, "requests    {requests} per budget");
    let _ = writeln!(
        out,
        "preamble    {} characters of stable prefix",
        preamble.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "{:>7}  {:>8}  {:>8}  {:>8}  {:>8}  {:>7}",
        "width", "p50 ms", "p95 ms", "min ms", "max ms", "tokens"
    );

    // Three conditions, because neither of the obvious two is the product's
    // tick and they mislead in opposite directions.
    //
    //   cache off              nothing is reused, including the system prompt
    //                          the product deliberately keeps byte-identical
    //   cache on, same image   everything is reused, including the image — a
    //                          best case no real tick ever sees
    //   cache on, fresh image  blocks 1 and 2 hit, the image does not. This is
    //                          the tick.
    for (cached, fresh, label) in [
        (false, false, "OFF                (nothing reused)"),
        (
            true,
            false,
            "ON, same image     (best case, not a real tick)",
        ),
        (
            true,
            true,
            "ON, fresh image    (the tick: prefix hits, image does not)",
        ),
    ] {
        let _ = writeln!(out, "  prefix cache {label}");

        // The floor: the same request with no image at all. Without it there is
        // no way to tell what the image costs from what the round trip costs,
        // and a conclusion about the vision tower would rest on both.
        ask_text(host, cached, &preamble)?;
        let mut floor = Vec::with_capacity(requests);
        for _ in 0..requests {
            let started = Instant::now();
            ask_text(host, cached, &preamble)?;
            floor.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        floor.sort_by(f64::total_cmp);
        let _ = writeln!(
            out,
            "{:>7}  {:>8.1}  {:>8.1}  {:>8.1}  {:>8.1}  {:>7}",
            "none",
            percentile(&floor, 50.0),
            percentile(&floor, 95.0),
            floor.first().copied().unwrap_or(0.0),
            floor.last().copied().unwrap_or(0.0),
            "-"
        );

        for width in WIDTHS {
            if width > image.width {
                continue;
            }
            let scaled = image.resample(width);
            let png = scaled.to_png()?;

            // One request outside the measurement, to load whatever the first
            // one loads and to warm the cache when it is on. Including it makes
            // the first sample an outlier that says nothing about a steady tick.
            let _ = ask(host, &png, cached, &preamble)?;

            let mut samples = Vec::with_capacity(requests);
            let mut tokens = 0;
            for index in 0..requests {
                // A fresh image per request, which is what a real tick sends:
                // the scene moved. Perturbing a handful of pixels is enough to
                // make the encoded image differ, which is all the cache cares
                // about.
                let bytes = if fresh {
                    scaled.perturbed(index).to_png()?
                } else {
                    png.clone()
                };
                let started = Instant::now();
                let answer = ask(host, &bytes, cached, &preamble)?;
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
                tokens = answer;
            }
            samples.sort_by(f64::total_cmp);

            let _ = writeln!(
                out,
                "{width:>7}  {:>8.1}  {:>8.1}  {:>8.1}  {:>8.1}  {tokens:>7}",
                percentile(&samples, 50.0),
                percentile(&samples, 95.0),
                samples.first().copied().unwrap_or(0.0),
                samples.last().copied().unwrap_or(0.0),
            );
        }
        let _ = writeln!(out);
    }

    Ok(out)
}

/// Nearest-rank, because interpolating between fifty samples invents precision.
fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (percentile / 100.0 * sorted.len() as f64).ceil() as usize;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

/// One request shaped like a tactical tick, returning its prompt token count.
fn ask(host: &str, png: &[u8], cached: bool, preamble: &str) -> Result<u64, String> {
    let data = client::base64(png);
    let request = serde_json::json!({
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": format!("{preamble}{PROMPT}")},
                {"type": "image_url",
                 "image_url": {"url": format!("data:image/png;base64,{data}")}}
            ]
        }],
        "grammar": GRAMMAR,
        "temperature": 0.0,
        // A tactical answer is one short tool call. Measuring a long generation
        // would measure decoding, and decoding is not what a tick spends.
        "max_tokens": 24,
        // Measured both ways, because neither alone is the product's tick.
        // Off is the worst case and the honest floor. On is what the product
        // actually does: FR-CTX-002 makes blocks 1 and 2 byte-identical
        // between calls precisely so this cache hits.
        "cache_prompt": cached,
    });

    let body = client::post_json(host, "/v1/chat/completions", &request.to_string())?;
    let response: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| format!("response is not JSON: {error}"))?;
    response
        .pointer("/usage/prompt_tokens")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "no token count in the response".to_owned())
}

/// The same request with no image, for the floor.
fn ask_text(host: &str, cached: bool, preamble: &str) -> Result<(), String> {
    let request = serde_json::json!({
        "messages": [{"role": "user", "content": format!("{preamble}{PROMPT}")}],
        "grammar": GRAMMAR,
        "temperature": 0.0,
        "max_tokens": 24,
        "cache_prompt": cached,
    });
    client::post_json(host, "/v1/chat/completions", &request.to_string()).map(|_| ())
}

/// The question, shaped like the one a tactical tick asks.
const PROMPT: &str = "You are playing this game. Looking at the screen, choose the single next \
                      action. Answer with one tool call and nothing else.";

/// A grammar of the shape `ADR-0013` describes: a closed set of tools with
/// typed parameters, so an answer outside it cannot be produced.
const GRAMMAR: &str = "root ::= look-call | wait-call\n\
                       look-call ::= \"{\\\"tool\\\":\\\"look\\\",\\\"dx\\\":\" signed \"}\"\n\
                       wait-call ::= \"{\\\"tool\\\":\\\"wait\\\"}\"\n\
                       signed ::= \"-\"? [0-9]+\n";

/// A decoded image.
struct Image {
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Image {
    /// Resample to a given width, keeping the aspect ratio. Nearest neighbour,
    /// because the question is what an image of this size costs, not how good
    /// the resampling is.
    fn resample(&self, width: usize) -> Self {
        let height = (self.height * width / self.width).max(1);
        let mut pixels = vec![0_u8; width * height * 3];
        for y in 0..height {
            let source_y = y * self.height / height;
            for x in 0..width {
                let source_x = x * self.width / width;
                let from = (source_y * self.width + source_x) * 3;
                let to = (y * width + x) * 3;
                pixels[to..to + 3].copy_from_slice(&self.pixels[from..from + 3]);
            }
        }
        Self {
            pixels,
            width,
            height,
        }
    }

    /// The same image with a few pixels changed, so it encodes differently.
    ///
    /// A real tactical tick sends a frame that moved. Reusing one image fifty
    /// times measures a cache hit on the image, which no tick ever gets.
    fn perturbed(&self, step: usize) -> Self {
        let mut pixels = self.pixels.clone();
        for offset in 0..16 {
            let index = (step * 997 + offset * 31) % (pixels.len() / 3) * 3;
            pixels[index] = pixels[index].wrapping_add(97);
        }
        Self {
            pixels,
            width: self.width,
            height: self.height,
        }
    }

    fn to_png(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(
                &mut out,
                u32::try_from(self.width).map_err(|_| "width does not fit".to_owned())?,
                u32::try_from(self.height).map_err(|_| "height does not fit".to_owned())?,
            );
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
            writer
                .write_image_data(&self.pixels)
                .map_err(|error| error.to_string())?;
        }
        Ok(out)
    }
}

/// Read a PNG written by the capture probe.
fn load(path: &str) -> Result<Image, String> {
    let file = std::fs::File::open(path).map_err(|error| format!("{path}: {error}"))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("{path}: {error}"))?;
    let mut buffer = vec![0; reader.output_buffer_size().unwrap_or(0)];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| format!("{path}: {error}"))?;

    let width = info.width as usize;
    let height = info.height as usize;
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("{path}: unsupported colour type {other:?}")),
    };

    let mut pixels = Vec::with_capacity(width * height * 3);
    for chunk in buffer[..info.buffer_size()].chunks_exact(channels) {
        pixels.extend_from_slice(&chunk[..3]);
    }
    Ok(Image {
        pixels,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_percentile_is_nearest_rank() {
        let samples: Vec<f64> = (1..=100).map(f64::from).collect();
        assert!((percentile(&samples, 50.0) - 50.0).abs() < 1e-9);
        assert!((percentile(&samples, 95.0) - 95.0).abs() < 1e-9);
        assert!((percentile(&samples, 100.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn an_empty_sample_has_no_percentile() {
        assert!((percentile(&[], 95.0)).abs() < 1e-9);
    }

    #[test]
    fn resampling_keeps_the_aspect_ratio() {
        let image = Image {
            pixels: vec![0; 1280 * 720 * 3],
            width: 1280,
            height: 720,
        };
        let small = image.resample(256);
        assert_eq!(small.width, 256);
        assert_eq!(small.height, 144);
        assert_eq!(small.pixels.len(), 256 * 144 * 3);
    }

    #[test]
    fn the_grammar_admits_only_the_two_tools() {
        // The same property ADR-0013 relies on: an answer outside the grammar
        // cannot be produced, so a malformed tool call is not a failure mode
        // the measurement has to handle.
        assert!(GRAMMAR.contains("root ::= look-call | wait-call"));
        assert!(!GRAMMAR.contains("string"));
    }
}
