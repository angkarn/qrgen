//! Generate QR code images with text layouts from templates and data.
//!
//! Layouts are [textplate](https://github.com/angkarn/textplate) templates with an extra
//! `qr` node type. [`Generator`] renders a template once per data row; [`Row`] provides the
//! `{{1}}`, `{{ROW}}`, `{{data}}` and header-name variables.
//!
//! ```no_run
//! use qrgen::{Canvas, Generator, QrStyle, Row};
//! use textplate::{Assets, Fonts, Template, TemplateString};
//!
//! let mut fonts = Fonts::new();
//! fonts.add("default", qrgen::DEFAULT_FONT)?;
//! let template = Template::from_json5(r#"[
//!   { type: "qr", height: "85%" },
//!   { type: "text", y: "85%", height: "15%", text: "{{1}}", size: "5vw", align: "center", valign: "middle" },
//! ]"#)?;
//! let generator = Generator::new(fonts, template, Canvas::default(), QrStyle::default());
//!
//! let columns = vec!["https://example.com".to_string()];
//! let row = Row::new(0, &columns).with_data(&TemplateString::parse("{{1}}")).unwrap();
//! let output = generator.render(&row, &Assets::new())?;
//! std::fs::write("qr.png", qrgen::encode_png(&output.image)?)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod generator;
#[cfg(feature = "decode")]
mod job;
mod output;
mod qr;
mod row;

pub use error::{Error, Result};
pub use generator::{default_template, Canvas, Generator, Output};
#[cfg(feature = "decode")]
pub use job::{Batch, FileSource, FontSpec, Format, Job, OutputSpec, TemplateSource};
pub use output::{encode_png, sanitize_file_name, UniqueNames};
pub use qr::{draw_qr, encode, to_terminal, Ecc, QrElement, QrStyle};
pub use row::Row;
pub use textplate;

/// Poppins Regular (Latin), embedded with the `default-font` feature.
#[cfg(feature = "default-font")]
pub static DEFAULT_FONT: &[u8] = include_bytes!("../assets/Poppins-Regular.ttf");
