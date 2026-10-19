use std::collections::HashSet;

use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ImageEncoder,
};
use textplate::image::RgbaImage;

use crate::Result;

/// Encodes an image as PNG.
///
/// Uses fast compression with a fixed Paeth filter: about twice as fast as the adaptive
/// filter for files a few percent larger.
pub fn encode_png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    PngEncoder::new_with_quality(&mut bytes, CompressionType::Fast, FilterType::Paeth)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )?;
    Ok(bytes)
}

/// Makes `name` safe to use as a file name on Linux, macOS and Windows.
///
/// Path separators, reserved and control characters become `_`, leading/trailing dots and
/// spaces are removed, and Windows device names (`CON`, `COM1`, ...) get a `_` prefix.
pub fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || r#"<>:"/\|?*"#.contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if trimmed.is_empty() {
        return "_".to_string();
    }
    let stem = trimmed.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        format!("_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Hands out unique file names: `a`, `a_2`, `a_3`, ... (case-insensitive, for macOS/Windows).
#[derive(Default)]
pub struct UniqueNames {
    seen: HashSet<String>,
}

impl UniqueNames {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a sanitized, unique `name` + `extension` (e.g. `".png"`).
    pub fn next(&mut self, name: &str, extension: &str) -> String {
        let base = sanitize_file_name(name);
        let mut n = 1;
        loop {
            let candidate = if n == 1 {
                format!("{base}{extension}")
            } else {
                format!("{base}_{n}{extension}")
            };
            if self.seen.insert(candidate.to_lowercase()) {
                return candidate;
            }
            n += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes() {
        assert_eq!(sanitize_file_name("a/b\\c:d"), "a_b_c_d");
        assert_eq!(sanitize_file_name(".."), "_");
        assert_eq!(sanitize_file_name("  .hidden. "), "hidden");
        assert_eq!(sanitize_file_name("con"), "_con");
        assert_eq!(sanitize_file_name("COM3.txt"), "_COM3.txt");
        assert_eq!(sanitize_file_name("COMPANY"), "COMPANY");
        assert_eq!(sanitize_file_name("สวัสดี 😀"), "สวัสดี 😀");
    }

    #[test]
    fn unique_names() {
        let mut names = UniqueNames::new();
        assert_eq!(names.next("a", ".png"), "a.png");
        assert_eq!(names.next("A", ".png"), "A_2.png");
        assert_eq!(names.next("a", ".png"), "a_3.png");
        assert_eq!(names.next("a_2", ".png"), "a_2_2.png");
    }

    #[test]
    fn png_round_trip() {
        let img = RgbaImage::from_fn(31, 17, |x, y| {
            image::Rgba([x as u8 * 8, y as u8 * 15, 7, 200])
        });
        let bytes = encode_png(&img).unwrap();
        assert_eq!(image::load_from_memory(&bytes).unwrap().into_rgba8(), img);
    }
}
