//! Synthetic images the model cannot have memorised.
//!
//! "It loaded" is not the question. A vision tower wired to the wrong projector
//! returns fluent, plausible text about the general image — which is exactly the
//! quiet failure the known-good matrix warns about for a version mismatch. The
//! assertion has to be discriminative, meaning a broken projector has to score
//! near chance rather than near right.
//!
//! So each scene carries three independent facts:
//!
//! | Fact | Values | Chance |
//! | --- | --- | --- |
//! | A three-digit number | 100 to 999 | 1 in 900 |
//! | A named background colour | six | 1 in 6 |
//! | A shape | four | 1 in 4 |
//!
//! All three correct by luck is one in 21,600. Seven of eight scenes correct on
//! all three fields is not something a broken projector reaches.
//!
//! The number is drawn at run time rather than fixed, so a scene cannot have
//! been seen during training and cannot be memorised between runs of this probe.

use std::fmt::Write as _;

/// The six background colours, and the words the model is allowed to answer.
///
/// Deliberately far apart in hue. The question under test is whether the image
/// reaches the model at all, not whether it can tell teal from turquoise, and a
/// near-miss palette would turn a clear result into an argument.
pub(crate) const COLOURS: [(&str, [u8; 3]); 6] = [
    ("red", [200, 30, 30]),
    ("green", [30, 150, 60]),
    ("blue", [40, 70, 200]),
    ("yellow", [220, 200, 40]),
    ("purple", [120, 40, 160]),
    ("grey", [128, 128, 128]),
];

/// The four shapes.
pub(crate) const SHAPES: [&str; 4] = ["circle", "square", "triangle", "cross"];

/// Image dimensions. Large enough that the digits survive whatever rescaling
/// the runtime applies, small enough that eight of them encode quickly.
pub(crate) const WIDTH: usize = 512;
pub(crate) const HEIGHT: usize = 512;

/// One generated scene and the three answers it expects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scene {
    /// The three-digit number drawn across the middle.
    pub(crate) number: u16,
    /// Index into [`COLOURS`].
    pub(crate) colour: usize,
    /// Index into [`SHAPES`].
    pub(crate) shape: usize,
}

impl Scene {
    /// The expected answer, as the grammar constrains the model to write it.
    #[must_use]
    pub(crate) fn expected(&self) -> (String, &'static str, &'static str) {
        (
            self.number.to_string(),
            COLOURS[self.colour].0,
            SHAPES[self.shape],
        )
    }

    /// A one-line description for the transcript.
    #[must_use]
    pub(crate) fn describe(&self) -> String {
        let mut out = String::new();
        let _ = write!(
            out,
            "{} on {} with a {}",
            self.number, COLOURS[self.colour].0, SHAPES[self.shape]
        );
        out
    }

    /// Render the scene to an RGB buffer.
    #[must_use]
    pub(crate) fn render(&self) -> Vec<u8> {
        let background = COLOURS[self.colour].1;
        let mut pixels = vec![0_u8; WIDTH * HEIGHT * 3];
        for chunk in pixels.chunks_mut(3) {
            chunk.copy_from_slice(&background);
        }

        // Ink is chosen for contrast against the background rather than fixed,
        // because white digits on yellow are a legibility test nobody asked for.
        let luminance = f64::from(background[0]).mul_add(
            0.299,
            f64::from(background[1]).mul_add(0.587, f64::from(background[2]) * 0.114),
        );
        let ink: [u8; 3] = if luminance > 140.0 {
            [20, 20, 20]
        } else {
            [245, 245, 245]
        };

        draw_shape(&mut pixels, self.shape, ink);
        draw_number(&mut pixels, self.number, ink);
        pixels
    }
}

/// Draw the shape in the upper half.
fn draw_shape(pixels: &mut [u8], shape: usize, ink: [u8; 3]) {
    let centre_x = WIDTH as isize / 2;
    let centre_y = HEIGHT as isize / 4;
    let radius = 80_isize;

    for y in (centre_y - radius)..=(centre_y + radius) {
        for x in (centre_x - radius)..=(centre_x + radius) {
            let dx = x - centre_x;
            let dy = y - centre_y;
            let inside = match shape {
                // circle
                0 => dx * dx + dy * dy <= radius * radius,
                // square
                1 => dx.abs() <= radius && dy.abs() <= radius,
                // triangle, apex up
                2 => {
                    let height = dy + radius;
                    height >= 0 && height <= 2 * radius && dx.abs() * 2 <= height
                }
                // cross
                _ => dx.abs() <= radius / 3 || dy.abs() <= radius / 3,
            };
            if inside {
                put(pixels, x, y, ink);
            }
        }
    }
}

/// Draw the three digits across the lower half, in a seven-segment face.
///
/// Hand-drawn rather than rendered from a font. A font would be a dependency,
/// and a seven-segment digit is unambiguous at any scale the runtime picks.
fn draw_number(pixels: &mut [u8], number: u16, ink: [u8; 3]) {
    let digits = [number / 100 % 10, number / 10 % 10, number % 10];
    let cell_width = 110_isize;
    let start_x = WIDTH as isize / 2 - cell_width * 3 / 2 + 15;
    let top = HEIGHT as isize * 5 / 8;

    for (index, digit) in digits.iter().enumerate() {
        let x = start_x + cell_width * index as isize;
        draw_digit(pixels, *digit as usize, x, top, ink);
    }
}

/// Segment layout, clockwise from the top, then the middle bar.
const SEGMENTS: [[bool; 7]; 10] = [
    [true, true, true, true, true, true, false],     // 0
    [false, true, true, false, false, false, false], // 1
    [true, true, false, true, true, false, true],    // 2
    [true, true, true, true, false, false, true],    // 3
    [false, true, true, false, false, true, true],   // 4
    [true, false, true, true, false, true, true],    // 5
    [true, false, true, true, true, true, true],     // 6
    [true, true, true, false, false, false, false],  // 7
    [true, true, true, true, true, true, true],      // 8
    [true, true, true, true, false, true, true],     // 9
];

fn draw_digit(pixels: &mut [u8], digit: usize, x: isize, y: isize, ink: [u8; 3]) {
    let width = 70_isize;
    let height = 140_isize;
    let thickness = 16_isize;
    let on = SEGMENTS[digit.min(9)];

    // top, top-right, bottom-right, bottom, bottom-left, top-left, middle
    let bars: [(isize, isize, isize, isize); 7] = [
        (x, y, width, thickness),
        (x + width - thickness, y, thickness, height / 2),
        (x + width - thickness, y + height / 2, thickness, height / 2),
        (x, y + height - thickness, width, thickness),
        (x, y + height / 2, thickness, height / 2),
        (x, y, thickness, height / 2),
        (x, y + height / 2 - thickness / 2, width, thickness),
    ];

    for (index, (bx, by, bw, bh)) in bars.iter().enumerate() {
        if !on[index] {
            continue;
        }
        for py in *by..(by + bh) {
            for px in *bx..(bx + bw) {
                put(pixels, px, py, ink);
            }
        }
    }
}

fn put(pixels: &mut [u8], x: isize, y: isize, ink: [u8; 3]) {
    if x < 0 || y < 0 || x >= WIDTH as isize || y >= HEIGHT as isize {
        return;
    }
    let offset = (y as usize * WIDTH + x as usize) * 3;
    pixels[offset..offset + 3].copy_from_slice(&ink);
}

/// Resample an RGB buffer to a square of `side` pixels, nearest neighbour.
///
/// Nearest neighbour on purpose. The budget question is how the runtime prices
/// an image of a given size, and a smoothing filter would change the content as
/// well as the dimensions, leaving two variables where there should be one.
#[must_use]
pub(crate) fn resample(pixels: &[u8], side: usize) -> Vec<u8> {
    let mut out = vec![0_u8; side * side * 3];
    for y in 0..side {
        let source_y = y * HEIGHT / side;
        for x in 0..side {
            let source_x = x * WIDTH / side;
            let from = (source_y * WIDTH + source_x) * 3;
            let to = (y * side + x) * 3;
            out[to..to + 3].copy_from_slice(&pixels[from..from + 3]);
        }
    }
    out
}

/// Encode an RGB buffer of a given square size as a PNG.
///
/// # Errors
///
/// Returns the encoder's error if the buffer cannot be written.
pub(crate) fn to_png_sized(pixels: &[u8], side: usize) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let side = u32::try_from(side).map_err(|_| "side does not fit in u32".to_owned())?;
        let mut encoder = png::Encoder::new(&mut out, side, side);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(pixels).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Encode an RGB buffer as a PNG.
///
/// # Errors
///
/// Returns the encoder's error if the buffer cannot be written.
pub(crate) fn to_png(pixels: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, WIDTH as u32, HEIGHT as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(pixels).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scene_renders_at_the_declared_size() {
        let scene = Scene {
            number: 407,
            colour: 2,
            shape: 1,
        };
        assert_eq!(scene.render().len(), WIDTH * HEIGHT * 3);
    }

    #[test]
    fn the_background_is_the_colour_the_scene_names() {
        // Corner pixels are never drawn on, so they carry the background.
        let scene = Scene {
            number: 123,
            colour: 0,
            shape: 0,
        };
        let pixels = scene.render();
        assert_eq!(&pixels[0..3], &COLOURS[0].1);
    }

    #[test]
    fn ink_flips_for_a_light_background() {
        // Dark ink on yellow, light ink on blue. A fixed ink colour would make
        // one of the six palettes a legibility test rather than a vision test.
        let yellow = Scene {
            number: 111,
            colour: 3,
            shape: 0,
        }
        .render();
        let blue = Scene {
            number: 111,
            colour: 2,
            shape: 0,
        }
        .render();
        let centre = (HEIGHT / 4 * WIDTH + WIDTH / 2) * 3;
        assert!(yellow[centre] < 64, "dark ink expected on yellow");
        assert!(blue[centre] > 192, "light ink expected on blue");
    }

    #[test]
    fn every_digit_has_a_distinct_segment_pattern() {
        // A duplicated row would make two digits indistinguishable, and the
        // model would be marked wrong for reading the image correctly.
        for (a, first) in SEGMENTS.iter().enumerate() {
            for (b, second) in SEGMENTS.iter().enumerate().skip(a + 1) {
                assert_ne!(first, second, "digits {a} and {b} render alike");
            }
        }
    }

    #[test]
    fn a_scene_encodes_as_a_png() {
        let scene = Scene {
            number: 999,
            colour: 5,
            shape: 3,
        };
        let png = to_png(&scene.render()).expect("the encoder accepts the buffer");
        assert_eq!(&png[1..4], b"PNG");
    }
}
