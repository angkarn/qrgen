use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use qrgen::{
    default_template, encode_png,
    textplate::{self, Assets, Fonts, Options, Template, TemplateString},
    Generator, Row, UniqueNames,
};
use rayon::prelude::*;

use crate::job::{Format, Job, TemplateSource};

/// Runs a job. Relative paths in the job are resolved against `base`.
/// Returns `false` if any row failed.
pub fn run(job: Job, base: &Path) -> Result<bool> {
    let started = Instant::now();
    let resolve = |p: &Path| {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            base.join(p)
        }
    };

    let template = match &job.template {
        None => default_template(),
        Some(TemplateSource::Path(path)) => {
            let path = resolve(path);
            let src = fs::read_to_string(&path)
                .with_context(|| format!("cannot read template {}", path.display()))?;
            Template::from_json5(&src).with_context(|| format!("in template {}", path.display()))?
        }
        Some(TemplateSource::Inline(value)) => {
            Template::from_value(value.clone()).context("in template")?
        }
    };

    let mut fonts = Fonts::new();
    for font in &job.fonts {
        let path = resolve(&font.path);
        let data =
            fs::read(&path).with_context(|| format!("cannot read font {}", path.display()))?;
        fonts
            .add(&font.name, data)
            .with_context(|| format!("font {}", path.display()))?;
    }
    if fonts.family("default").is_none() {
        fonts.add("default", qrgen::DEFAULT_FONT)?;
    }

    let options = Options {
        strict: job.strict,
        ..Options::default()
    };
    let generator =
        Generator::new(fonts, template, job.canvas.clone(), job.qr.clone()).with_options(options);

    // Rows: CSV rows, or one row whose `{{data}}` is the literal data
    let (headers, records) = match &job.csv {
        Some(path) => read_csv(&resolve(path), job.header)?,
        None => (None, vec![Vec::new()]),
    };
    let single = job.csv.is_none();
    let rows = RowMaker {
        headers,
        // a single image uses the data as-is; CSV rows fill it as a template
        data: match (&job.data, single) {
            (data, true) => RowData::Literal(data.clone().unwrap_or_default()),
            (Some(data), false) => RowData::Template(TemplateString::parse(data)),
            (None, false) => RowData::Template(TemplateString::parse("{{1}}")),
        },
    };
    let make_row = |i: usize, columns| rows.make(i, columns);

    // Output targets, decided up front so names are unique and stable
    let name_template = TemplateString::parse(job.output.name.as_deref().unwrap_or("{{ROW}}"));
    let out_dir = resolve(job.output.dir.as_deref().unwrap_or(Path::new("output")));
    let mut unique = UniqueNames::new();
    let targets: Vec<Result<Target, String>> = records
        .iter()
        .enumerate()
        .map(|(i, columns)| {
            if single {
                return Ok(match job.output.file.as_deref() {
                    Some(p) if p == Path::new("-") => Target::Stdout,
                    Some(p) => Target::File(resolve(p)),
                    None => Target::File(resolve(Path::new("qr.png"))),
                });
            }
            let row = make_row(i, columns)?;
            let name = name_template
                .render(&row)
                .map_err(|n| format!("missing variable {n:?} in output name"))?;
            Ok(Target::File(out_dir.join(unique.next(&name, ".png"))))
        })
        .collect();
    if !single && job.output.format == Format::Png {
        fs::create_dir_all(&out_dir)
            .with_context(|| format!("cannot create {}", out_dir.display()))?;
    }

    // Load every image the rows need, once
    let mut needed = BTreeSet::new();
    for (i, columns) in records.iter().enumerate() {
        if let Ok(row) = make_row(i, columns) {
            needed.extend(generator.asset_names(&row).unwrap_or_default());
        }
    }
    let loaded: Vec<(String, Result<textplate::image::RgbaImage, String>)> = needed
        .into_par_iter()
        .map(|name| {
            let path = resolve(Path::new(&name));
            let image = fs::read(&path)
                .map_err(|e| format!("cannot read image {}: {e}", path.display()))
                .and_then(|bytes| {
                    textplate::image::load_from_memory(&bytes)
                        .map(|img| img.into_rgba8())
                        .map_err(|e| format!("cannot decode image {}: {e}", path.display()))
                });
            (name, image)
        })
        .collect();
    let mut assets = Assets::new();
    let mut asset_errors = HashMap::new();
    for (name, image) in loaded {
        match image {
            Ok(img) => {
                assets.insert(name, img);
            }
            Err(e) => {
                asset_errors.insert(name, e);
            }
        }
    }

    let stop = AtomicBool::new(false);
    let label = |i: usize| {
        if single {
            String::new()
        } else {
            format!("row {}: ", i + 1)
        }
    };
    let results: Vec<bool> = records
        .par_iter()
        .enumerate()
        .map(|(i, columns)| {
            if stop.load(Ordering::Relaxed) {
                return false;
            }
            let result = (|| -> Result<(), String> {
                let target = targets[i].clone()?;
                let row = make_row(i, columns)?;
                for name in generator.asset_names(&row).map_err(|e| e.to_string())? {
                    if let Some(e) = asset_errors.get(&name) {
                        return Err(e.clone());
                    }
                }
                let output = generator.render(&row, &assets).map_err(|e| e.to_string())?;
                for overflow in &output.report.overflows {
                    eprintln!("{}warning: {overflow}", label(i));
                }
                let png = encode_png(&output.image).map_err(|e| e.to_string())?;
                write_output(&target, &png, job.output.format, single)
            })();
            match result {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("{}error: {e}", label(i));
                    if job.fail_fast {
                        stop.store(true, Ordering::Relaxed);
                    }
                    false
                }
            }
        })
        .collect();

    let ok = results.iter().filter(|r| **r).count();
    let failed = results.len() - ok;
    if !single || failed > 0 {
        eprintln!(
            "{ok} created, {failed} failed in {:.2}s",
            started.elapsed().as_secs_f64()
        );
    }
    Ok(failed == 0)
}

struct RowMaker {
    headers: Option<Vec<String>>,
    data: RowData,
}

enum RowData {
    Literal(String),
    Template(TemplateString),
}

impl RowMaker {
    fn make<'a>(&'a self, i: usize, columns: &'a [String]) -> Result<Row<'a>, String> {
        let mut row = Row::new(i, columns);
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
}

#[derive(Clone)]
enum Target {
    File(PathBuf),
    Stdout,
}

fn write_output(target: &Target, png: &[u8], format: Format, single: bool) -> Result<(), String> {
    match (target, format) {
        (Target::Stdout, Format::Png) => std::io::stdout()
            .lock()
            .write_all(png)
            .map_err(|e| e.to_string()),
        (_, Format::Base64) if single => {
            println!("{}", STANDARD.encode(png));
            Ok(())
        }
        (Target::File(path), Format::Base64) => {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            println!("{name}\t{}", STANDARD.encode(png));
            Ok(())
        }
        (Target::File(path), Format::Png) => {
            if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
                fs::create_dir_all(dir)
                    .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
            }
            fs::write(path, png).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
            println!("{}", path.display());
            Ok(())
        }
        (Target::Stdout, Format::Base64) => unreachable!("handled by the single-image arm"),
    }
}

type Records = (Option<Vec<String>>, Vec<Vec<String>>);

fn read_csv(path: &Path, header: bool) -> Result<Records> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(header)
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("cannot read CSV {}", path.display()))?;
    let headers = if header {
        Some(reader.headers()?.iter().map(str::to_string).collect())
    } else {
        None
    };
    let mut records = Vec::new();
    for (i, record) in reader.records().enumerate() {
        let record = record.with_context(|| format!("CSV {} line {}", path.display(), i + 1))?;
        records.push(record.iter().map(str::to_string).collect());
    }
    if records.is_empty() {
        bail!("CSV {} has no rows", path.display());
    }
    Ok((headers, records))
}
