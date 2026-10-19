use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use qrgen::{encode_png, Batch, FileSource, Format, Job};
use rayon::prelude::*;

/// Reads job files from disk, relative to the job file's folder (or the working directory).
struct DiskFiles<'a> {
    base: &'a Path,
}

impl DiskFiles<'_> {
    fn resolve(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base.join(path)
        }
    }
}

impl FileSource for DiskFiles<'_> {
    fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        fs::read(self.resolve(path)).map_err(|e| e.to_string())
    }
}

/// Runs a job. Relative paths in the job are resolved against `base`.
/// Returns `false` if any row failed.
pub fn run(job: Job, base: &Path) -> Result<bool> {
    let started = Instant::now();
    let files = DiskFiles { base };
    let batch = Batch::new(&job, &files)?;
    let single = job.is_single();

    let out_dir = files.resolve(job.output.dir.as_deref().unwrap_or("output"));
    if !single && job.output.format == Format::Png {
        fs::create_dir_all(&out_dir)
            .with_context(|| format!("cannot create {}", out_dir.display()))?;
    }
    let target = |name: &str| -> Target {
        match (single, name) {
            (true, "-") => Target::Stdout,
            (true, _) => Target::File(files.resolve(name)),
            (false, _) => Target::File(out_dir.join(name)),
        }
    };

    let stop = AtomicBool::new(false);
    let label = |i: usize| {
        if single {
            String::new()
        } else {
            format!("row {}: ", i + 1)
        }
    };
    let results: Vec<bool> = (0..batch.len())
        .into_par_iter()
        .map(|i| {
            if stop.load(Ordering::Relaxed) {
                return false;
            }
            let result = (|| -> Result<(), String> {
                let target = target(batch.name(i)?);
                let output = batch.render(i).map_err(|e| e.to_string())?;
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
