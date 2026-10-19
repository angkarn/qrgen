use std::path::PathBuf;

use qrgen::{Canvas, QrStyle};
use serde::Deserialize;
use serde_json::Value;

/// Everything needed for a run. Built from CLI flags or read from a job file (`qrgen run`).
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Job {
    /// QR data. With `csv` it is a template (default `{{1}}`); otherwise the literal data.
    pub data: Option<String>,
    /// CSV file: one image per row.
    pub csv: Option<PathBuf>,
    /// The CSV's first row names the columns (`{{name}}`) and is not rendered.
    pub header: bool,
    /// Layout template: a path to a JSON5 file or the template itself.
    pub template: Option<TemplateSource>,
    pub canvas: Canvas,
    /// Fonts in order; the first is the default text font.
    pub fonts: Vec<FontSpec>,
    pub qr: QrStyle,
    pub output: OutputSpec,
    /// Fail when text does not fit its box (default: warn).
    pub strict: bool,
    /// Stop at the first failed row.
    pub fail_fast: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TemplateSource {
    Path(PathBuf),
    Inline(Value),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontSpec {
    pub name: String,
    pub path: PathBuf,
}

impl std::str::FromStr for FontSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let (name, path) = s.split_once('=').ok_or("expected NAME=PATH")?;
        if name.is_empty() || path.is_empty() {
            return Err("expected NAME=PATH".into());
        }
        Ok(Self {
            name: name.to_string(),
            path: path.into(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OutputSpec {
    /// Single image: output file (`-` for stdout). Default `qr.png`.
    pub file: Option<PathBuf>,
    /// CSV rows: output directory. Default `output`.
    pub dir: Option<PathBuf>,
    /// CSV rows: file name template, without extension. Default `{{ROW}}`.
    pub name: Option<String>,
    pub format: Format,
}

impl Default for OutputSpec {
    fn default() -> Self {
        Self {
            file: None,
            dir: None,
            name: None,
            format: Format::Png,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// Write PNG files
    #[default]
    Png,
    /// Print base64-encoded PNGs to stdout
    Base64,
}
