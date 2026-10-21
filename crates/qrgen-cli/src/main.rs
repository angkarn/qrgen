mod run;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use qrgen::{textplate::Color, Ecc};

use qrgen::{FontSpec, Format, Job, TemplateSource};

/// Generate QR code images with text, from one value or every row of a CSV file.
#[derive(Parser)]
#[command(name = "qrgen", version, about, max_term_width = 100)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate one image
    Gen {
        /// QR code content
        data: String,
        /// Output file (`-` for stdout) [default: qr.png, or qr.jpg with --format jpeg]
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[command(flatten)]
        common: Common,
    },
    /// Generate one image per CSV row
    #[command(
        after_help = "Templates can use {{1}}, {{2}}, ... for columns, {{ROW}} for the row number, \
{{data}} for the QR content and, with --header, column names like {{email}}."
    )]
    Batch {
        /// CSV file
        csv: PathBuf,
        /// QR content template
        #[arg(short, long, default_value = "{{1}}")]
        data: String,
        /// The first CSV row holds column names
        #[arg(long)]
        header: bool,
        /// Output directory
        #[arg(short, long, default_value = "output")]
        out_dir: PathBuf,
        /// File name template (".png" or ".jpg" is added)
        #[arg(short, long, default_value = "{{ROW}}")]
        name: String,
        /// Stop at the first failed row
        #[arg(long)]
        fail_fast: bool,
        /// Worker threads (default: all cores)
        #[arg(short, long)]
        jobs: Option<usize>,
        #[command(flatten)]
        common: Common,
    },
    /// Run a job file (JSON5); relative paths are resolved from the file's folder
    Run {
        job: PathBuf,
        /// Worker threads (default: all cores)
        #[arg(short, long)]
        jobs: Option<usize>,
    },
    /// Print a QR code in the terminal
    Show {
        data: String,
        #[arg(long, default_value = "m")]
        ecc: Ecc,
        /// Dark modules as blocks (for light terminal themes)
        #[arg(long)]
        invert: bool,
    },
    /// Print the license of the built-in font
    Licenses,
}

#[derive(Args)]
struct Common {
    /// Layout template (JSON5). Default: a QR code filling the image
    #[arg(short, long)]
    template: Option<PathBuf>,
    /// Image width in pixels
    #[arg(long, default_value_t = 1000)]
    width: u32,
    /// Image height in pixels [default: width]
    #[arg(long)]
    height: Option<u32>,
    /// Background color
    #[arg(long, default_value = "#ffffff")]
    background: Color,
    /// Draw on this image instead (path; may use {{...}} in batch)
    #[arg(long)]
    base_image: Option<String>,
    /// Font as NAME=PATH, repeatable; the first is the default text font
    #[arg(short, long = "font", value_name = "NAME=PATH")]
    fonts: Vec<FontSpec>,
    /// QR error correction level: l, m, q or h
    #[arg(long, default_value = "m")]
    ecc: Ecc,
    /// QR dark module color
    #[arg(long, default_value = "#000000")]
    dark: Color,
    /// QR light module color
    #[arg(long, default_value = "#ffffff")]
    light: Color,
    /// QR quiet zone in modules
    #[arg(long, default_value_t = 1)]
    quiet_zone: u32,
    /// Image format
    #[arg(long, value_enum, default_value_t = OutputFormat::Png)]
    format: OutputFormat,
    /// JPEG quality, 1-100
    #[arg(long, default_value_t = qrgen::DEFAULT_JPEG_QUALITY,
          value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: u8,
    /// Print base64-encoded images to stdout instead of writing files
    #[arg(long)]
    base64: bool,
    /// Fail when text does not fit its box (default: warn)
    #[arg(long)]
    strict: bool,
}

impl Common {
    fn into_job(self) -> Job {
        let mut job = Job {
            template: self.template.map(|p| TemplateSource::Path(path_string(&p))),
            fonts: self.fonts,
            strict: self.strict,
            ..Job::default()
        };
        job.canvas.width = self.width;
        job.canvas.height = self.height;
        job.canvas.background = self.background;
        job.canvas.base_image = self
            .base_image
            .as_deref()
            .map(qrgen::textplate::TemplateString::parse);
        job.qr = qrgen::QrStyle {
            ecc: self.ecc,
            dark: self.dark,
            light: self.light,
            quiet_zone: self.quiet_zone,
        };
        job.output.format = match self.format {
            OutputFormat::Png => Format::Png,
            OutputFormat::Jpeg => Format::Jpeg,
        };
        job.output.quality = Some(self.quality);
        job.output.base64 = self.base64;
        job
    }
}

fn main() -> ExitCode {
    match run_cli(Cli::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli(cli: Cli) -> Result<bool> {
    match cli.command {
        Command::Gen {
            data,
            output,
            common,
        } => {
            let mut job = common.into_job();
            job.data = Some(data);
            job.output.file = output.map(|p| path_string(&p));
            run::run(job, Path::new(""))
        }
        Command::Batch {
            csv,
            data,
            header,
            out_dir,
            name,
            fail_fast,
            jobs,
            common,
        } => {
            let mut job = common.into_job();
            job.csv = Some(path_string(&csv));
            job.data = Some(data);
            job.header = header;
            job.output.dir = Some(path_string(&out_dir));
            job.output.name = Some(name);
            job.fail_fast = fail_fast;
            with_threads(jobs, || run::run(job, Path::new("")))
        }
        Command::Run { job: path, jobs } => {
            let src = std::fs::read_to_string(&path)
                .with_context(|| format!("cannot read {}", path.display()))?;
            let job = Job::from_json5(&src)
                .with_context(|| format!("invalid job file {}", path.display()))?;
            let base = path.parent().unwrap_or(Path::new("")).to_path_buf();
            with_threads(jobs, || run::run(job, &base))
        }
        Command::Show { data, ecc, invert } => {
            print!("{}", qrgen::to_terminal(&data, ecc, 1, invert)?);
            Ok(true)
        }
        Command::Licenses => {
            print!("{}", qrgen::DEFAULT_FONT_LICENSE);
            Ok(true)
        }
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Png,
    /// Smaller files for photos; transparent pixels are drawn on white
    Jpeg,
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn with_threads(jobs: Option<usize>, f: impl FnOnce() -> Result<bool> + Send) -> Result<bool> {
    match jobs {
        Some(n) => rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()?
            .install(f),
        None => f(),
    }
}
