use std::{fmt, str::FromStr};

use qrcodegen::{QrCode, QrCodeEcc};
use serde::Deserialize;
use textplate::{image::RgbaImage, paint, BoxError, Color, Element, ElementContext, Rect};

use crate::{Error, Result};

/// QR error correction level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecc {
    /// ~7% of the code can be damaged.
    #[serde(alias = "l")]
    Low,
    /// ~15%
    #[default]
    #[serde(alias = "m")]
    Medium,
    /// ~25%
    #[serde(alias = "q")]
    Quartile,
    /// ~30%
    #[serde(alias = "h")]
    High,
}

impl Ecc {
    fn to_qrcodegen(self) -> QrCodeEcc {
        match self {
            Ecc::Low => QrCodeEcc::Low,
            Ecc::Medium => QrCodeEcc::Medium,
            Ecc::Quartile => QrCodeEcc::Quartile,
            Ecc::High => QrCodeEcc::High,
        }
    }
}

impl FromStr for Ecc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "l" | "low" => Ok(Ecc::Low),
            "m" | "medium" => Ok(Ecc::Medium),
            "q" | "quartile" => Ok(Ecc::Quartile),
            "h" | "high" => Ok(Ecc::High),
            _ => Err(Error::Invalid(format!(
                "error correction level {s:?}: expected l, m, q or h"
            ))),
        }
    }
}

impl fmt::Display for Ecc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Ecc::Low => "l",
            Ecc::Medium => "m",
            Ecc::Quartile => "q",
            Ecc::High => "h",
        })
    }
}

/// How a QR code looks.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct QrStyle {
    pub ecc: Ecc,
    /// Color of dark modules.
    pub dark: Color,
    /// Color of light modules and the quiet zone.
    pub light: Color,
    /// Light border around the code, in modules.
    pub quiet_zone: u32,
}

impl Default for QrStyle {
    fn default() -> Self {
        Self {
            ecc: Ecc::Medium,
            dark: Color::BLACK,
            light: Color::WHITE,
            quiet_zone: 1,
        }
    }
}

/// Encodes `data` into a QR module matrix.
pub fn encode(data: &str, ecc: Ecc) -> Result<QrCode> {
    QrCode::encode_text(data, ecc.to_qrcodegen()).map_err(|_| Error::DataTooLong {
        len: data.len(),
        ecc,
    })
}

/// Draws a QR code for `data`, as large as fits in `rect` and centered in it.
///
/// Modules are whole pixels, so the code is crisp; any leftover space stays untouched.
pub fn draw_qr(canvas: &mut RgbaImage, rect: Rect, data: &str, style: &QrStyle) -> Result<()> {
    let code = encode(data, style.ecc)?;
    let total = code.size() as u32 + 2 * style.quiet_zone;
    let side = rect.width.min(rect.height).max(0.0);
    let module = (side / total as f32).floor() as u32;
    if module == 0 {
        return Err(Error::QrTooSmall {
            needed: total,
            width: rect.width,
            height: rect.height,
        });
    }
    let px = (module * total) as f32;
    let x0 = (rect.x + (rect.width - px) / 2.0).floor();
    let y0 = (rect.y + (rect.height - px) / 2.0).floor();
    let quiet = style.quiet_zone as i32;
    let dark_at = |mx: u32, my: u32| code.get_module(mx as i32 - quiet, my as i32 - quiet);

    // One rectangle per horizontal run of same-colored modules
    let m = module as f32;
    for my in 0..total {
        let mut mx = 0;
        while mx < total {
            let dark = dark_at(mx, my);
            let start = mx;
            while mx < total && dark_at(mx, my) == dark {
                mx += 1;
            }
            let color = if dark { style.dark } else { style.light };
            let run = Rect::new(
                x0 + start as f32 * m,
                y0 + my as f32 * m,
                (mx - start) as f32 * m,
                m,
            );
            paint::fill_rect(canvas, run, color);
        }
    }
    Ok(())
}

/// Renders QR codes in the terminal with half-block characters (two modules per character).
///
/// By default light modules are drawn as filled blocks, which suits dark terminal themes;
/// set `invert` for light themes.
pub fn to_terminal(data: &str, ecc: Ecc, quiet_zone: u32, invert: bool) -> Result<String> {
    let code = encode(data, ecc)?;
    let quiet = quiet_zone as i32;
    let total = code.size() + 2 * quiet;
    let light = |x: i32, y: i32| y < total && !code.get_module(x - quiet, y - quiet) != invert;
    let mut out = String::new();
    for y in (0..total).step_by(2) {
        for x in 0..total {
            out.push(match (light(x, y), light(x, y + 1)) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }
        out.push('\n');
    }
    Ok(out)
}

/// The `qr` template element:
///
/// ```json5
/// { type: "qr", data: "{{url}}", ecc: "h", dark: "#1f2937", light: "#fff0", quiet_zone: 2 }
/// ```
///
/// Unset fields use the defaults the element was created with.
pub struct QrElement {
    pub defaults: QrStyle,
}

impl Element for QrElement {
    fn draw(&self, cx: &mut ElementContext<'_>) -> Result<(), BoxError> {
        let data: String = cx.prop("data")?.ok_or("missing `data`")?;
        let ecc = match cx.prop::<String>("ecc")? {
            Some(s) => s.parse()?,
            None => self.defaults.ecc,
        };
        let color = |name: &str, default: Color| -> Result<Color, BoxError> {
            Ok(match cx.prop::<String>(name)? {
                Some(s) => s.parse()?,
                None => default,
            })
        };
        let style = QrStyle {
            ecc,
            dark: color("dark", self.defaults.dark)?,
            light: color("light", self.defaults.light)?,
            quiet_zone: cx.prop("quiet_zone")?.unwrap_or(self.defaults.quiet_zone),
        };
        for key in cx.props.keys() {
            if !["data", "ecc", "dark", "light", "quiet_zone"].contains(&key.as_str()) {
                return Err(format!("unknown field `{key}`").into());
            }
        }
        draw_qr(cx.canvas, cx.rect, &data, &style)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ecc() {
        assert_eq!("H".parse::<Ecc>().unwrap(), Ecc::High);
        assert_eq!("quartile".parse::<Ecc>().unwrap(), Ecc::Quartile);
        assert!("x".parse::<Ecc>().is_err());
    }

    #[test]
    fn terminal_output_is_square() {
        let s = to_terminal("hello", Ecc::Low, 1, false).unwrap();
        let lines: Vec<_> = s.lines().collect();
        // version 1 = 21 modules + 2 quiet = 23 columns, 12 lines of two rows
        assert_eq!(lines.len(), 12);
        assert!(lines.iter().all(|l| l.chars().count() == 23));
        assert!(lines[0].starts_with('█'), "quiet zone is light");
    }

    #[test]
    fn rejects_data_that_does_not_fit() {
        let long = "x".repeat(5000);
        assert!(matches!(
            encode(&long, Ecc::High),
            Err(Error::DataTooLong { .. })
        ));
    }

    #[test]
    fn too_small_box() {
        let mut img = RgbaImage::new(10, 10);
        let err = draw_qr(
            &mut img,
            Rect::new(0.0, 0.0, 10.0, 10.0),
            "hello",
            &QrStyle::default(),
        );
        assert!(matches!(err, Err(Error::QrTooSmall { needed: 23, .. })));
    }
}
