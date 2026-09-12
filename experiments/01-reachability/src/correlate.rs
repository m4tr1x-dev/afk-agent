//! Detect horizontal image displacement between two frames, without a model.
//!
//! The known-good matrix says question 1's experiment "confirms the camera
//! turned". That is not a method, and doing it by eye would make the answer to
//! the project's gating question a matter of opinion. This module is the
//! method.
//!
//! A small yaw rotation translates the projected image almost uniformly
//! sideways. So: reduce each frame to a one-dimensional profile of column
//! intensities, and cross-correlate the two profiles. The lag at which
//! correlation peaks is how far the world moved.
//!
//! Two details do the real work.
//!
//! **The central band only.** Interface elements are pinned to the edges of the
//! screen and do not move when the camera does. Including them anchors the
//! correlation at zero lag and hides exactly the signal being measured.
//!
//! **Normalised correlation.** A scene that brightens between frames — a muzzle
//! flash, a light source coming into view — changes every column's magnitude.
//! Normalising makes the peak depend on the shape of the profile rather than
//! its scale.

/// A frame reduced to what the correlation needs.
#[derive(Debug, Clone)]
pub(crate) struct Profile {
    values: Vec<f64>,
}

/// Fraction of the frame height kept. Interface elements live above and below.
const BAND_HEIGHT: f64 = 0.60;

/// Fraction of the frame width kept.
const BAND_WIDTH: f64 = 0.80;

/// Widest displacement searched, as a fraction of the profile length. A lag
/// beyond this is not a camera turn; it is a scene change.
const MAX_LAG: f64 = 0.25;

/// Correlation scores within this of each other count as equal, so that the
/// tie-break towards the smallest displacement can take effect.
const TIE: f64 = 1e-9;

impl Profile {
    /// Reduce a grayscale frame to the central band's column intensities.
    ///
    /// `pixels` is row-major, one byte per pixel, `width * height` long.
    pub(crate) fn from_gray(pixels: &[u8], width: usize, height: usize) -> Option<Self> {
        if width == 0 || height == 0 || pixels.len() < width * height {
            return None;
        }

        let band_h = ((height as f64) * BAND_HEIGHT) as usize;
        let band_w = ((width as f64) * BAND_WIDTH) as usize;
        if band_h == 0 || band_w == 0 {
            return None;
        }
        let top = (height - band_h) / 2;
        let left = (width - band_w) / 2;

        let mut values = vec![0.0_f64; band_w];
        for y in top..top + band_h {
            let row = &pixels[y * width..y * width + width];
            for (index, value) in values.iter_mut().enumerate() {
                *value += f64::from(row[left + index]);
            }
        }
        let inverse = 1.0 / band_h as f64;
        for value in &mut values {
            *value *= inverse;
        }
        Some(Self { values })
    }

    /// Number of columns in the profile.
    ///
    /// The caller reads a lag as a fraction of this, because a displacement of
    /// twenty columns means something different on a 640-wide capture than on a
    /// 3840-wide one.
    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    /// Is this frame uniform enough that correlating it is meaningless?
    ///
    /// A black frame is the most common first capture failure, and a black
    /// frame correlates perfectly with another black frame at every lag. Saying
    /// "the camera did not turn" on that evidence would be wrong in the most
    /// misleading direction, so the caller is told the capture failed instead.
    pub(crate) fn is_featureless(&self) -> bool {
        let mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
        let variance = self
            .values
            .iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f64>()
            / self.values.len() as f64;
        variance < 1.0
    }
}

/// The best horizontal alignment between two profiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Displacement {
    /// Columns the second profile moved relative to the first. Positive means
    /// the image moved right, which a leftward camera turn produces.
    pub(crate) lag: i32,
    /// Normalised correlation at that lag, in `-1.0..=1.0`.
    pub(crate) confidence: f64,
}

/// Cross-correlate two profiles and return the best alignment.
///
/// Returns `None` when the profiles differ in length or either is degenerate.
pub(crate) fn displacement(before: &Profile, after: &Profile) -> Option<Displacement> {
    let a = &before.values;
    let b = &after.values;
    if a.len() != b.len() || a.len() < 16 {
        return None;
    }

    let max_lag = ((a.len() as f64) * MAX_LAG) as i32;
    let mut best = Displacement {
        lag: 0,
        confidence: -1.0,
    };

    // Ties are resolved towards the smallest displacement, and that is not a
    // tidiness choice. A periodic texture — a tiled floor, a railing, a row of
    // windows — correlates just as well at every multiple of its period, so a
    // scene can offer several equally good answers. The probe emits small
    // per-tick deltas, so the true displacement is the small one; picking the
    // largest would turn an ordinary wall into a reported 90-degree turn.
    for lag in -max_lag..=max_lag {
        let Some(score) = correlation_at(a, b, lag) else {
            continue;
        };
        let better = score > best.confidence + TIE;
        let equal_but_closer = (score - best.confidence).abs() <= TIE && lag.abs() < best.lag.abs();
        if better || equal_but_closer {
            best = Displacement {
                lag,
                confidence: score,
            };
        }
    }

    if best.confidence <= -1.0 {
        None
    } else {
        Some(best)
    }
}

/// Normalised cross-correlation of the overlapping region at one lag.
fn correlation_at(a: &[f64], b: &[f64], lag: i32) -> Option<f64> {
    let len = a.len() as i32;
    let (start_a, start_b) = if lag >= 0 { (lag, 0) } else { (0, -lag) };
    let overlap = len - lag.abs();
    // Fewer than half the columns in common is not evidence of anything.
    if overlap < len / 2 {
        return None;
    }

    let slice_a = &a[start_a as usize..(start_a + overlap) as usize];
    let slice_b = &b[start_b as usize..(start_b + overlap) as usize];
    let count = f64::from(overlap);

    let mean_a = slice_a.iter().sum::<f64>() / count;
    let mean_b = slice_b.iter().sum::<f64>() / count;

    let mut covariance = 0.0;
    let mut variance_a = 0.0;
    let mut variance_b = 0.0;
    for (x, y) in slice_a.iter().zip(slice_b) {
        let dx = x - mean_a;
        let dy = y - mean_b;
        covariance += dx * dy;
        variance_a += dx * dx;
        variance_b += dy * dy;
    }

    let denominator = (variance_a * variance_b).sqrt();
    if denominator < f64::EPSILON {
        return None;
    }
    Some(covariance / denominator)
}

#[cfg(test)]
mod tests {
    // Narrow scope, and only here. The coding standards forbid crate-wide
    // allows, and rightly: a panic in the reflex loop is a defect that reaches
    // a user. Inside a test a panic is the reporting mechanism, and writing
    // every assertion as a match would obscure what is being asserted.
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    /// A frame with a non-repeating horizontal pattern, shifted by `offset`.
    ///
    /// Deliberately aperiodic. A repeating pattern correlates equally well at
    /// every multiple of its period, which is a real property of real scenes
    /// and is tested separately — but it is not what these tests are about.
    fn textured(width: usize, height: usize, offset: i32) -> Vec<u8> {
        let mut pixels = vec![0_u8; width * height];
        for y in 0..height {
            for x in 0..width {
                // Signed, and not wrapped into the frame: the pattern is
                // aperiodic, so wrapping a negative offset would produce a
                // large positive shift instead of a small negative one.
                let column = (x as i32).wrapping_add(offset);
                let hashed = (column as u32).wrapping_mul(2_654_435_761) >> 24;
                pixels[y * width + x] = hashed as u8;
            }
        }
        pixels
    }

    /// A frame with vertical stripes of period 32, shifted by `offset`.
    fn striped(width: usize, height: usize, offset: usize) -> Vec<u8> {
        let mut pixels = vec![0_u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let phase = (x + offset) % 32;
                pixels[y * width + x] = if phase < 16 { 40 } else { 210 };
            }
        }
        pixels
    }

    #[test]
    fn identical_frames_have_zero_displacement() {
        // COVERS: FR-ACT-004
        let pixels = textured(320, 180, 0);
        let a = Profile::from_gray(&pixels, 320, 180).expect("profile");
        let b = a.clone();
        let found = displacement(&a, &b).expect("displacement");
        assert_eq!(found.lag, 0);
        assert!(found.confidence > 0.99, "confidence {}", found.confidence);
    }

    #[test]
    fn a_shifted_frame_reports_its_shift() {
        // COVERS: FR-ACT-004
        for shift in [4_i32, 8, 12, -8, -20] {
            let a = Profile::from_gray(&textured(320, 180, 0), 320, 180).expect("profile");
            let b = Profile::from_gray(&textured(320, 180, shift), 320, 180).expect("profile");
            let found = displacement(&a, &b).expect("displacement");
            // The second frame samples the pattern `shift` columns further
            // along, so its content is the first frame's slid left by `shift`,
            // and the profiles align at lag `+shift`.
            //
            // Sign is not a detail here. Reversing the input must reverse the
            // lag, and that is one of the four conjunct tests separating a real
            // camera turn from a scene that happened to move.
            assert_eq!(found.lag, shift, "shift {shift} produced lag {}", found.lag);
            assert!(found.confidence > 0.95, "confidence {}", found.confidence);
        }
    }

    #[test]
    fn a_blank_frame_is_recognised_as_featureless() {
        // COVERS: FR-PERC-009
        //
        // A black frame is the most common first capture failure, and two black
        // frames correlate perfectly at every lag. Reporting "the camera did
        // not turn" on that evidence would be wrong in the most misleading
        // direction.
        let blank = vec![0_u8; 320 * 180];
        let profile = Profile::from_gray(&blank, 320, 180).expect("profile");
        assert!(profile.is_featureless());
    }

    #[test]
    fn a_textured_frame_is_not_featureless() {
        // COVERS: FR-PERC-009
        let profile = Profile::from_gray(&textured(320, 180, 0), 320, 180).expect("profile");
        assert!(!profile.is_featureless());
    }

    #[test]
    fn brightness_change_does_not_move_the_peak() {
        // COVERS: FR-ACT-004
        //
        // A scene that brightens between frames changes every column's
        // magnitude. Normalised correlation must depend on the shape of the
        // profile rather than its scale, or a muzzle flash reads as a turn.
        let base = textured(320, 180, 0);
        let brighter: Vec<u8> = base.iter().map(|v| v.saturating_add(20)).collect();
        let a = Profile::from_gray(&base, 320, 180).expect("profile");
        let b = Profile::from_gray(&brighter, 320, 180).expect("profile");
        let found = displacement(&a, &b).expect("displacement");
        assert_eq!(found.lag, 0);
        assert!(found.confidence > 0.99, "confidence {}", found.confidence);
    }

    #[test]
    fn a_repeating_pattern_resolves_to_the_smallest_displacement() {
        // COVERS: FR-ACT-004
        //
        // A tiled floor, a railing, a row of windows: real scenes contain
        // periodic texture, and a periodic profile correlates equally well at
        // every multiple of its period. The probe emits small per-tick deltas,
        // so the true answer is the small one, and reporting the largest would
        // turn an ordinary wall into a 90-degree turn.
        let a = Profile::from_gray(&striped(320, 180, 0), 320, 180).expect("profile");
        let b = Profile::from_gray(&striped(320, 180, 32), 320, 180).expect("profile");
        let found = displacement(&a, &b).expect("displacement");
        assert_eq!(
            found.lag, 0,
            "a full-period shift is indistinguishable from none"
        );
    }

    #[test]
    fn a_degenerate_profile_is_rejected_rather_than_guessed_at() {
        // COVERS: FR-PERC-009
        assert!(Profile::from_gray(&[], 0, 0).is_none());
        assert!(Profile::from_gray(&[1, 2, 3], 100, 100).is_none());
    }
}
