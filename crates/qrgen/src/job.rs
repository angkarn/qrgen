use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Deserialize;
use serde_json::Value;
use textplate::{image::RgbaImage, Assets, Fonts, Options, Template, TemplateString};

use crate::{
    default_template, encode_image, Canvas, Error, Format, Generator, Output, QrStyle, Result, Row,
    UniqueNames, DEFAULT_JPEG_QUALITY,
};

/// Reads the files a [`Job`] refers to (template, fonts, CSV, images) by the path written in it.
///
/// The CLI reads from disk; the browser build reads from files the page provides.
pub trait FileSource {
    fn read(&self, path: &str) -> std::result::Result<Vec<u8>, String>;
}

/// A complete description of what to generate, as written in a JSON5 job file.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Job {
    /// QR data. With `csv` it is a template (default `{{1}}`); otherwise the literal data.
    pub data: Option<String>,
    /// CSV file: one image per row. Without it, one image is generated.
    pub csv: Option<String>,
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

impl Job {
    pub fn from_json5(src: &str) -> Result<Self> {
        json5::from_str(src).map_err(|e| Error::Job(format!("job: {e}")))
    }

    /// `true` when the job makes one image (no CSV).
    pub fn is_single(&self) -> bool {
        self.csv.is_none()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TemplateSource {
    Path(String),
    Inline(Value),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontSpec {
    pub name: String,
    pub path: String,
}

impl std::str::FromStr for FontSpec {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s.split_once('=') {
            Some((name, path)) if !name.is_empty() && !path.is_empty() => Ok(Self {
                name: name.to_string(),
                path: path.to_string(),
            }),
            _ => Err("expected NAME=PATH".into()),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OutputSpec {
    /// Single image: output file (`-` for stdout in the CLI). Default `qr.png` / `qr.jpg`.
    pub file: Option<String>,
    /// CSV rows: output directory. Default `output`.
    pub dir: Option<String>,
    /// CSV rows: file name template, without extension. Default `{{ROW}}`.
    pub name: Option<String>,
    /// `png` (default) or `jpeg`.
    pub format: Format,
    /// JPEG quality, 1-100. Default 85.
    pub quality: Option<u8>,
    /// CLI: print base64-encoded images to stdout instead of writing files.
    pub base64: bool,
}

/// A job loaded and ready to render, one image per row.
///
/// `Batch` is `Send + Sync`: render rows in parallel with [`Batch::render`].
pub struct Batch {
    generator: Generator,
    headers: Option<Vec<String>>,
    records: Vec<Vec<String>>,
    data: RowData,
    names: Vec<std::result::Result<String, String>>,
    format: Format,
    quality: u8,
    /// Images used by several rows, decoded once.
    assets: Assets,
    /// Images used by one row, decoded when that row is rendered.
    encoded: HashMap<String, Vec<u8>>,
    asset_errors: HashMap<String, String>,
}

enum RowData {
    Literal(String),
    Template(TemplateString),
}

impl Batch {
    /// Loads the template, fonts, CSV and images the job needs.
    ///
    /// Problems that affect only some rows (a missing image or column) are reported by
    /// [`Batch::render`] for those rows; problems with the job itself fail here.
    pub fn new(job: &Job, files: &dyn FileSource) -> Result<Self> {
        let read = |kind: &str, path: &str| {
            files
                .read(path)
                .map_err(|e| Error::Job(format!("cannot read {kind} {path}: {e}")))
        };

        let template = match &job.template {
            None => default_template(),
            Some(TemplateSource::Path(path)) => {
                let bytes = read("template", path)?;
                let src = String::from_utf8(bytes)
                    .map_err(|_| Error::Job(format!("template {path} is not UTF-8")))?;
                Template::from_json5(&src)?
            }
            Some(TemplateSource::Inline(value)) => Template::from_value(value.clone())?,
        };

        let quality = job.output.quality.unwrap_or(DEFAULT_JPEG_QUALITY);
        if !(1..=100).contains(&quality) {
            return Err(Error::Job(format!(
                "output.quality must be 1-100, not {quality}"
            )));
        }

        let mut fonts = Fonts::new();
        for font in &job.fonts {
            fonts.add(&font.name, read("font", &font.path)?)?;
        }
        #[cfg(feature = "default-font")]
        if fonts.family("default").is_none() {
            fonts.add("default", crate::DEFAULT_FONT)?;
        }

        let options = Options {
            strict: job.strict,
            ..Options::default()
        };
        let generator = Generator::new(fonts, template, job.canvas.clone(), job.qr.clone())
            .with_options(options);

        let (headers, records) = match &job.csv {
            Some(path) => read_csv(&read("CSV", path)?, job.header, path)?,
            None => (None, vec![Vec::new()]),
        };
        // A single image uses the data as-is; CSV rows fill it as a template
        let data = match (&job.data, job.is_single()) {
            (data, true) => RowData::Literal(data.clone().unwrap_or_default()),
            (Some(data), false) => RowData::Template(TemplateString::parse(data)),
            (None, false) => RowData::Template(TemplateString::parse("{{1}}")),
        };
        let mut batch = Self {
            generator,
            headers,
            records,
            data,
            names: Vec::new(),
            format: job.output.format,
            quality,
            assets: Assets::new(),
            encoded: HashMap::new(),
            asset_errors: HashMap::new(),
        };

        // File names, decided up front so they are unique and stable
        let name_template = TemplateString::parse(job.output.name.as_deref().unwrap_or("{{ROW}}"));
        let extension = format!(".{}", job.output.format.extension());
        let mut unique = UniqueNames::new();
        batch.names = (0..batch.len())
            .map(|i| {
                if job.is_single() {
                    return Ok(job
                        .output
                        .file
                        .clone()
                        .unwrap_or_else(|| format!("qr{extension}")));
                }
                let row = batch.row(i)?;
                let name = name_template
                    .render(&row)
                    .map_err(|n| format!("missing variable {n:?} in output name"))?;
                Ok(unique.next(&name, &extension))
            })
            .collect();

        // Images used by several rows (a logo, a shared background) are decoded once here. The
        // others stay encoded until their row renders, so a batch of photos is decoded in parallel
        // and never held in memory all at once.
        let mut uses = BTreeMap::<String, usize>::new();
        for i in 0..batch.len() {
            if let Ok(row) = batch.row(i) {
                let names: BTreeSet<_> = batch
                    .generator
                    .asset_names(&row)
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                for name in names {
                    *uses.entry(name).or_default() += 1;
                }
            }
        }
        for (name, count) in uses {
            let bytes = match files.read(&name) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let message = format!("cannot read image {name}: {e}");
                    batch.asset_errors.insert(name, message);
                    continue;
                }
            };
            if count == 1 {
                batch.encoded.insert(name, bytes);
                continue;
            }
            match decode(&name, &bytes) {
                Ok(img) => {
                    batch.assets.insert(name, img);
                }
                Err(e) => {
                    batch.asset_errors.insert(name, e);
                }
            }
        }
        Ok(batch)
    }

    /// Number of images: CSV rows, or 1.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Unique, file-system-safe name of row `i` (e.g. `card_1.png`), or why it has none.
    pub fn name(&self, i: usize) -> std::result::Result<&str, String> {
        self.names[i].as_deref().map_err(String::clone)
    }

    /// Variables of row `i`, with `{{data}}` filled in.
    pub fn row(&self, i: usize) -> std::result::Result<Row<'_>, String> {
        let mut row = Row::new(i, &self.records[i]);
        if let Some(h) = &self.headers {
            row = row.with_headers(h);
        }
        match &self.data {
            RowData::Literal(data) => {
                row.data = Some(data.clone());
                Ok(row)
            }
            RowData::Template(t) => row
                .with_data(t)
                .map_err(|name| format!("missing variable {name:?} in data")),
        }
    }

    /// Renders row `i`.
    pub fn render(&self, i: usize) -> Result<Output> {
        self.names[i].as_ref().map_err(|e| Error::Data(e.clone()))?;
        let row = self.row(i).map_err(Error::Data)?;
        let mut row_assets = None;
        for name in self.generator.asset_names(&row)? {
            if let Some(e) = self.asset_errors.get(&name) {
                return Err(Error::Data(e.clone()));
            }
            if let Some(bytes) = self.encoded.get(&name) {
                let img = decode(&name, bytes).map_err(Error::Data)?;
                row_assets
                    .get_or_insert_with(|| self.assets.clone())
                    .insert(name, img);
            }
        }
        self.generator
            .render(&row, row_assets.as_ref().unwrap_or(&self.assets))
    }

    /// Encodes a rendered row in the job's format (`output.format`, `output.quality`).
    pub fn encode(&self, output: &Output) -> Result<Vec<u8>> {
        encode_image(&output.image, self.format, self.quality)
    }

    /// Image format of the job's output files.
    pub fn format(&self) -> Format {
        self.format
    }

    pub fn generator(&self) -> &Generator {
        &self.generator
    }
}

fn decode(name: &str, bytes: &[u8]) -> std::result::Result<RgbaImage, String> {
    textplate::image::load_from_memory(bytes)
        .map(|img| img.into_rgba8())
        .map_err(|e| format!("cannot decode image {name}: {e}"))
}

type Records = (Option<Vec<String>>, Vec<Vec<String>>);

fn read_csv(bytes: &[u8], header: bool, path: &str) -> Result<Records> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(header)
        .flexible(true)
        .from_reader(bytes);
    let headers = if header {
        let h = reader
            .headers()
            .map_err(|e| Error::Job(format!("CSV {path}: {e}")))?;
        Some(h.iter().map(str::to_string).collect())
    } else {
        None
    };
    let mut records = Vec::new();
    for (i, record) in reader.records().enumerate() {
        let record = record.map_err(|e| Error::Job(format!("CSV {path} line {}: {e}", i + 1)))?;
        records.push(record.iter().map(str::to_string).collect());
    }
    if records.is_empty() {
        return Err(Error::Job(format!("CSV {path} has no rows")));
    }
    Ok((headers, records))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Files(HashMap<&'static str, Vec<u8>>);

    impl FileSource for Files {
        fn read(&self, path: &str) -> std::result::Result<Vec<u8>, String> {
            self.0.get(path).cloned().ok_or_else(|| "not found".into())
        }
    }

    #[test]
    fn plans_rows_names_and_errors() {
        let files = Files(HashMap::from([(
            "people.csv",
            b"name,id\nAnn,1\nAnn,2\nBob\n".to_vec(),
        )]));
        let job = Job::from_json5(
            r#"{ csv: "people.csv", header: true, data: "id:{{id}}", output: { name: "{{name}}" } }"#,
        )
        .unwrap();
        let batch = Batch::new(&job, &files).unwrap();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch.name(0).unwrap(), "Ann.png");
        assert_eq!(batch.name(1).unwrap(), "Ann_2.png");
        assert_eq!(batch.row(1).unwrap().data.as_deref(), Some("id:2"));
        assert!(batch.render(0).is_ok());
        assert_eq!(
            batch.encode(&batch.render(0).unwrap()).unwrap()[..4],
            *b"\x89PNG"
        );
        let err = batch.render(2).err().unwrap().to_string();
        assert!(err.contains("missing variable \"id\""), "{err}");
    }

    #[test]
    fn jpeg_output() {
        let files = Files(HashMap::from([("rows.csv", b"a\nb\n".to_vec())]));
        let job = Job::from_json5(
            r#"{ csv: "rows.csv", output: { format: "jpeg", quality: 90 }, canvas: { width: 200 } }"#,
        )
        .unwrap();
        let batch = Batch::new(&job, &files).unwrap();
        assert_eq!(batch.name(0).unwrap(), "1.jpg");
        let jpeg = batch.encode(&batch.render(0).unwrap()).unwrap();
        let gray = textplate::image::load_from_memory(&jpeg)
            .unwrap()
            .into_luma8();
        let mut prepared = rqrr::PreparedImage::prepare(gray);
        let grids = prepared.detect_grids();
        assert_eq!(grids[0].decode().unwrap().1, "a");

        let single = Job::from_json5(r#"{ data: "x", output: { format: "jpg" } }"#).unwrap();
        assert_eq!(
            Batch::new(&single, &files).unwrap().name(0).unwrap(),
            "qr.jpg"
        );
        let bad = Job::from_json5(r#"{ data: "x", output: { format: "jpeg", quality: 0 } }"#);
        let err = Batch::new(&bad.unwrap(), &files).err().unwrap().to_string();
        assert!(err.contains("quality must be 1-100"), "{err}");
        assert!(Job::from_json5(r#"{ output: { format: "base64" } }"#).is_err());
    }

    #[test]
    fn job_errors() {
        let files = Files(HashMap::new());
        assert!(Job::from_json5("{ csvv: 1 }").is_err());
        let job = Job::from_json5(r#"{ csv: "nope.csv" }"#).unwrap();
        let err = Batch::new(&job, &files).err().unwrap().to_string();
        assert!(err.contains("cannot read CSV nope.csv"), "{err}");
        let job = Job::from_json5(r#"{ data: "x", canvas: { base_image: "bg.png" } }"#).unwrap();
        let batch = Batch::new(&job, &files).unwrap();
        let err = batch.render(0).err().unwrap().to_string();
        assert!(err.contains("cannot read image bg.png"), "{err}");
    }

    #[test]
    fn decodes_shared_images_once_and_row_images_on_render() {
        let mut png = Vec::new();
        textplate::image::DynamicImage::new_rgba8(4, 4)
            .write_to(
                &mut std::io::Cursor::new(&mut png),
                textplate::image::ImageFormat::Png,
            )
            .unwrap();
        let files = Files(HashMap::from([
            ("rows.csv", b"a.png\nb.png\nbad.png\n".to_vec()),
            ("logo.png", png.clone()),
            ("a.png", png.clone()),
            ("b.png", png),
            ("bad.png", b"not an image".to_vec()),
        ]));
        let job = Job::from_json5(
            r#"{ csv: "rows.csv", canvas: { base_image: "{{1}}" },
                 template: [{ type: "image", src: "logo.png" }] }"#,
        )
        .unwrap();
        let batch = Batch::new(&job, &files).unwrap();
        assert!(batch.assets.contains("logo.png"));
        assert_eq!(batch.encoded.len(), 3);
        assert_eq!(batch.render(0).unwrap().image.dimensions(), (4, 4));
        assert!(batch.render(1).is_ok());
        let err = batch.render(2).err().unwrap().to_string();
        assert!(err.starts_with("cannot decode image bad.png"), "{err}");
    }
}
