use std::collections::{BTreeSet, HashMap};

use serde::Deserialize;
use serde_json::Value;
use textplate::{Assets, Fonts, Options, Template, TemplateString};

use crate::{
    default_template, Canvas, Error, Generator, Output, QrStyle, Result, Row, UniqueNames,
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
    /// Single image: output file (`-` for stdout in the CLI). Default `qr.png`.
    pub file: Option<String>,
    /// CSV rows: output directory. Default `output`.
    pub dir: Option<String>,
    /// CSV rows: file name template, without extension. Default `{{ROW}}`.
    pub name: Option<String>,
    pub format: Format,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// PNG files
    #[default]
    Png,
    /// Base64-encoded PNGs printed to stdout (CLI)
    Base64,
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
    assets: Assets,
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
            assets: Assets::new(),
            asset_errors: HashMap::new(),
        };

        // File names, decided up front so they are unique and stable
        let name_template = TemplateString::parse(job.output.name.as_deref().unwrap_or("{{ROW}}"));
        let mut unique = UniqueNames::new();
        batch.names = (0..batch.len())
            .map(|i| {
                if job.is_single() {
                    return Ok(job.output.file.clone().unwrap_or_else(|| "qr.png".into()));
                }
                let row = batch.row(i)?;
                let name = name_template
                    .render(&row)
                    .map_err(|n| format!("missing variable {n:?} in output name"))?;
                Ok(unique.next(&name, ".png"))
            })
            .collect();

        // Every image the rows need, loaded once
        let mut needed = BTreeSet::new();
        for i in 0..batch.len() {
            if let Ok(row) = batch.row(i) {
                needed.extend(batch.generator.asset_names(&row).unwrap_or_default());
            }
        }
        for name in needed {
            let image = files.read(&name).and_then(|bytes| {
                textplate::image::load_from_memory(&bytes)
                    .map(|img| img.into_rgba8())
                    .map_err(|e| format!("cannot decode image {name}: {e}"))
            });
            match image {
                Ok(img) => {
                    batch.assets.insert(name, img);
                }
                Err(e) => {
                    let message = if e.starts_with("cannot decode") {
                        e
                    } else {
                        format!("cannot read image {name}: {e}")
                    };
                    batch.asset_errors.insert(name, message);
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
        for name in self.generator.asset_names(&row)? {
            if let Some(e) = self.asset_errors.get(&name) {
                return Err(Error::Data(e.clone()));
            }
        }
        self.generator.render(&row, &self.assets)
    }

    pub fn generator(&self) -> &Generator {
        &self.generator
    }
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
        let err = batch.render(2).err().unwrap().to_string();
        assert!(err.contains("missing variable \"id\""), "{err}");
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
}
