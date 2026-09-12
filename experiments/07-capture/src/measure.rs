//! What a capture route is measured on.
//!
//! The matrix asks which route works. That is the cheap half of the answer, and
//! it is not the half `FR-PERC-009` needs. That requirement asks the system to
//! tell an unsupported window mode apart from a transient failure, and no amount
//! of reasoning produces the signatures that distinguish them — only a route run
//! against a mode it cannot handle, with the failure written down.
//!
//! So every cell of the comparison records six things, and four of them are
//! about failure:
//!
//! | Reading | Why it is here |
//! | --- | --- |
//! | Achieved frames per second | The budget in `15-performance-budgets.md` is per tick |
//! | Frame latency, median and 95th | A mean hides the stalls that cost the game frames |
//! | Black-frame rate | The characteristic failure of a route on the wrong window mode: it succeeds and returns nothing |
//! | Featureless-frame rate | A frame that is uniform but not black — a route returning the desktop background, or a window that has not painted |
//! | Identical-frame rate | A route that succeeds and returns the same bytes for ever, which is a stale surface rather than a still scene |
//! | Cursor present | `FR-PERC-006`: the cursor in the frame is the agent's own pointer, and perception must not mistake it for an element |
//!
//! The last three are the ones that catch a route being quietly useless.
//! A route that returns black is obvious. A route that returns the same correct
//! frame for ever is not, and it is the failure that would send a month of work
//! to the perception layer.

use std::time::Duration;

/// One frame, as any route returns it.
pub(crate) struct Frame {
    /// Row-major, three bytes per pixel, top row first.
    pub(crate) pixels: Vec<u8>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    /// How long the route took to produce it.
    pub(crate) elapsed: Duration,
}

impl Frame {
    /// A frame is black when no channel of any pixel exceeds this.
    ///
    /// Not zero. A capture of a window that has not painted often returns a
    /// near-black surface with a few units of noise, and treating that as "not
    /// black" would report a working route.
    const BLACK: u8 = 8;

    /// True when every pixel is at or below [`Self::BLACK`].
    pub(crate) fn is_black(&self) -> bool {
        self.pixels.iter().all(|&value| value <= Self::BLACK)
    }

    /// True when the frame carries almost no variation.
    ///
    /// Separate from black on purpose. A route that returns the desktop
    /// background, or a window mid-resize, produces a uniform frame that is not
    /// black, and reporting it as a success is how a route looks fine in a
    /// summary and useless in practice.
    pub(crate) fn is_featureless(&self) -> bool {
        let Some(&first) = self.pixels.first() else {
            return true;
        };
        let mut low = first;
        let mut high = first;
        for &value in &self.pixels {
            low = low.min(value);
            high = high.max(value);
        }
        u16::from(high) - u16::from(low) < 12
    }

    /// How densely [`Self::digest`] samples the frame.
    ///
    /// Sampled rather than complete, because hashing eleven megabytes thirty
    /// times a second would reduce the achieved frame rate this harness exists
    /// to measure — the digest is outside the latency timer but not outside the
    /// wall clock.
    ///
    /// **The stride is the blind spot, so it is small and it is written down.**
    /// At 61 bytes, any change covering 21 pixels or more is certain to be
    /// sampled. A smaller one can be missed, and the test named after this
    /// records that rather than leaving it to be rediscovered.
    ///
    /// This started at 997 and a test caught it: a genuinely changing scene was
    /// reported as a stale surface, which is the detector producing its own
    /// worst failure.
    const STRIDE: usize = 61;

    /// A content hash, for spotting a surface that never updates.
    pub(crate) fn digest(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for (index, value) in self.pixels.iter().enumerate().step_by(Self::STRIDE) {
            // The index is mixed in as well as the value, so that two bytes
            // swapping places changes the hash. Without it a scrolling frame
            // can hash identically to the one before it.
            hash ^= u64::from(*value) ^ (index as u64);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        hash ^= (self.pixels.len() as u64).wrapping_mul(0x9e37_79b9);
        hash
    }
}

/// What one route scored against one window.
#[derive(Default)]
pub(crate) struct Tally {
    pub(crate) attempted: usize,
    pub(crate) succeeded: usize,
    pub(crate) black: usize,
    pub(crate) featureless: usize,
    /// Frames whose content was identical to the frame before.
    pub(crate) repeated: usize,
    /// Every frame's latency, in microseconds, in arrival order.
    pub(crate) latencies: Vec<u64>,
    /// The first error, if any. Later ones are counted, not kept.
    pub(crate) first_error: Option<String>,
    pub(crate) errors: usize,
    /// How long the whole run took, wall clock.
    pub(crate) wall: Duration,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Tally {
    /// Fold one successful frame in.
    pub(crate) fn record(&mut self, frame: &Frame, previous: Option<u64>) -> u64 {
        self.attempted += 1;
        self.succeeded += 1;
        self.width = frame.width;
        self.height = frame.height;
        self.latencies.push(frame.elapsed.as_micros() as u64);
        if frame.is_black() {
            self.black += 1;
        } else if frame.is_featureless() {
            self.featureless += 1;
        }
        let digest = frame.digest();
        if previous == Some(digest) {
            self.repeated += 1;
        }
        digest
    }

    /// Fold one failure in.
    pub(crate) fn record_error(&mut self, error: String) {
        self.attempted += 1;
        self.errors += 1;
        if self.first_error.is_none() {
            self.first_error = Some(error);
        }
    }

    /// Frames per second actually achieved over the run.
    pub(crate) fn fps(&self) -> f64 {
        let seconds = self.wall.as_secs_f64();
        if seconds <= 0.0 {
            0.0
        } else {
            self.succeeded as f64 / seconds
        }
    }

    /// Latency at a percentile, in milliseconds.
    ///
    /// Nearest-rank, because the samples are few enough that interpolating
    /// between them invents precision the measurement does not have.
    pub(crate) fn latency_ms(&self, percentile: f64) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort_unstable();
        let rank = (percentile / 100.0 * sorted.len() as f64).ceil() as usize;
        let index = rank.saturating_sub(1).min(sorted.len() - 1);
        sorted[index] as f64 / 1000.0
    }

    /// A share as a percentage of attempted frames.
    pub(crate) fn share(&self, count: usize) -> f64 {
        if self.attempted == 0 {
            0.0
        } else {
            count as f64 * 100.0 / self.attempted as f64
        }
    }

    /// The verdict, in the vocabulary `FR-PERC-009` needs.
    ///
    /// Deliberately more than "worked" and "failed". A route that never
    /// produces a frame and a route that produces the same frame for ever are
    /// both failures, and sending them to the same remedy wastes a month.
    /// Below this many frames, no verdict is claimed.
    ///
    /// The compositor route delivers on change rather than on a clock, so a
    /// static window legitimately yields one or two frames in ten seconds.
    /// Calling that "usable" on the strength of a single frame is the same
    /// error as calling it broken: neither is supported by two samples.
    const ENOUGH: usize = 5;

    pub(crate) fn verdict(&self) -> &'static str {
        if self.succeeded == 0 {
            "no frames"
        } else if self.succeeded < Self::ENOUGH {
            "too few frames"
        } else if self.share(self.black) > 50.0 {
            "black frames"
        } else if self.share(self.featureless) > 50.0 {
            "featureless"
        } else if self.share(self.repeated) > 90.0 {
            "stale surface"
        } else if self.errors > 0 {
            "intermittent"
        } else {
            "usable"
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    fn frame(pixels: Vec<u8>) -> Frame {
        let width = pixels.len() / 3;
        Frame {
            pixels,
            width,
            height: 1,
            elapsed: Duration::from_millis(1),
        }
    }

    #[test]
    fn near_black_counts_as_black() {
        // A window that has not painted returns a near-black surface with a
        // few units of noise. Requiring exact zero reports that as a working
        // route, which is the failure this threshold exists to catch.
        assert!(frame(vec![0, 3, 7, 2, 0, 8]).is_black());
        assert!(!frame(vec![0, 3, 7, 2, 0, 9]).is_black());
    }

    #[test]
    fn a_uniform_frame_is_featureless_without_being_black() {
        let grey = frame(vec![128; 300]);
        assert!(!grey.is_black());
        assert!(grey.is_featureless());
    }

    #[test]
    fn a_varied_frame_is_neither() {
        let mut pixels = vec![0_u8; 300];
        for (index, value) in pixels.iter_mut().enumerate() {
            *value = (index % 256) as u8;
        }
        let varied = frame(pixels);
        assert!(!varied.is_black());
        assert!(!varied.is_featureless());
    }

    #[test]
    fn a_stale_surface_is_not_reported_as_usable() {
        // The failure that matters most here: every frame succeeds, every
        // frame is correct, and nothing ever changes. A verdict of "usable"
        // would send a month of work to the perception layer.
        let mut tally = Tally {
            wall: Duration::from_secs(1),
            ..Tally::default()
        };
        let mut previous = None;
        for _ in 0..30 {
            let mut pixels = vec![0_u8; 3000];
            pixels[7] = 200;
            previous = Some(tally.record(&frame(pixels), previous));
        }
        assert_eq!(tally.verdict(), "stale surface");
    }

    #[test]
    fn a_changing_scene_is_usable() {
        let mut tally = Tally {
            wall: Duration::from_secs(1),
            ..Tally::default()
        };
        let mut previous = None;
        // A region rather than two bytes, because that is what a scene change
        // looks like: the smallest real one is a caret or a hovered button.
        for step in 0..30_usize {
            let mut pixels = vec![0_u8; 3000];
            for offset in 0..300 {
                pixels[600 + offset] = ((step * 7 + offset) % 200) as u8;
            }
            previous = Some(tally.record(&frame(pixels), previous));
        }
        assert_eq!(tally.verdict(), "usable");
    }

    #[test]
    fn the_digest_samples_and_can_miss_a_change_smaller_than_its_stride() {
        // Written to record the blind spot rather than to defend it. A change
        // of a few bytes between two sample points hashes identically, so a
        // scene whose only motion is smaller than 21 pixels reads as stale.
        //
        // That is a real limitation of a sampled digest, and it is why the
        // stride is 61 rather than the 997 this started at: at 997 the fixture
        // above failed, and a genuinely changing scene was reported as a stale
        // surface — the detector's own worst failure, produced by the detector.
        let mut base = vec![0_u8; 3000];
        base[7] = 1;
        let mut nudged = base.clone();
        nudged[8] = 250;
        assert_eq!(frame(base).digest(), frame(nudged).digest());
    }

    #[test]
    fn a_handful_of_frames_earns_no_verdict() {
        // The compositor route delivers on change, so a static window yields
        // one or two frames in ten seconds. Reading that as "usable" is the
        // same error as reading it as broken, and the first run of this probe
        // did exactly that on a single frame.
        let mut tally = Tally {
            wall: Duration::from_secs(8),
            ..Tally::default()
        };
        tally.record(&frame(vec![7, 200, 30, 90, 12, 250]), None);
        assert_eq!(tally.verdict(), "too few frames");
    }

    #[test]
    fn no_frames_outranks_every_other_verdict() {
        let mut tally = Tally::default();
        tally.record_error("the window has no client area".to_owned());
        assert_eq!(tally.verdict(), "no frames");
    }

    #[test]
    fn the_percentile_is_nearest_rank() {
        let tally = Tally {
            latencies: (1..=100).map(|n| n * 1000).collect(),
            ..Tally::default()
        };
        // 1000 microseconds is 1 ms, so the nth sample is n ms.
        assert!((tally.latency_ms(50.0) - 50.0).abs() < 1e-9);
        assert!((tally.latency_ms(95.0) - 95.0).abs() < 1e-9);
        assert!((tally.latency_ms(100.0) - 100.0).abs() < 1e-9);
    }
}
