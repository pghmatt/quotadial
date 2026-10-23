//! The tray icon, drawn in code so it can show the meter. On macOS and Linux it is a gauge whose
//! needle points at the share used, with the number beside it as the tray title. On Windows, where
//! a tray icon cannot have text beside it, it is the number itself, drawn into the icon.

use crate::meter::Level;

/// An RGBA image, not premultiplied, as the tray takes it.
#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    pub rgba: Vec<u8>,
    pub size: u32,
    /// Whether macOS should tint it to match the menu bar, using only its alpha.
    pub template: bool,
}

type Rgb = [u8; 3];

/// macOS's system colours, readable on light and dark bars alike.
const GREY: Rgb = [0x8e, 0x8e, 0x93];
const AMBER: Rgb = [0xff, 0x9f, 0x0a];
const RED: Rgb = [0xff, 0x45, 0x3a];
const BLACK: Rgb = [0, 0, 0];
const WHITE: Rgb = [0xff, 0xff, 0xff];
/// Darker backgrounds for the Windows number, so white digits stay readable.
const BADGE_GREY: Rgb = [0x48, 0x48, 0x4a];
const BADGE_AMBER: Rgb = [0xb2, 0x5f, 0x00];
const BADGE_RED: Rgb = [0xc4, 0x1e, 0x13];

/// Samples per pixel along each axis, for smooth edges.
const SAMPLES: u32 = 4;

/// The arc runs from 210° (lower left) through the top to -30° (lower right).
const ARC_START: f64 = 210.0;
const ARC_SWEEP: f64 = 240.0;

/// A gauge `size` pixels square. As a template it is black, for macOS to tint; otherwise it is
/// grey, amber or red by level.
#[must_use]
pub fn gauge(size: u32, percent: Option<u8>, level: Level, template: bool) -> Icon {
    let colour = if template {
        BLACK
    } else {
        match level {
            Level::Unknown | Level::Normal => GREY,
            Level::Warning => AMBER,
            Level::High => RED,
        }
    };
    let needle = percent
        .map(|percent| (ARC_START - ARC_SWEEP * f64::from(percent.min(100)) / 100.0).to_radians());
    let rgba = draw(size, |x, y| {
        // In units of the icon's size, y up, centred a little low so the gauge sits in the middle.
        let (dx, dy) = (x - 0.5, 0.56 - y);
        let distance = dx.hypot(dy);
        let angle = dy.atan2(dx);
        let mut alpha: f64 = 0.0;
        // The arc: the part used so far solid, the rest faint.
        if (0.30..=0.44).contains(&distance) && on_arc(angle) {
            let used =
                needle.is_some_and(|needle| sweep_from_start(angle) <= sweep_from_start(needle));
            alpha = alpha.max(if used { 1.0 } else { 0.35 });
        }
        if let Some(needle) = needle {
            let tip = (0.33 * needle.cos(), 0.33 * needle.sin());
            if distance_to_segment((dx, dy), tip) <= 0.045 {
                alpha = 1.0;
            }
        }
        // The hub.
        if distance <= 0.08 {
            alpha = 1.0;
        }
        (colour, alpha)
    });
    Icon {
        rgba,
        size,
        template,
    }
}

/// The share used as white digits on a rounded square coloured by level, `size` pixels square, or
/// a dash before there are numbers.
#[must_use]
pub fn number(size: u32, percent: Option<u8>, level: Level) -> Icon {
    let background = match level {
        Level::Unknown | Level::Normal => BADGE_GREY,
        Level::Warning => BADGE_AMBER,
        Level::High => BADGE_RED,
    };
    let text = percent.map_or_else(|| "-".to_owned(), |percent| percent.min(100).to_string());
    let segments = digit_segments(&text);
    let rgba = draw(size, |x, y| {
        if !in_rounded_square(x, y, 0.2) {
            return (background, 0.0);
        }
        if segments.iter().any(|segment| segment.contains(x, y)) {
            (WHITE, 1.0)
        } else {
            (background, 1.0)
        }
    });
    Icon {
        rgba,
        size,
        template: false,
    }
}

/// Renders `shade`, which gives the colour and coverage at a point in units of the icon's size,
/// averaging several samples per pixel.
fn draw(size: u32, shade: impl Fn(f64, f64) -> (Rgb, f64)) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let step = 1.0 / f64::from(size * SAMPLES);
    for row in 0..size {
        for column in 0..size {
            let (mut colour, mut alpha) = ([0.0_f64; 3], 0.0);
            for sample_y in 0..SAMPLES {
                for sample_x in 0..SAMPLES {
                    let x = (f64::from(column * SAMPLES + sample_x) + 0.5) * step;
                    let y = (f64::from(row * SAMPLES + sample_y) + 0.5) * step;
                    let (rgb, coverage) = shade(x, y);
                    for (sum, channel) in colour.iter_mut().zip(rgb) {
                        *sum += f64::from(channel) * coverage;
                    }
                    alpha += coverage;
                }
            }
            let samples = f64::from(SAMPLES * SAMPLES);
            for sum in colour {
                // The colour of the covered samples only, so edges keep their hue.
                rgba.push(if alpha > 0.0 {
                    (sum / alpha).round() as u8
                } else {
                    0
                });
            }
            rgba.push((alpha / samples * 255.0).round() as u8);
        }
    }
    rgba
}

/// How far round the arc an angle is, from its start, in degrees.
fn sweep_from_start(angle: f64) -> f64 {
    (ARC_START - angle.to_degrees()).rem_euclid(360.0)
}

fn on_arc(angle: f64) -> bool {
    sweep_from_start(angle) <= ARC_SWEEP
}

/// The distance from `point` to the segment from the origin to `tip`.
fn distance_to_segment(point: (f64, f64), tip: (f64, f64)) -> f64 {
    let length_squared = tip.0 * tip.0 + tip.1 * tip.1;
    let along = ((point.0 * tip.0 + point.1 * tip.1) / length_squared).clamp(0.0, 1.0);
    (point.0 - along * tip.0).hypot(point.1 - along * tip.1)
}

fn in_rounded_square(x: f64, y: f64, radius: f64) -> bool {
    let nearest_x = x.clamp(radius, 1.0 - radius);
    let nearest_y = y.clamp(radius, 1.0 - radius);
    (x - nearest_x).hypot(y - nearest_y) <= radius
}

/// An axis-aligned rectangle in units of the icon's size.
#[derive(Debug, Clone, Copy)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Rect {
    fn contains(&self, x: f64, y: f64) -> bool {
        (self.left..=self.right).contains(&x) && (self.top..=self.bottom).contains(&y)
    }
}

/// The lit segments of `text`, digits and dashes drawn like a seven-segment display and centred.
fn digit_segments(text: &str) -> Vec<Rect> {
    let count = text.chars().count().max(1) as f64;
    // Three digits need narrower cells than one or two.
    let (width, height, stroke, gap) = if count > 2.0 {
        (0.24, 0.48, 0.07, 0.05)
    } else {
        (0.30, 0.56, 0.09, 0.08)
    };
    let total = count * width + (count - 1.0) * gap;
    let top = (1.0 - height) / 2.0;
    let mut lit = Vec::new();
    for (index, character) in text.chars().enumerate() {
        let left = (1.0 - total) / 2.0 + index as f64 * (width + gap);
        let (right, bottom, middle) = (left + width, top + height, top + height / 2.0);
        let half = stroke / 2.0;
        // a top, b upper right, c lower right, d bottom, e lower left, f upper left, g middle.
        let segment = |name: char| match name {
            'a' => Rect {
                left,
                top,
                right,
                bottom: top + stroke,
            },
            'b' => Rect {
                left: right - stroke,
                top,
                right,
                bottom: middle + half,
            },
            'c' => Rect {
                left: right - stroke,
                top: middle - half,
                right,
                bottom,
            },
            'd' => Rect {
                left,
                top: bottom - stroke,
                right,
                bottom,
            },
            'e' => Rect {
                left,
                top: middle - half,
                right: left + stroke,
                bottom,
            },
            'f' => Rect {
                left,
                top,
                right: left + stroke,
                bottom: middle + half,
            },
            _ => Rect {
                left,
                top: middle - half,
                right,
                bottom: middle + half,
            },
        };
        let names = match character {
            '0' => "abcdef",
            '1' => "bc",
            '2' => "abged",
            '3' => "abgcd",
            '4' => "fgbc",
            '5' => "afgcd",
            '6' => "afgedc",
            '7' => "abc",
            '8' => "abcdefg",
            '9' => "abcdfg",
            _ => "g",
        };
        lit.extend(names.chars().map(segment));
    }
    lit
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pixel at `(x, y)`, in units of the icon's size.
    fn pixel(icon: &Icon, x: f64, y: f64) -> [u8; 4] {
        let size = f64::from(icon.size);
        let (column, row) = ((x * size) as usize, (y * size) as usize);
        let at = (row * icon.size as usize + column) * 4;
        [
            icon.rgba[at],
            icon.rgba[at + 1],
            icon.rgba[at + 2],
            icon.rgba[at + 3],
        ]
    }

    /// The point `radius` from the gauge's centre at `degrees` (0° right, 90° up).
    fn polar(radius: f64, degrees: f64) -> (f64, f64) {
        let angle = degrees.to_radians();
        (0.5 + radius * angle.cos(), 0.56 - radius * angle.sin())
    }

    #[test]
    fn the_needle_points_at_the_share_used() {
        // 0% points to the arc's start at 210°, 50% straight up, 100% to its end at -30°.
        for (percent, degrees) in [(0, 210.0), (50, 90.0), (100, -30.0)] {
            let icon = gauge(64, Some(percent), Level::Normal, true);
            let (x, y) = polar(0.24, degrees);
            assert!(pixel(&icon, x, y)[3] > 200, "{percent}% at {degrees}°");
            let (x, y) = polar(0.24, degrees + 180.0);
            assert!(pixel(&icon, x, y)[3] < 50, "{percent}%: nothing opposite");
        }
    }

    #[test]
    fn the_used_part_of_the_arc_is_solid_and_the_rest_faint() {
        let icon = gauge(64, Some(50), Level::Normal, true);

        let (x, y) = polar(0.37, 150.0);
        assert!(pixel(&icon, x, y)[3] > 230, "used");
        let (x, y) = polar(0.37, 30.0);
        let faint = pixel(&icon, x, y)[3];
        assert!((60..=120).contains(&faint), "left: {faint}");
        let (x, y) = polar(0.37, 270.0);
        assert_eq!(pixel(&icon, x, y)[3], 0, "the gap at the bottom");
    }

    #[test]
    fn a_template_is_black_and_the_levels_have_colours() {
        let template = gauge(36, Some(95), Level::High, true);
        assert!(template.template);
        assert!(template.rgba.chunks(4).all(|pixel| pixel[..3] == [0, 0, 0]));

        let (x, y) = polar(0.0, 0.0);
        for (level, colour) in [
            (Level::Normal, GREY),
            (Level::Warning, AMBER),
            (Level::High, RED),
        ] {
            let icon = gauge(36, Some(95), level, false);
            assert!(!icon.template);
            assert_eq!(pixel(&icon, x, y)[..3], colour, "{level:?}");
        }
        assert_eq!(template.rgba.len(), 36 * 36 * 4);
    }

    #[test]
    fn without_numbers_the_gauge_has_no_needle() {
        let icon = gauge(64, None, Level::Unknown, true);

        let (x, y) = polar(0.24, 90.0);
        assert!(pixel(&icon, x, y)[3] < 50);
        let (x, y) = polar(0.37, 90.0);
        assert!(pixel(&icon, x, y)[3] > 60, "the faint arc");
    }

    /// Whether the segment at a cell's left edge, upper half, is lit in a one- or two-digit number.
    fn upper_left_lit(icon: &Icon, digit_index: usize, digits: usize) -> bool {
        let (width, gap) = (0.30, 0.08);
        let total = digits as f64 * width + (digits as f64 - 1.0) * gap;
        let left = (1.0 - total) / 2.0 + digit_index as f64 * (width + gap);
        pixel(icon, left + 0.03, 0.36)[..3] == WHITE
    }

    #[test]
    fn windows_numbers_light_the_right_segments() {
        let eighteen = number(64, Some(18), Level::Normal);
        assert!(!upper_left_lit(&eighteen, 0, 2), "1 has no left segments");
        assert!(upper_left_lit(&eighteen, 1, 2), "8 has every segment");
        assert_eq!(pixel(&eighteen, 0.5, 0.06)[..3], BADGE_GREY);
        assert_eq!(pixel(&eighteen, 0.0, 0.0)[3], 0, "rounded corners");

        let high = number(64, Some(100), Level::High);
        assert_eq!(pixel(&high, 0.5, 0.06)[..3], BADGE_RED);
        assert_eq!(digit_segments("100").len(), 2 + 6 + 6);
        assert_eq!(digit_segments("-").len(), 1);
        assert_eq!(number(32, None, Level::Unknown).rgba.len(), 32 * 32 * 4);
    }
}
